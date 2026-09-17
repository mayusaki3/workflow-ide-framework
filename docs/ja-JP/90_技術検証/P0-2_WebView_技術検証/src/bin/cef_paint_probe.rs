//! WV-11-02 CEF Paint コールバック受信 Probe。
//!
//! 役割:
//! - CEF-IT-SPEC-005 の Windowless Browser Paint コールバック受信を単独検証する。
//! - Paint 回数と描画領域の幅・高さのみを記録する。
//!
//! 注意点:
//! - 本ファイルは技術検証用であり、正式 API 仕様ではない。
//! - 画素バッファ内容の保持・検証は CEF-IT-SPEC-006 で行う。
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
const PAINT_TIMEOUT: Duration = Duration::from_secs(10);
const MESSAGE_PUMP_INTERVAL: Duration = Duration::from_millis(10);

/// Paint コールバックの観測状態。
///
/// @hldocs.ref doc-20260628-000011Z-WV11#sec_twsz1102y36h
#[derive(Debug, Default)]
struct PaintProbeState {
    count: u32,
    width: i32,
    height: i32,
}

#[derive(Clone)]
struct PaintProbeRenderHandler {
    state: Rc<RefCell<PaintProbeState>>,
}

wrap_render_handler! {
    struct PaintProbeRenderHandlerBuilder {
        handler: PaintProbeRenderHandler,
    }

    impl RenderHandler {
        /// Windowless Browser の描画領域を返す。
        ///
        /// # 引数
        /// - `_browser`: CEF Browser。固定サイズ Probe では未使用。
        /// - `rect`: CEF へ返す描画領域。
        fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut Rect>) {
            if let Some(rect) = rect {
                rect.width = VIEW_WIDTH;
                rect.height = VIEW_HEIGHT;
            }
        }

        /// CEF OSR の Paint コールバックを観測する。
        ///
        /// @hldocs.ref doc-20260628-000011Z-WV11#sec_twsz1102y36h
        ///
        /// # 引数
        /// - `_browser`: CEF Browser。観測のみのため未使用。
        /// - `_type_`: Paint 対象種別。005 では種別判定を行わない。
        /// - `_dirty_rects`: 更新領域。005 では内容を検証しない。
        /// - `_buffer`: 画素バッファ。006 まで参照しない。
        /// - `width`: Paint 描画領域幅。
        /// - `height`: Paint 描画領域高さ。
        ///
        /// # 注意点
        /// - 005 では Paint 発生と正の幅・高さだけを記録する。
        fn on_paint(
            &self,
            _browser: Option<&mut Browser>,
            _type_: PaintElementType,
            _dirty_rects: Option<&[Rect]>,
            _buffer: *const u8,
            width: ::std::os::raw::c_int,
            height: ::std::os::raw::c_int,
        ) {
            let mut state = self.handler.state.borrow_mut();
            state.count = state.count.saturating_add(1);
            state.width = width;
            state.height = height;
        }
    }
}

impl PaintProbeRenderHandlerBuilder {
    /// Paint 観測状態を共有する RenderHandler を生成する。
    ///
    /// # 引数
    /// - `state`: Paint 回数と最新サイズを保持する共有状態。
    ///
    /// # 戻り値
    /// - CEF Windowless Browser 用 RenderHandler。
    fn build(state: Rc<RefCell<PaintProbeState>>) -> RenderHandler {
        Self::new(PaintProbeRenderHandler { state })
    }
}

wrap_client! {
    struct PaintProbeClientBuilder {
        render_handler: RenderHandler,
    }

    impl Client {
        /// Windowless Rendering 用 RenderHandler を返す。
        fn render_handler(&self) -> Option<RenderHandler> {
            Some(self.render_handler.clone())
        }
    }
}

impl PaintProbeClientBuilder {
    /// CEF-IT-SPEC-005 用の最小 Client を生成する。
    ///
    /// # 引数
    /// - `state`: Paint 観測状態。
    ///
    /// # 戻り値
    /// - RenderHandler を持つ CEF Client。
    fn build(state: Rc<RefCell<PaintProbeState>>) -> Client {
        Self::new(PaintProbeRenderHandlerBuilder::build(state))
    }
}

/// CEF subprocess かを判定する。
///
/// # 引数
/// - `args`: プロセスのコマンドライン引数。
///
/// # 戻り値
/// - Chromium の `--type` が存在する場合は `true`。
fn is_cef_subprocess(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg == "--type" || arg.starts_with("--type="))
}

/// CEF subprocess を処理する。
///
/// # 戻り値
/// - CEF subprocess の終了コード。
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
///
/// # 引数
/// - `browser`: 閉じる Browser。
///
/// # 注意点
/// - 本処理は Probe 後始末であり、CEF-IT-SPEC-008 の合格判定には使用しない。
fn close_browser_for_probe(browser: &mut Browser) {
    if let Some(host) = browser.host() {
        host.close_browser(true.into());
    }

    for _ in 0..50 {
        do_message_loop_work();
        sleep(MESSAGE_PUMP_INTERVAL);
    }
}

/// CEF-IT-SPEC-005 Paint コールバック受信 Probe を実行する。
///
/// @hldocs.ref doc-20260628-000011Z-WV11#sec_twsz1102y36h
///
/// # 戻り値
/// - 成功時: Paint 回数と描画領域サイズを含む文字列。
/// - 失敗時: 初期化、Browser 作成、Paint timeout 等の理由。
fn run_paint_probe() -> Result<String, String> {
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
            "CEF initialize failed before paint probe: cef_initialize returned {initialized}"
        ));
    }

    let state = Rc::new(RefCell::new(PaintProbeState::default()));
    let mut client = PaintProbeClientBuilder::build(state.clone());
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
        Some(&"data:text/html,<html><body>WV-11-02%20Paint%20Probe</body></html>".into()),
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
            if observed.count > 0 && observed.width > 0 && observed.height > 0 {
                break;
            }
        }

        if started.elapsed() >= PAINT_TIMEOUT {
            close_browser_for_probe(&mut browser);
            shutdown();
            return Err(format!(
                "CEF paint callback timeout after {} ms",
                PAINT_TIMEOUT.as_millis()
            ));
        }

        sleep(MESSAGE_PUMP_INTERVAL);
    }

    let (paint_count, width, height) = {
        let observed = state.borrow();
        (observed.count, observed.width, observed.height)
    };

    close_browser_for_probe(&mut browser);
    shutdown();

    Ok(format!(
        "CEF paint callback received: count={paint_count}, size={width}x{height}"
    ))
}

/// WV-11-02 CEF Paint Probe エントリーポイント。
fn main() {
    let cli_args: Vec<String> = std::env::args().collect();

    if is_cef_subprocess(&cli_args) {
        std::process::exit(run_subprocess());
    }

    println!("WV-11-02 CEF paint probe start");

    match run_paint_probe() {
        Ok(message) => {
            println!("{message}");
            println!("WV-11-02 CEF paint probe OK");
        }
        Err(error) => {
            eprintln!("WV-11-02 CEF paint probe failed");
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
