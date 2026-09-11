//! WV-11-03 CEF -> egui Texture 生成 Probe。
//!
//! 役割:
//! - TEX-IT-SPEC-001 の BGRA -> RGBA8 正規化を検証する。
//! - TEX-IT-SPEC-002 の egui Texture 登録を検証する。
//!
//! 注意点:
//! - 本ファイルは技術検証用であり、正式 Surface API ではない。
//! - Texture の Paint 連動更新は TEX-IT-SPEC-003 で検証する。
//! - Dock Panel 内表示は TEX-IT-SPEC-004 で検証する。

use cef::*;
use std::cell::RefCell;
use std::ptr;
use std::rc::Rc;
use std::thread::sleep;
use std::time::{Duration, Instant};

const VIEW_WIDTH: i32 = 800;
const VIEW_HEIGHT: i32 = 600;
const TEXTURE_TIMEOUT: Duration = Duration::from_secs(10);
const MESSAGE_PUMP_INTERVAL: Duration = Duration::from_millis(10);

/// CEF Paint から取得した BGRA 画素バッファ。
///
/// @hldocs.ref doc-20260911-120000Z-WV13#sec_3n8qk2v5u7am
#[derive(Debug, Default)]
struct TextureProbeState {
    width: i32,
    height: i32,
    bgra: Vec<u8>,
}

#[derive(Clone)]
struct TextureProbeRenderHandler {
    state: Rc<RefCell<TextureProbeState>>,
}

wrap_render_handler! {
    struct TextureProbeRenderHandlerBuilder {
        handler: TextureProbeRenderHandler,
    }

    impl RenderHandler {
        /// Windowless Browser の固定描画領域を返す。
        fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut Rect>) {
            if let Some(rect) = rect {
                rect.width = VIEW_WIDTH;
                rect.height = VIEW_HEIGHT;
            }
        }

        /// CEF OSR の BGRA バッファを Probe 所有領域へコピーする。
        ///
        /// @hldocs.ref doc-20260911-120000Z-WV13#sec_3n8qk2v5u7am
        ///
        /// # 引数
        /// - `_browser`: CEF Browser。Probe では未使用。
        /// - `_type_`: Paint 対象種別。Probe では未使用。
        /// - `_dirty_rects`: 更新領域。Probe では未使用。
        /// - `buffer`: CEF が提供する BGRA 画素バッファ。
        /// - `width`: 描画幅。
        /// - `height`: 描画高さ。
        ///
        /// # 注意点
        /// - CEF の buffer はコールバック外で有効とは限らないため、その場でコピーする。
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
            state.width = width;
            state.height = height;
            state.bgra.clear();
            state.bgra.extend_from_slice(source);
        }
    }
}

impl TextureProbeRenderHandlerBuilder {
    /// 共有 Paint 状態を持つ RenderHandler を生成する。
    fn build(state: Rc<RefCell<TextureProbeState>>) -> RenderHandler {
        Self::new(TextureProbeRenderHandler { state })
    }
}

wrap_client! {
    struct TextureProbeClientBuilder {
        render_handler: RenderHandler,
    }

    impl Client {
        fn render_handler(&self) -> Option<RenderHandler> {
            Some(self.render_handler.clone())
        }
    }
}

