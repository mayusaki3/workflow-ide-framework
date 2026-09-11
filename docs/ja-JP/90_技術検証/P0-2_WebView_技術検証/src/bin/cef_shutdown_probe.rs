//! WV-11-02 CEF Browser / CEF 終了 Probe。
//!
//! 役割:
//! - CEF-IT-SPEC-008 の Browser / CEF 正常終了を単独検証する。
//! - Browser の close 完了を LifeSpanHandler で観測した後に CEF shutdown を実行する。
//!
//! 注意点:
//! - 本ファイルは技術検証用であり、正式 API 仕様ではない。
//! - Browser close 要求だけで成功扱いせず、`on_before_close` の発生まで確認する。

use cef::*;
use std::cell::Cell;
use std::ptr;
use std::rc::Rc;
use std::thread::sleep;
use std::time::{Duration, Instant};

const VIEW_WIDTH: i32 = 800;
const VIEW_HEIGHT: i32 = 600;
const CLOSE_TIMEOUT: Duration = Duration::from_secs(10);
const MESSAGE_PUMP_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Clone)]
struct ShutdownProbeRenderHandler;

wrap_render_handler! {
    struct ShutdownProbeRenderHandlerBuilder {
        handler: ShutdownProbeRenderHandler,
    }

    impl RenderHandler {
        fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut Rect>) {
            if let Some(rect) = rect {
                rect.width = VIEW_WIDTH;
                rect.height = VIEW_HEIGHT;
            }
        }
    }
}

impl ShutdownProbeRenderHandlerBuilder {
    fn build() -> RenderHandler {
        Self::new(ShutdownProbeRenderHandler)
    }
}

#[derive(Clone)]
struct ShutdownProbeLifeSpanHandler {
    closed: Rc<Cell<bool>>,
}

wrap_life_span_handler! {
    struct ShutdownProbeLifeSpanHandlerBuilder {
        handler: ShutdownProbeLifeSpanHandler,
    }

    impl LifeSpanHandler {
        /// Browser の終了完了を記録する。
        ///
        /// @hldocs.ref doc-20260628-000011Z-WV11#sec_y5zildrpcolm
        fn on_before_close(&self, _browser: Option<&mut Browser>) {
            self.handler.closed.set(true);
        }
    }
}

impl ShutdownProbeLifeSpanHandlerBuilder {
    fn build(closed: Rc<Cell<bool>>) -> LifeSpanHandler {
        Self::new(ShutdownProbeLifeSpanHandler { closed })
    }
}

wrap_client! {
    struct ShutdownProbeClientBuilder {
        render_handler: RenderHandler,
        life_span_handler: LifeSpanHandler,
    }

    impl Client {
        fn render_handler(&self) -> Option<RenderHandler> {
            Some(self.render_handler.clone())
        }

        fn life_span_handler(&self) -> Option<LifeSpanHandler> {
            Some(self.life_span_handler.clone())
        }
    }
}

impl ShutdownProbeClientBuilder {
    fn build(closed: Rc<Cell<bool>>) -> Client {
        Self::new(
            ShutdownProbeRenderHandlerBuilder::build(),
            ShutdownProbeLifeSpanHandlerBuilder::build(closed),
        )
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
        eprintln!("WV-11-02 CEF subprocess failed: cef_execute_process returned {exit_code}");
        1
    }
}

/// CEF-IT-SPEC-008 Browser / CEF 終了 Probe を実行する。
///
/// @hldocs.ref doc-20260628-000011Z-WV11#sec_y5zildrpcolm
///
/// # 戻り値
/// - 成功時: Browser close 完了後に CEF shutdown を実行できたことを示す文字列。
/// - 失敗時: 初期化、Browser 作成、Browser close timeout 等の理由。
fn run_shutdown_probe() -> Result<String, String> {
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
            "CEF initialize failed before shutdown probe: cef_initialize returned {initialized}"
        ));
    }

    let closed = Rc::new(Cell::new(false));
    let mut client = ShutdownProbeClientBuilder::build(closed.clone());
    let window_info = WindowInfo {
        windowless_rendering_enabled: 1,
        ..Default::default()
    };
    let browser_settings = BrowserSettings::default();

    let browser = browser_host_create_browser_sync(
        Some(&window_info),
        Some(&mut client),
        Some(&"about:blank".into()),
        Some(&browser_settings),
        None,
        None,
    );

    let Some(mut browser) = browser else {
        shutdown();
        return Err("CEF windowless browser creation returned None".to_string());
    };

    if let Some(host) = browser.host() {
        host.close_browser(true.into());
    } else {
        shutdown();
        return Err("CEF BrowserHost is not available during shutdown probe".to_string());
    }

    let started = Instant::now();
    while !closed.get() {
        do_message_loop_work();

        if started.elapsed() >= CLOSE_TIMEOUT {
            shutdown();
            return Err(format!(
                "CEF browser close timeout after {} ms",
                CLOSE_TIMEOUT.as_millis()
            ));
        }

        sleep(MESSAGE_PUMP_INTERVAL);
    }

    // Browser close 完了後に残りのCEF処理を短時間流し、shutdownを実行する。
    for _ in 0..10 {
        do_message_loop_work();
        sleep(MESSAGE_PUMP_INTERVAL);
    }

    shutdown();

    Ok("CEF browser close completed and shutdown succeeded".to_string())
}

fn main() {
    let cli_args: Vec<String> = std::env::args().collect();

    if is_cef_subprocess(&cli_args) {
        std::process::exit(run_subprocess());
    }

    println!("WV-11-02 CEF shutdown probe start");

    match run_shutdown_probe() {
        Ok(message) => {
            println!("{message}");
            println!("WV-11-02 CEF shutdown probe OK");
        }
        Err(error) => {
            eprintln!("WV-11-02 CEF shutdown probe failed");
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
