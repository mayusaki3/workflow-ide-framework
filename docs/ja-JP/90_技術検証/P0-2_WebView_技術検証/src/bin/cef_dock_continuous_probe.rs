//! WV-11-03 CEF Browser Surface Dock 継続更新 Probe。
//!
//! 役割:
//! - TEX-IT-SPEC-006 の Dock 表示継続更新を Windows 実機で検証する。
//! - Browser 側で一定間隔に表示内容を変更し、CEF OSR Paint を同一 egui Texture へ継続反映する。
//!
//! 注意点:
//! - 本ファイルは技術検証用であり、正式 Surface API ではない。
//! - 入力イベント転送は WV-11-04 で検証する。
//! - Windows では CEF の multi-threaded message loop を使用する。
//! - Windowless Browser は CEF UI thread の `on_context_initialized` から非同期生成する。
//! - Browser 側の JavaScript タイマーだけに依存せず、Browser Process 側から定期的に JavaScript を実行して描画変化を発生させる。

use cef::*;
use eframe::egui;
use egui_dock::{DockArea, DockState, TabViewer};
use std::ptr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::sleep;
use std::time::{Duration, Instant};

const VIEW_WIDTH: i32 = 800;
const VIEW_HEIGHT: i32 = 600;
const MESSAGE_PUMP_INTERVAL: Duration = Duration::from_millis(10);
const BROWSER_CREATE_TIMEOUT: Duration = Duration::from_secs(3);
const CLOSE_TIMEOUT: Duration = Duration::from_secs(3);
const CONTINUOUS_TICK_INTERVAL: Duration = Duration::from_millis(500);
const CONTINUOUS_PASS_GENERATION: u64 = 5;
const TEST_URL: &str = "data:text/html,<html><body style='margin:0;background:rgb(235,245,255);color:rgb(20,30,45);font-family:sans-serif;display:flex;align-items:center;justify-content:center;height:100vh;overflow:hidden'><div style='text-align:center;border:12px solid rgb(20,30,45);padding:48px;min-width:560px'><h1 style='font-size:48px;margin:0 0 24px'>WV-11-03 CONTINUOUS</h1><div id='counter' style='font-size:92px;font-weight:bold'>0</div><p style='font-size:24px;margin:20px 0 0'>CEF OSR continuous Paint to egui Texture</p></div></body></html>";

/// 最新 CEF Paint を保持する共有状態。
///
/// @hldocs.ref doc-20260911-120000Z-WV13#sec_m9a2e6r4t7yc
#[derive(Debug, Default)]
struct ContinuousState {
    width: i32,
    height: i32,
    rgba: Vec<u8>,
    generation: u64,
}

#[derive(Clone)]
struct ContinuousRenderHandler {
    state: Arc<Mutex<ContinuousState>>,
}

wrap_render_handler! {
    struct ContinuousRenderHandlerBuilder {
        handler: ContinuousRenderHandler,
    }

    impl RenderHandler {
        fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut Rect>) {
            if let Some(rect) = rect {
                rect.width = VIEW_WIDTH;
                rect.height = VIEW_HEIGHT;
            }
        }

        /// CEF BGRA Paint を RGBA8 に変換して最新フレームとして保持する。
        ///
        /// @hldocs.ref doc-20260911-120000Z-WV13#sec_m9a2e6r4t7yc
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
            let mut rgba = Vec::with_capacity(byte_len);
            for pixel in source.chunks_exact(4) {
                rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
            }

            let Ok(mut state) = self.handler.state.lock() else {
                return;
            };
            state.width = width;
            state.height = height;
            state.rgba = rgba;
            state.generation = state.generation.saturating_add(1);

            println!(
                "CEF OSR continuous Paint: generation={}, size={}x{}",
                state.generation, state.width, state.height
            );
        }
    }
}

impl ContinuousRenderHandlerBuilder {
    /// 共有 Paint 状態を持つ RenderHandler を生成する。
    fn build(state: Arc<Mutex<ContinuousState>>) -> RenderHandler {
        Self::new(ContinuousRenderHandler { state })
    }
}

