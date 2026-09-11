//! WV-11-03 Paint 連動 egui Texture 更新 Probe。
//!
//! 役割:
//! - TEX-IT-SPEC-003 の Paint 連動 Texture 更新を単独検証する。
//! - 初回 Paint から生成した TextureHandle を保持し、後続 Paint を同一 Texture へ反映する。
//!
//! 注意点:
//! - 本ファイルは技術検証用であり、正式 Surface API ではない。
//! - Dock Panel 内表示は TEX-IT-SPEC-004 で検証する。

use cef::*;
use std::cell::RefCell;
use std::ptr;
use std::rc::Rc;
use std::thread::sleep;
use std::time::{Duration, Instant};

const VIEW_WIDTH: i32 = 800;
const VIEW_HEIGHT: i32 = 600;
const UPDATE_TIMEOUT: Duration = Duration::from_secs(10);
const MESSAGE_PUMP_INTERVAL: Duration = Duration::from_millis(10);

/// CEF Paint の最新状態。
///
/// @hldocs.ref doc-20260911-120000Z-WV13#sec_q6f1w9n3z8bd
#[derive(Debug, Default)]
struct TextureUpdateState {
    count: u32,
    width: i32,
    height: i32,
    bgra: Vec<u8>,
}

#[derive(Clone)]
struct TextureUpdateRenderHandler {
    state: Rc<RefCell<TextureUpdateState>>,
}

wrap_render_handler! {
    struct TextureUpdateRenderHandlerBuilder {
        handler: TextureUpdateRenderHandler,
    }

    impl RenderHandler {
        fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut Rect>) {
            if let Some(rect) = rect {
                rect.width = VIEW_WIDTH;
                rect.height = VIEW_HEIGHT;
            }
        }

        /// Paint 毎に最新 BGRA バッファを所有領域へコピーする。
        ///
        /// @hldocs.ref doc-20260911-120000Z-WV13#sec_q6f1w9n3z8bd
        fn on_paint(
            &self,
            _browser: Option<&mut Browser>,
            _type_: PaintElementType,
            _dirty_rects: Option<&[Rect]>,
            buffer: *const u8,
            width: ::std::os::raw::c_int,
            height: ::std::os::raw::c_int,
        ) {
            if buffer.is_null() || width <= 0 || height <= 0 {
                return;
            }

            let Some(byte_len) = (width as usize)
                .checked_mul(height as usize)
                .and_then(|value| value.checked_mul(4))
            else {
                return;
            };

            let source = unsafe { std::slice::from_raw_parts(buffer, byte_len) };
            let mut state = self.handler.state.borrow_mut();
            state.count = state.count.saturating_add(1);
            state.width = width;
            state.height = height;
            state.bgra.clear();
            state.bgra.extend_from_slice(source);
        }
    }
}

impl TextureUpdateRenderHandlerBuilder {
    fn build(state: Rc<RefCell<TextureUpdateState>>) -> RenderHandler {
        Self::new(TextureUpdateRenderHandler { state })
    }
}

wrap_client! {
    struct TextureUpdateClientBuilder {
        render_handler: RenderHandler,
    }

    impl Client {
        fn render_handler(&self) -> Option<RenderHandler> {
            Some(self.render_handler.clone())
        }
    }
}

impl TextureUpdateClientBuilder {
    fn build(state: Rc<RefCell<TextureUpdateState>>) -> Client {
        Self::new(TextureUpdateRenderHandlerBuilder::build(state))
    }
}

/// Chromium subprocess かを判定する。
fn is_cef_subprocess(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg == "--type" || arg.starts_with("--type="))
}

/// CEF subprocess を処理する。
fn run_subprocess() -> i32 {
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    let args = args::Args::new();
    let exit_code = execute_process(Some(args.as_main_args()), None, ptr::null_mut());

    if exit_code >= 0 {
        exit_code
    } else {
        eprintln!("WV-11-03 CEF subprocess failed: cef_execute_process returned {exit_code}");
        1
    }
}

/// Probe 後始末として Browser を閉じる。
fn close_browser_for_probe(browser: &mut Browser) {
    if let Some(host) = browser.host() {
        host.close_browser(true.into());
    }

    for _ in 0..50 {
        do_message_loop_work();
        sleep(MESSAGE_PUMP_INTERVAL);
    }
}

/// CEF BGRA を egui 入力用 RGBA8 へ変換する。
fn bgra_to_rgba(bgra: &[u8]) -> Result<Vec<u8>, String> {
    if bgra.len() % 4 != 0 {
        return Err(format!(
            "BGRA buffer length is not divisible by 4: {}",
            bgra.len()
        ));
    }

    let mut rgba = Vec::with_capacity(bgra.len());
    for pixel in bgra.chunks_exact(4) {
        rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
    }
    Ok(rgba)
}

