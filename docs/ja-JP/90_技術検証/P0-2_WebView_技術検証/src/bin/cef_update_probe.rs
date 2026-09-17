//! WV-11-02 CEF 継続描画更新 Probe。
//!
//! 役割:
//! - CEF-IT-SPEC-007 の継続描画更新を単独検証する。
//! - 初回 Paint 後にページ内容を変更し、追加 Paint と画素バッファ変化を確認する。
//!
//! 注意点:
//! - 本ファイルは技術検証用であり、正式 API 仕様ではない。
//! - Browser / CEF の正常終了判定は CEF-IT-SPEC-008 で行う。

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

/// 継続描画更新の観測状態。
///
/// @hldocs.ref doc-20260628-000011Z-WV11#sec_sdnuof1lgo4n
#[derive(Debug, Default)]
struct UpdateProbeState {
    count: u32,
    width: i32,
    height: i32,
    first_pixels: Option<Vec<u8>>,
    latest_pixels: Vec<u8>,
}

#[derive(Clone)]
struct UpdateProbeRenderHandler {
    state: Rc<RefCell<UpdateProbeState>>,
}

wrap_render_handler! {
    struct UpdateProbeRenderHandlerBuilder {
        handler: UpdateProbeRenderHandler,
    }

    impl RenderHandler {
        fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut Rect>) {
            if let Some(rect) = rect {
                rect.width = VIEW_WIDTH;
                rect.height = VIEW_HEIGHT;
            }
        }

        /// Paint 毎に所有バッファへコピーし、初回と最新状態を保持する。
        ///
        /// @hldocs.ref doc-20260628-000011Z-WV11#sec_sdnuof1lgo4n
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
            let pixels = source.to_vec();

            let mut state = self.handler.state.borrow_mut();
            state.count = state.count.saturating_add(1);
            state.width = width;
            state.height = height;
            if state.first_pixels.is_none() {
                state.first_pixels = Some(pixels.clone());
            }
            state.latest_pixels = pixels;
        }
    }
}

impl UpdateProbeRenderHandlerBuilder {
    fn build(state: Rc<RefCell<UpdateProbeState>>) -> RenderHandler {
        Self::new(UpdateProbeRenderHandler { state })
    }
}

wrap_client! {
    struct UpdateProbeClientBuilder {
        render_handler: RenderHandler,
    }

    impl Client {
        fn render_handler(&self) -> Option<RenderHandler> {
            Some(self.render_handler.clone())
        }
    }
}

impl UpdateProbeClientBuilder {
    fn build(state: Rc<RefCell<UpdateProbeState>>) -> Client {
        Self::new(UpdateProbeRenderHandlerBuilder::build(state))
    }
}

fn is_cef_subprocess(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg == "--type" || arg.starts_with("--type="))
}

fn run_subprocess() -> i32 {
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    let args = args::Args::new();
    let exit_code = execute_process(Some(args.as_main_args()), None, ptr::null_mut());

    if exit_code >= 0 {
        exit_code
    } else {
        eprintln!("WV-11-02 CEF subprocess failed: cef_execute_process returned {exit_code}");
        1
    }
}

fn close_browser_for_probe(browser: &mut Browser) {
    if let Some(host) = browser.host() {
        host.close_browser(true.into());
    }

    for _ in 0..50 {
        do_message_loop_work();
        sleep(MESSAGE_PUMP_INTERVAL);
    }
}

/// CEF-IT-SPEC-007 継続描画更新 Probe を実行する。
///
/// @hldocs.ref doc-20260628-000011Z-WV11#sec_sdnuof1lgo4n
fn run_update_probe() -> Result<String, String> {
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
            "CEF initialize failed before update probe: cef_initialize returned {initialized}"
        ));
    }

    let state = Rc::new(RefCell::new(UpdateProbeState::default()));
    let mut client = UpdateProbeClientBuilder::build(state.clone());
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
        Some(&"data:text/html,<html><body style='background:#ffffff'>WV-11-02%20Update%20Probe</body></html>".into()),
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
        if state.borrow().count > 0 {
            break;
        }
        if initial_started.elapsed() >= UPDATE_TIMEOUT {
            close_browser_for_probe(&mut browser);
            shutdown();
            return Err("Initial CEF paint callback timeout".to_string());
        }
        sleep(MESSAGE_PUMP_INTERVAL);
    }

    let initial_count = state.borrow().count;

    let frame = browser.main_frame().ok_or_else(|| {
        close_browser_for_probe(&mut browser);
        shutdown();
        "CEF main frame is not available".to_string()
    })?;

    frame.execute_java_script(
        Some(&"document.body.style.backgroundColor='#0000ff'; document.body.textContent='WV-11-02 Update Changed';".into()),
        Some(&"about:blank".into()),
        0,
    );

    let update_started = Instant::now();
    loop {
        do_message_loop_work();

        let updated = {
            let observed = state.borrow();
            let content_changed = observed
                .first_pixels
                .as_ref()
                .is_some_and(|first| !observed.latest_pixels.is_empty() && *first != observed.latest_pixels);
            observed.count > initial_count && content_changed
        };

        if updated {
            break;
        }

        if update_started.elapsed() >= UPDATE_TIMEOUT {
            let count = state.borrow().count;
            close_browser_for_probe(&mut browser);
            shutdown();
            return Err(format!(
                "CEF continued paint update timeout: initial_count={initial_count}, final_count={count}"
            ));
        }

        sleep(MESSAGE_PUMP_INTERVAL);
    }

    let (count, width, height) = {
        let observed = state.borrow();
        (observed.count, observed.width, observed.height)
    };

    close_browser_for_probe(&mut browser);
    shutdown();

    Ok(format!(
        "CEF continued paint update received: initial_count={initial_count}, final_count={count}, size={width}x{height}, buffer_changed=true"
    ))
}

fn main() {
    let cli_args: Vec<String> = std::env::args().collect();

    if is_cef_subprocess(&cli_args) {
        std::process::exit(run_subprocess());
    }

    println!("WV-11-02 CEF update probe start");

    match run_update_probe() {
        Ok(message) => {
            println!("{message}");
            println!("WV-11-02 CEF update probe OK");
        }
        Err(error) => {
            eprintln!("WV-11-02 CEF update probe failed");
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