#[derive(Clone)]
struct ContinuousLifeSpanHandler {
    browser: Arc<Mutex<Option<Browser>>>,
    browser_created: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
}

wrap_life_span_handler! {
    struct ContinuousLifeSpanHandlerBuilder {
        handler: ContinuousLifeSpanHandler,
    }

    impl LifeSpanHandler {
        fn on_after_created(&self, browser: Option<&mut Browser>) {
            let Some(browser) = browser.cloned() else {
                return;
            };
            if let Ok(mut slot) = self.handler.browser.lock() {
                *slot = Some(browser);
                self.handler.browser_created.store(true, Ordering::Release);
            }
        }

        fn on_before_close(&self, _browser: Option<&mut Browser>) {
            if let Ok(mut slot) = self.handler.browser.lock() {
                *slot = None;
            }
            self.handler.closed.store(true, Ordering::Release);
        }
    }
}

impl ContinuousLifeSpanHandlerBuilder {
    fn build(
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
    ) -> LifeSpanHandler {
        Self::new(ContinuousLifeSpanHandler {
            browser,
            browser_created,
            closed,
        })
    }
}

wrap_client! {
    struct ContinuousClientBuilder {
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

impl ContinuousClientBuilder {
    fn build(
        state: Arc<Mutex<ContinuousState>>,
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
    ) -> Client {
        Self::new(
            ContinuousRenderHandlerBuilder::build(state),
            ContinuousLifeSpanHandlerBuilder::build(browser, browser_created, closed),
        )
    }
}

#[derive(Clone)]
struct ContinuousBrowserProcessHandler {
    state: Arc<Mutex<ContinuousState>>,
    browser: Arc<Mutex<Option<Browser>>>,
    browser_created: Arc<AtomicBool>,
    browser_create_failed: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
}

wrap_browser_process_handler! {
    struct ContinuousBrowserProcessHandlerBuilder {
        handler: ContinuousBrowserProcessHandler,
    }

    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            let mut client = ContinuousClientBuilder::build(
                self.handler.state.clone(),
                self.handler.browser.clone(),
                self.handler.browser_created.clone(),
                self.handler.closed.clone(),
            );
            let window_info = WindowInfo {
                windowless_rendering_enabled: 1,
                ..Default::default()
            };
            let browser_settings = BrowserSettings {
                windowless_frame_rate: 30,
                ..Default::default()
            };
            let url = CefString::from(TEST_URL);

            let accepted = browser_host_create_browser(
                Some(&window_info),
                Some(&mut client),
                Some(&url),
                Some(&browser_settings),
                None,
                None,
            );
            if accepted != 1 {
                self.handler
                    .browser_create_failed
                    .store(true, Ordering::Release);
            }
        }
    }
}

impl ContinuousBrowserProcessHandlerBuilder {
    fn build(handler: ContinuousBrowserProcessHandler) -> BrowserProcessHandler {
        Self::new(handler)
    }
}

#[derive(Clone)]
struct ContinuousCefApp {
    browser_process_handler: BrowserProcessHandler,
}

wrap_app! {
    struct ContinuousCefAppBuilder {
        handler: ContinuousCefApp,
    }

    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(self.handler.browser_process_handler.clone())
        }
    }
}

impl ContinuousCefAppBuilder {
    fn build(handler: ContinuousBrowserProcessHandler) -> App {
        Self::new(ContinuousCefApp {
            browser_process_handler: ContinuousBrowserProcessHandlerBuilder::build(handler),
        })
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
        eprintln!(
            "WV-11-03 continuous subprocess failed: cef_execute_process returned {exit_code}"
        );
        1
    }
}

#[derive(Clone)]
enum ContinuousTab {
    Browser,
}

struct ContinuousViewer<'a> {
    texture: Option<&'a egui::TextureHandle>,
}

