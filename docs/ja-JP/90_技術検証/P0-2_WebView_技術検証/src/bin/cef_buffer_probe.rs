//! WV-11-02 CEF 描画バッファ取得 Probe。
//!
//! 役割:
//! - CEF-IT-SPEC-006 の Paint 画素バッファ取得を単独検証する。
//! - Paint コールバックから BGRA 画素データを所有可能な Vec<u8> へコピーする。
//!
//! 注意点:
//! - 本ファイルは技術検証用であり、正式 API 仕様ではない。
//! - 継続描画更新の判定は CEF-IT-SPEC-007 で行う。
//! - Browser / CEF の正常終了判定は CEF-IT-SPEC-008 で行う。

use cef::*;
use std::cell::RefCell;
use std::ptr;
use std::rc::Rc;
use std::thread::sleep;
use std::time::{Duration, Instant};

const VIEW_WIDTH: i32 = 800;
const VIEW_HEIGHT: i32 = 600;
const BUFFER_TIMEOUT: Duration = Duration::from_secs(10);
const MESSAGE_PUMP_INTERVAL: Duration = Duration::from_millis(10);

/// 描画バッファ観測状態。
///
/// @hldocs.ref doc-20260628-000011Z-WV11#sec_brb8qsq5y25r
#[derive(Debug, Default)]
struct BufferProbeState {
    width: i32,
    height: i32,
    pixels: Vec<u8>,
}

#[derive(Clone)]
struct BufferProbeRenderHandler {
    state: Rc<RefCell<BufferProbeState>>,
}

wrap_render_handler! {
    struct BufferProbeRenderHandlerBuilder {
        handler: BufferProbeRenderHandler,
    }

    impl RenderHandler {
        /// Windowless Browser の描画領域を返す。
        fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut Rect>) {
            if let Some(rect) = rect {
                rect.width = VIEW_WIDTH;
                rect.height = VIEW_HEIGHT;
            }
        }

        /// Paint 画素バッファを所有可能な Vec<u8> へコピーする。
        ///
        /// @hldocs.ref doc-20260628-000011Z-WV11#sec_brb8qsq5y25r
        ///
        /// # 引数
        /// - `_browser`: CEF Browser。Probeでは未使用。
        /// - `_type_`: Paint対象種別。006では種別判定しない。
        /// - `_dirty_rects`: 更新領域。006では内容を検証しない。
        /// - `buffer`: CEF が提供する BGRA 画素バッファ。
        /// - `width`: 描画領域幅。
        /// - `height`: 描画領域高さ。
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

            let Some(pixel_count) = (width as usize)
                .checked_mul(height as usize)
                .and_then(|value| value.checked_mul(4))
            else {
                return;
            };

            let source = unsafe { std::slice::from_raw_parts(buffer, pixel_count) };
            let mut state = self.handler.state.borrow_mut();
            state.width = width;
            state.height = height;
            state.pixels.clear();
            state.pixels.extend_from_slice(source);
        }
    }
}

impl BufferProbeRenderHandlerBuilder {
    /// 共有状態を持つ RenderHandler を生成する。
    fn build(state: Rc<RefCell<BufferProbeState>>) -> RenderHandler {
        Self::new(BufferProbeRenderHandler { state })
    }
}

wrap_client! {
    struct BufferProbeClientBuilder {
        render_handler: RenderHandler,
    }

    impl Client {
        fn render_handler(&self) -> Option<RenderHandler> {
            Some(self.render_handler.clone())
        }
    }
}

impl BufferProbeClientBuilder {
    /// CEF-IT-SPEC-006 用 Client を生成する。
    fn build(state: Rc<RefCell<BufferProbeState>>) -> Client {
        Self::new(BufferProbeRenderHandlerBuilder::build(state))
    }
}

/// CEF subprocess かを判定する。
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
        eprintln!(
            "WV-11-02 CEF subprocess failed: cef_execute_process returned {exit_code}"
        );
        1
    }
}

/// Browser を閉じ、CEF shutdown 前の後始末用メッセージ処理を行う。
fn close_browser_for_probe(browser: &mut Browser) {
    if let Some(host) = browser.host() {
        host.close_browser(true.into());
    }

    for _ in 0..50 {
        do_message_loop_work();
        sleep(MESSAGE_PUMP_INTERVAL);
    }
}

/// CEF-IT-SPEC-006 描画バッファ取得 Probe を実行する。
///
/// @hldocs.ref doc-20260628-000011Z-WV11#sec_brb8qsq5y25r
///
/// # 戻り値
/// - 成功時: 描画サイズとコピー済みバイト数を含む文字列。
/// - 失敗時: 初期化、Browser作成、Buffer timeout等の理由。
fn run_buffer_probe() -> Result<String, String> {
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    let args = args::Args::new();

    let process_result = execute_process(Some(args.as_main_args()), None, ptr::null_mut());
    if process_result >= 0 {
        return Ok(format!(
            "CEF subprocess completed with exit code {process_result}"
        ));
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
            "CEF initialize failed before buffer probe: cef_initialize returned {initialized}"
        ));
    }

    let state = Rc::new(RefCell::new(BufferProbeState::default()));
    let mut client = BufferProbeClientBuilder::build(state.clone());
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
        Some(&"data:text/html,<html><body>WV-11-02%20Buffer%20Probe</body></html>".into()),
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

        {
            let observed = state.borrow();
            let expected_len = if observed.width > 0 && observed.height > 0 {
                (observed.width as usize)
                    .checked_mul(observed.height as usize)
                    .and_then(|value| value.checked_mul(4))
            } else {
                None
            };

            if expected_len.is_some_and(|len| len > 0 && observed.pixels.len() == len) {
                break;
            }
        }

        if started.elapsed() >= BUFFER_TIMEOUT {
            close_browser_for_probe(&mut browser);
            shutdown();
            return Err(format!(
                "CEF paint buffer timeout after {} ms",
                BUFFER_TIMEOUT.as_millis()
            ));
        }

        sleep(MESSAGE_PUMP_INTERVAL);
    }

    let (width, height, byte_len) = {
        let observed = state.borrow();
        (observed.width, observed.height, observed.pixels.len())
    };

    close_browser_for_probe(&mut browser);
    shutdown();

    Ok(format!(
        "CEF paint buffer captured: size={width}x{height}, bytes={byte_len}"
    ))
}

/// WV-11-02 CEF Buffer Probe エントリーポイント。
fn main() {
    let cli_args: Vec<String> = std::env::args().collect();

    if is_cef_subprocess(&cli_args) {
        std::process::exit(run_subprocess());
    }

    println!("WV-11-02 CEF buffer probe start");

    match run_buffer_probe() {
        Ok(message) => {
            println!("{message}");
            println!("WV-11-02 CEF buffer probe OK");
        }
        Err(error) => {
            eprintln!("WV-11-02 CEF buffer probe failed");
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