impl TextureProbeClientBuilder {
    /// TEX-IT-SPEC-001/002 用 CEF Client を生成する。
    fn build(state: Rc<RefCell<TextureProbeState>>) -> Client {
        Self::new(TextureProbeRenderHandlerBuilder::build(state))
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
///
/// # 注意点
/// - 正常終了順序そのものは WV-11-02 CEF-IT-SPEC-008 で検証済み。
fn close_browser_for_probe(browser: &mut Browser) {
    if let Some(host) = browser.host() {
        host.close_browser(true.into());
    }

    for _ in 0..50 {
        do_message_loop_work();
        sleep(MESSAGE_PUMP_INTERVAL);
    }
}

/// CEF BGRA 画素列を RGBA8 へ正規化する。
///
/// @hldocs.ref doc-20260911-120000Z-WV13#sec_3n8qk2v5u7am
///
/// # 引数
/// - `bgra`: 4 byte / pixel の BGRA 画素列。
///
/// # 戻り値
/// - 成功時: 同一画素数の RGBA8 画素列。
/// - 失敗時: バッファ長が4の倍数でない場合の理由。
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

/// TEX-IT-SPEC-001 / 002 を実行する。
///
/// @hldocs.ref doc-20260911-120000Z-WV13#sec_3n8qk2v5u7am
/// @hldocs.ref doc-20260911-120000Z-WV13#sec_r8m4t2x9c6kp
///
/// # 戻り値
/// - 成功時: RGBA バイト数と Texture サイズを含む文字列。
/// - 失敗時: CEF、Paint、画素形式、Texture 生成の失敗理由。
fn run_texture_probe() -> Result<String, String> {
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
            "CEF initialize failed before texture probe: cef_initialize returned {initialized}"
        ));
    }

    let state = Rc::new(RefCell::new(TextureProbeState::default()));
    let mut client = TextureProbeClientBuilder::build(state.clone());
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
        Some(&"data:text/html,<html><body style='background:#336699;color:white'>WV-11-03%20Texture%20Probe</body></html>".into()),
        Some(&browser_settings),
        None,
        None,
    );

    let Some(mut browser) = browser else {
        shutdown();
        return Err("CEF windowless browser creation returned None".to_string());
    };

    let started = Instant::now();
    loop {
        do_message_loop_work();

        let ready = {
            let observed = state.borrow();
            observed.width > 0 && observed.height > 0 && !observed.bgra.is_empty()
        };
        if ready {
            break;
        }

        if started.elapsed() >= TEXTURE_TIMEOUT {
            close_browser_for_probe(&mut browser);
            shutdown();
            return Err("CEF paint buffer timeout before texture creation".to_string());
        }

        sleep(MESSAGE_PUMP_INTERVAL);
    }

    let (width, height, bgra) = {
        let observed = state.borrow();
        (observed.width, observed.height, observed.bgra.clone())
    };

    let rgba = bgra_to_rgba(&bgra)?;
    let expected_len = (width as usize)
        .checked_mul(height as usize)
        .and_then(|value| value.checked_mul(4))
        .ok_or_else(|| "Texture buffer size overflow".to_string())?;

    if rgba.len() != expected_len {
        close_browser_for_probe(&mut browser);
        shutdown();
        return Err(format!(
            "RGBA buffer length mismatch: expected={expected_len}, actual={}",
            rgba.len()
        ));
    }

    let egui_context = egui::Context::default();
    let image = egui::ColorImage::from_rgba_unmultiplied(
        [width as usize, height as usize],
        &rgba,
    );
    let texture = egui_context.load_texture(
        "wv11_03_texture_probe",
        image,
        egui::TextureOptions::LINEAR,
    );
    let texture_size = texture.size();

    if texture_size != [width as usize, height as usize] {
        close_browser_for_probe(&mut browser);
        shutdown();
        return Err(format!(
            "egui texture size mismatch: expected={}x{}, actual={}x{}",
            width, height, texture_size[0], texture_size[1]
        ));
    }

    close_browser_for_probe(&mut browser);
    shutdown();

    Ok(format!(
        "CEF BGRA normalized and egui texture created: size={}x{}, rgba_bytes={}",
        width,
        height,
        rgba.len()
    ))
}

/// WV-11-03 Texture Probe エントリーポイント。
fn main() {
    let cli_args: Vec<String> = std::env::args().collect();

    if is_cef_subprocess(&cli_args) {
        std::process::exit(run_subprocess());
    }

    println!("WV-11-03 CEF texture probe start");

    match run_texture_probe() {
        Ok(message) => {
            println!("{message}");
            println!("WV-11-03 CEF texture probe OK");
        }
        Err(error) => {
            eprintln!("WV-11-03 CEF texture probe failed");
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