impl<'a> TabViewer for ContinuousViewer<'a> {
    type Tab = ContinuousTab;

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn title(&mut self, _tab: &mut Self::Tab) -> egui::WidgetText {
        "Browser Surface Continuous".into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, _tab: &mut Self::Tab) {
        let available = ui.available_size();
        if let Some(texture) = self.texture {
            let texture_size = texture.size_vec2();
            let scale = (available.x / texture_size.x)
                .min(available.y / texture_size.y)
                .max(0.01);
            let display_size = texture_size * scale;
            ui.centered_and_justified(|ui| {
                ui.image((texture.id(), display_size));
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Waiting for continuous CEF OSR Paint...");
            });
        }
    }
}

struct ContinuousRuntime {
    state: Arc<Mutex<ContinuousState>>,
    browser: Arc<Mutex<Option<Browser>>>,
    closed: Arc<AtomicBool>,
}

impl ContinuousRuntime {
    /// CEF と Windowless Browser を初期化する。
    fn new() -> Result<Self, String> {
        let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
        let args = args::Args::new();
        let process_result = execute_process(Some(args.as_main_args()), None, ptr::null_mut());
        if process_result >= 0 {
            return Err(format!(
                "Unexpected subprocess result in browser process: {process_result}"
            ));
        }

        #[cfg(target_os = "windows")]
        let settings = Settings {
            windowless_rendering_enabled: 1,
            no_sandbox: 1,
            multi_threaded_message_loop: 1,
            ..Default::default()
        };

        #[cfg(not(target_os = "windows"))]
        let settings = Settings {
            windowless_rendering_enabled: 1,
            no_sandbox: 1,
            ..Default::default()
        };

        let state = Arc::new(Mutex::new(ContinuousState::default()));
        let browser = Arc::new(Mutex::new(None));
        let browser_created = Arc::new(AtomicBool::new(false));
        let browser_create_failed = Arc::new(AtomicBool::new(false));
        let closed = Arc::new(AtomicBool::new(false));

        let handler = ContinuousBrowserProcessHandler {
            state: state.clone(),
            browser: browser.clone(),
            browser_created: browser_created.clone(),
            browser_create_failed: browser_create_failed.clone(),
            closed: closed.clone(),
        };
        let mut app = ContinuousCefAppBuilder::build(handler);

        let initialized = initialize(
            Some(args.as_main_args()),
            Some(&settings),
            Some(&mut app),
            ptr::null_mut(),
        );
        if initialized != 1 {
            return Err(format!("cef_initialize returned {initialized}"));
        }

        #[cfg(target_os = "windows")]
        {
            let started = Instant::now();
            while !browser_created.load(Ordering::Acquire)
                && !browser_create_failed.load(Ordering::Acquire)
                && started.elapsed() < BROWSER_CREATE_TIMEOUT
            {
                sleep(MESSAGE_PUMP_INTERVAL);
            }
            if browser_create_failed.load(Ordering::Acquire) {
                shutdown();
                return Err("CEF asynchronous windowless browser creation was rejected".to_string());
            }
            if !browser_created.load(Ordering::Acquire) {
                shutdown();
                return Err("CEF asynchronous windowless browser creation timed out".to_string());
            }
        }

        Ok(Self {
            state,
            browser,
            closed,
        })
    }

    /// 必要なプラットフォームで CEF メッセージを処理する。
    fn pump(&self) {
        #[cfg(not(target_os = "windows"))]
        do_message_loop_work();
    }

    /// Browser Process 側から表示内容を1回変更し、OSR 再描画を要求する。
    ///
    /// # 引数
    /// - `counter`: 表示する連番。
    ///
    /// # 戻り値
    /// - JavaScript 実行要求を CEF UI thread へ送信できた場合は `true`。
    fn request_tick(&self, counter: u64) -> bool {
        let browser = self
            .browser
            .lock()
            .ok()
            .and_then(|slot| slot.as_ref().cloned());
        let Some(browser) = browser else {
            return false;
        };

        #[cfg(target_os = "windows")]
        {
            let mut task = ContinuousTickTaskBuilder::build(browser, counter);
            return post_task(ThreadId::UI, Some(&mut task)) == 1;
        }

        #[cfg(not(target_os = "windows"))]
        {
            execute_tick(&browser, counter);
            true
        }
    }