/// TEX-IT-SPEC-003 Paint 連動 Texture 更新 Probe を実行する。
///
/// @hldocs.ref doc-20260911-120000Z-WV13#sec_q6f1w9n3z8bd
///
/// # 戻り値
/// - 成功時: 初回 / 更新後 Paint 回数と同一 Texture ID を含む文字列。
/// - 失敗時: CEF、Paint、Texture 更新の失敗理由。
fn run_texture_update_probe() -> Result<String, String> {
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    let args = args::Args::new();

    let process_result = execute_process(Some(args.as_main_args()), None, ptr::null_mut());
    if process_result >= 0 {
        return Ok(format!("CEF subprocess completed with exit code {process_result}"));
    }

    let settings = Settings {
        windowless_rendering_enabled: 1,
        no_sandbox: 1,
        ..Default::default()
    };

    let initialized = initialize(
        Some(args.as_main_args()),
        Some(&settings),
        None,
        ptr::null_mut(),
    );
    if initialized != 1 {
        return Err(format!(
            "CEF initialize failed before texture update probe: cef_initialize returned {initialized}"
        ));
    }

    let state = Rc::new(RefCell::new(TextureUpdateState::default()));
    let mut client = TextureUpdateClientBuilder::build(state.clone());
    let window_info = WindowInfo {
        windowless_rendering_enabled: 1,
        ..Default::default()
    };
    let browser_settings = BrowserSettings {
        windowless_frame_rate: 30,
        ..Default::default()
    };

    let browser = browser_host_create_browser_sync(
        Some(&window_info),
        Some(&mut client),
        Some(&"data:text/html,<html><body style='background:#ffffff;color:#000000'>WV-11-03%20Texture%20Update</body></html>".into()),
        Some(&browser_settings),
        None,
        None,
    );

    let Some(mut browser) = browser else {
        shutdown();
        return Err("CEF windowless browser creation returned None".to_string());
    };

    let initial_started = Instant::now();
    loop {
        do_message_loop_work();
        if state.borrow().count > 0 && !state.borrow().bgra.is_empty() {
            break;
        }
        if initial_started.elapsed() >= UPDATE_TIMEOUT {
            close_browser_for_probe(&mut browser);
            shutdown();
            return Err("Initial CEF paint timeout before texture creation".to_string());
        }
        sleep(MESSAGE_PUMP_INTERVAL);
    }

    let (initial_count, width, height, initial_bgra) = {
        let observed = state.borrow();
        (
            observed.count,
            observed.width,
            observed.height,
            observed.bgra.clone(),
        )
    };
    let initial_rgba = bgra_to_rgba(&initial_bgra)?;

    let egui_context = egui::Context::default();
    let initial_image = egui::ColorImage::from_rgba_unmultiplied(
        [width as usize, height as usize],
        &initial_rgba,
    );
    let mut texture = egui_context.load_texture(
        "wv11_03_texture_update_probe",
        initial_image,
        egui::TextureOptions::LINEAR,
    );
    let texture_id_before = texture.id();

    let frame = browser.main_frame().ok_or_else(|| {
        close_browser_for_probe(&mut browser);
        shutdown();
        "CEF main frame is not available".to_string()
    })?;
    frame.execute_java_script(
        Some(&"document.body.style.backgroundColor='#0066cc'; document.body.style.color='#ffffff'; document.body.textContent='WV-11-03 Texture Updated';".into()),
        Some(&"about:blank".into()),
        0,
    );

    let update_started = Instant::now();
    loop {
        do_message_loop_work();
        let updated = {
            let observed = state.borrow();
            observed.count > initial_count && observed.bgra != initial_bgra
        };
        if updated {
            break;
        }
        if update_started.elapsed() >= UPDATE_TIMEOUT {
            let final_count = state.borrow().count;
            close_browser_for_probe(&mut browser);
            shutdown();
            return Err(format!(
                "CEF paint update timeout: initial_count={initial_count}, final_count={final_count}"
            ));
        }
        sleep(MESSAGE_PUMP_INTERVAL);
    }

    let (final_count, update_width, update_height, update_bgra) = {
        let observed = state.borrow();
        (
            observed.count,
            observed.width,
            observed.height,
            observed.bgra.clone(),
        )
    };
    if update_width != width || update_height != height {
        close_browser_for_probe(&mut browser);
        shutdown();
        return Err(format!(
            "Unexpected texture size change: initial={}x{}, updated={}x{}",
            width, height, update_width, update_height
        ));
    }

    let update_rgba = bgra_to_rgba(&update_bgra)?;
    let update_image = egui::ColorImage::from_rgba_unmultiplied(
        [update_width as usize, update_height as usize],
        &update_rgba,
    );
    texture.set(update_image, egui::TextureOptions::LINEAR);
    let texture_id_after = texture.id();

    if texture_id_before != texture_id_after {
        close_browser_for_probe(&mut browser);
        shutdown();
        return Err("egui TextureHandle identity changed during update".to_string());
    }

    close_browser_for_probe(&mut browser);
    shutdown();

    Ok(format!(
        "CEF paint updated existing egui texture: initial_count={initial_count}, final_count={final_count}, size={width}x{height}, texture_id={texture_id_before:?}"
    ))
}

fn main() {
    let cli_args: Vec<String> = std::env::args().collect();

    if is_cef_subprocess(&cli_args) {
        std::process::exit(run_subprocess());
    }

    println!("WV-11-03 CEF texture update probe start");

    match run_texture_update_probe() {
        Ok(message) => {
            println!("{message}");
            println!("WV-11-03 CEF texture update probe OK");
        }
        Err(error) => {
            eprintln!("WV-11-03 CEF texture update probe failed");
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