    /// UI 診断表示用に最新 Paint 状態を取得する。
    fn diagnostics(&self) -> (u64, i32, i32, usize) {
        let Ok(state) = self.state.lock() else {
            return (0, 0, 0, 0);
        };
        (
            state.generation,
            state.width,
            state.height,
            state.rgba.len(),
        )
    }
}

/// Browser の DOM を更新し、OSR の再描画を要求する。
///
/// # 引数
/// - `browser`: 更新対象 Browser。
/// - `counter`: 表示する連番。
fn execute_tick(browser: &Browser, counter: u64) {
    let Some(frame) = browser.main_frame() else {
        eprintln!("WV-11-03 continuous probe: main frame not available");
        return;
    };

    let r = 120 + (counter * 37) % 120;
    let g = 150 + (counter * 53) % 90;
    let b = 170 + (counter * 71) % 80;
    let script = format!(
        "document.getElementById('counter').textContent='{counter}';document.body.style.backgroundColor='rgb({r},{g},{b})';"
    );
    let script = CefString::from(script.as_str());
    let source_url = CefString::from("wv11-03-continuous-probe");
    frame.execute_java_script(Some(&script), Some(&source_url), 1);

    if let Some(host) = browser.host() {
        host.invalidate(PaintElementType::VIEW);
    }
    println!("CEF OSR continuous tick requested: counter={counter}");
}

#[derive(Clone)]
struct ContinuousTickTask {
    browser: Browser,
    counter: u64,
}

wrap_task! {
    struct ContinuousTickTaskBuilder {
        task: ContinuousTickTask,
    }

    impl Task {
        fn execute(&self) {
            execute_tick(&self.task.browser, self.task.counter);
        }
    }
}

impl ContinuousTickTaskBuilder {
    /// CEF UI thread で実行する表示更新 Task を生成する。
    ///
    /// # 引数
    /// - `browser`: 更新対象 Browser。
    /// - `counter`: 表示する連番。
    ///
    /// # 戻り値
    /// - CEF Task。
    fn build(browser: Browser, counter: u64) -> Task {
        Self::new(ContinuousTickTask { browser, counter })
    }
}

#[derive(Clone)]
struct CloseBrowserTask {
    browser: Browser,
}

wrap_task! {
    struct CloseBrowserTaskBuilder {
        task: CloseBrowserTask,
    }

    impl Task {
        fn execute(&self) {
            if let Some(host) = self.task.browser.host() {
                host.close_browser(true.into());
            }
        }
    }
}

impl CloseBrowserTaskBuilder {
    fn build(browser: Browser) -> Task {
        Self::new(CloseBrowserTask { browser })
    }
}

impl Drop for ContinuousRuntime {
    fn drop(&mut self) {
        let browser = self
            .browser
            .lock()
            .ok()
            .and_then(|slot| slot.as_ref().cloned());

        if let Some(browser) = browser {
            #[cfg(target_os = "windows")]
            {
                let mut task = CloseBrowserTaskBuilder::build(browser);
                if post_task(ThreadId::UI, Some(&mut task)) != 1 {
                    eprintln!("WV-11-03 continuous probe: failed to post browser close task");
                }
            }

            #[cfg(not(target_os = "windows"))]
            if let Some(host) = browser.host() {
                host.close_browser(true.into());
            }
        }

        let started = Instant::now();
        while !self.closed.load(Ordering::Acquire) && started.elapsed() < CLOSE_TIMEOUT {
            #[cfg(not(target_os = "windows"))]
            do_message_loop_work();
            sleep(MESSAGE_PUMP_INTERVAL);
        }
        if !self.closed.load(Ordering::Acquire) {
            eprintln!("WV-11-03 continuous probe: browser close timeout");
        }
        shutdown();
    }
}

struct ContinuousEframeApp {
    dock_state: DockState<ContinuousTab>,
    runtime: ContinuousRuntime,
    texture: Option<egui::TextureHandle>,
    applied_generation: u64,
    next_tick: u64,
    last_tick_at: Instant,
}

impl ContinuousEframeApp {
    fn new(runtime: ContinuousRuntime) -> Self {
        Self {
            dock_state: DockState::new(vec![ContinuousTab::Browser]),
            runtime,
            texture: None,
            applied_generation: 0,
            next_tick: 1,
            last_tick_at: Instant::now(),
        }
    }

    /// 最新 Paint を同一サイズの既存 Texture へ継続反映する。
    fn update_texture(&mut self, ctx: &egui::Context) {
        let snapshot = {
            let Ok(state) = self.runtime.state.lock() else {
                return;
            };
            if state.generation == self.applied_generation
                || state.width <= 0
                || state.height <= 0
                || state.rgba.is_empty()
            {
                None
            } else {
                Some((
                    state.generation,
                    state.width as usize,
                    state.height as usize,
                    state.rgba.clone(),
                ))
            }
        };

        let Some((generation, width, height, rgba)) = snapshot else {
            return;
        };

        let image = egui::ColorImage::from_rgba_unmultiplied([width, height], &rgba);
        match self.texture.as_mut() {
            Some(texture) if texture.size() == [width, height] => {
                texture.set(image, egui::TextureOptions::LINEAR);
            }
            _ => {
                self.texture = Some(ctx.load_texture(
                    "wv11_03_continuous_probe",
                    image,
                    egui::TextureOptions::LINEAR,
                ));
            }
        }
        self.applied_generation = generation;
    }

    /// 検証用の Browser 表示更新を一定間隔で発行する。
    fn update_browser_content(&mut self) {
        if self.last_tick_at.elapsed() < CONTINUOUS_TICK_INTERVAL {
            return;
        }

        if self.runtime.request_tick(self.next_tick) {
            self.next_tick = self.next_tick.saturating_add(1);
            self.last_tick_at = Instant::now();
        }
    }
}

impl eframe::App for ContinuousEframeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.runtime.pump();
        self.update_browser_content();
        self.update_texture(ctx);

        let (generation, paint_width, paint_height, rgba_bytes) = self.runtime.diagnostics();
        let texture_size = self
            .texture
            .as_ref()
            .map(|texture| texture.size())
            .unwrap_or([0, 0]);
        let continuous_ok = generation >= CONTINUOUS_PASS_GENERATION
            && self.applied_generation == generation
            && texture_size == [paint_width.max(0) as usize, paint_height.max(0) as usize];

        egui::TopBottomPanel::top("wv11_03_continuous_status").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("WV-11-03 TEX-IT-SPEC-006");
                ui.separator();
                ui.label(format!("Paint generation: {generation}"));
                ui.separator();
                ui.label(format!("Applied generation: {}", self.applied_generation));
                ui.separator();
                ui.label(format!("Paint: {paint_width}x{paint_height}"));
                ui.separator();
                ui.label(format!("Texture: {}x{}", texture_size[0], texture_size[1]));
                ui.separator();
                ui.label(format!("Continuous: {}", if continuous_ok { "OK" } else { "WAIT" }));
                ui.separator();
                ui.label(format!("RGBA: {rgba_bytes}"));
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut viewer = ContinuousViewer {
                texture: self.texture.as_ref(),
            };
            DockArea::new(&mut self.dock_state).show_inside(ui, &mut viewer);
        });

        ctx.request_repaint_after(MESSAGE_PUMP_INTERVAL);
    }
}

fn main() -> eframe::Result<()> {
    let cli_args: Vec<String> = std::env::args().collect();
    if is_cef_subprocess(&cli_args) {
        std::process::exit(run_subprocess());
    }

    println!("WV-11-03 CEF Dock continuous probe start");
    let runtime = match ContinuousRuntime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("WV-11-03 CEF Dock continuous probe failed: {error}");
            std::process::exit(1);
        }
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("WV-11-03 Browser Surface Continuous Probe")
            .with_inner_size([1000.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "WV-11-03 Browser Surface Continuous Probe",
        native_options,
        Box::new(move |_cc| Ok(Box::new(ContinuousEframeApp::new(runtime)))),
    )
}
