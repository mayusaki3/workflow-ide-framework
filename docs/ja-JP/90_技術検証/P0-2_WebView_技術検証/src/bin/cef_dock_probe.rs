//! WV-11-03 CEF Browser Surface Dock 表示 Probe。
//!
//! 役割:
//! - TEX-IT-SPEC-004 の Browser Surface Dock Panel 内表示を検証する。
//! - CEF OSR Paint を RGBA8 へ変換し、egui Texture として egui_dock 内へ描画する。
//!
//! 注意点:
//! - 本ファイルは技術検証用であり、正式 Surface API ではない。
//! - Resize 同期は TEX-IT-SPEC-005 で検証する。
//! - 継続更新は TEX-IT-SPEC-006 で検証する。
//! - 入力イベント転送は WV-11-04 で検証する。
//! - CEF subprocess は起動引数の `--type=` を判定し、Browser Process の GUI 初期化前に処理する。
//! - Windows では CEF の multi-threaded message loop を使用し、eframe/winit のイベントループと分離する。
//! - Windowless Browser は CEF UI thread の `on_context_initialized` から非同期生成する。

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
const TEST_URL: &str = "data:text/html,<html><body style='margin:0;background:#17324d;color:white;font-family:sans-serif;display:flex;align-items:center;justify-content:center;height:100vh'><div style='text-align:center'><h1>WV-11-03 Browser Surface</h1><p>CEF OSR rendered inside egui_dock</p></div></body></html>";

/// CEF Paint の最新フレーム。
///
/// @hldocs.ref doc-20260911-120000Z-WV13#sec_h5v2c8m7p1rs
#[derive(Debug, Default)]
struct DockProbeState {
    width: i32,
    height: i32,
    rgba: Vec<u8>,
    generation: u64,
}

#[derive(Clone)]
struct DockProbeRenderHandler {
    state: Arc<Mutex<DockProbeState>>,
}

wrap_render_handler! {
    struct DockProbeRenderHandlerBuilder {
        handler: DockProbeRenderHandler,
    }

    impl RenderHandler {
        fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut Rect>) {
            if let Some(rect) = rect {
                rect.width = VIEW_WIDTH;
                rect.height = VIEW_HEIGHT;
            }
        }

        /// CEF BGRA Paint を RGBA8 に正規化して最新フレームとして保持する。
        ///
        /// @hldocs.ref doc-20260911-120000Z-WV13#sec_h5v2c8m7p1rs
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
        }
    }
}

impl DockProbeRenderHandlerBuilder {
    fn build(state: Arc<Mutex<DockProbeState>>) -> RenderHandler {
        Self::new(DockProbeRenderHandler { state })
    }
}

#[derive(Clone)]
struct DockProbeLifeSpanHandler {
    browser: Arc<Mutex<Option<Browser>>>,
    browser_created: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
}

wrap_life_span_handler! {
    struct DockProbeLifeSpanHandlerBuilder {
        handler: DockProbeLifeSpanHandler,
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

impl DockProbeLifeSpanHandlerBuilder {
    fn build(
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
    ) -> LifeSpanHandler {
        Self::new(DockProbeLifeSpanHandler {
            browser,
            browser_created,
            closed,
        })
    }
}

wrap_client! {
    struct DockProbeClientBuilder {
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

impl DockProbeClientBuilder {
    fn build(
        state: Arc<Mutex<DockProbeState>>,
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
    ) -> Client {
        Self::new(
            DockProbeRenderHandlerBuilder::build(state),
            DockProbeLifeSpanHandlerBuilder::build(browser, browser_created, closed),
        )
    }
}

#[derive(Clone)]
struct DockProbeBrowserProcessHandler {
    state: Arc<Mutex<DockProbeState>>,
    browser: Arc<Mutex<Option<Browser>>>,
    browser_created: Arc<AtomicBool>,
    browser_create_failed: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
}

wrap_browser_process_handler! {
    struct DockProbeBrowserProcessHandlerBuilder {
        handler: DockProbeBrowserProcessHandler,
    }

    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            let mut client = DockProbeClientBuilder::build(
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

impl DockProbeBrowserProcessHandlerBuilder {
    fn build(handler: DockProbeBrowserProcessHandler) -> BrowserProcessHandler {
        Self::new(handler)
    }
}

#[derive(Clone)]
struct DockProbeCefApp {
    browser_process_handler: BrowserProcessHandler,
}

wrap_app! {
    struct DockProbeCefAppBuilder {
        handler: DockProbeCefApp,
    }

    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(self.handler.browser_process_handler.clone())
        }
    }
}

impl DockProbeCefAppBuilder {
    fn build(handler: DockProbeBrowserProcessHandler) -> App {
        Self::new(DockProbeCefApp {
            browser_process_handler: DockProbeBrowserProcessHandlerBuilder::build(handler),
        })
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
        eprintln!("WV-11-03 CEF subprocess failed: cef_execute_process returned {exit_code}");
        1
    }
}

#[derive(Clone)]
enum DockProbeTab {
    Browser,
}

struct DockProbeViewer<'a> {
    texture: Option<&'a egui::TextureHandle>,
}

impl<'a> TabViewer for DockProbeViewer<'a> {
    type Tab = DockProbeTab;

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn title(&mut self, _tab: &mut Self::Tab) -> egui::WidgetText {
        "Browser Surface".into()
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
                ui.label("Waiting for CEF OSR Paint...");
            });
        }
    }
}

struct CefDockRuntime {
    state: Arc<Mutex<DockProbeState>>,
    browser: Arc<Mutex<Option<Browser>>>,
    closed: Arc<AtomicBool>,
}

impl CefDockRuntime {
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

        let state = Arc::new(Mutex::new(DockProbeState::default()));
        let browser = Arc::new(Mutex::new(None));
        let browser_created = Arc::new(AtomicBool::new(false));
        let browser_create_failed = Arc::new(AtomicBool::new(false));
        let closed = Arc::new(AtomicBool::new(false));

        let handler = DockProbeBrowserProcessHandler {
            state: state.clone(),
            browser: browser.clone(),
            browser_created: browser_created.clone(),
            browser_create_failed: browser_create_failed.clone(),
            closed: closed.clone(),
        };
        let mut app = DockProbeCefAppBuilder::build(handler);

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

    fn pump(&self) {
        #[cfg(not(target_os = "windows"))]
        do_message_loop_work();
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

impl Drop for CefDockRuntime {
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
                    eprintln!("WV-11-03 CEF Dock display probe: failed to post browser close task");
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
            eprintln!("WV-11-03 CEF Dock display probe: browser close timeout");
        }

        shutdown();
    }
}

struct DockProbeEframeApp {
    dock_state: DockState<DockProbeTab>,
    runtime: CefDockRuntime,
    texture: Option<egui::TextureHandle>,
    applied_generation: u64,
}

impl DockProbeEframeApp {
    fn new(runtime: CefDockRuntime) -> Self {
        Self {
            dock_state: DockState::new(vec![DockProbeTab::Browser]),
            runtime,
            texture: None,
            applied_generation: 0,
        }
    }

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
                    "wv11_03_dock_probe",
                    image,
                    egui::TextureOptions::LINEAR,
                ));
            }
        }
        self.applied_generation = generation;
    }
}

impl eframe::App for DockProbeEframeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.runtime.pump();
        self.update_texture(ctx);

        egui::TopBottomPanel::top("wv11_03_status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("WV-11-03 TEX-IT-SPEC-004");
                ui.separator();
                ui.label("Browser Surface is rendered as an egui Texture; no Browser Native Window.");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut viewer = DockProbeViewer {
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

    println!("WV-11-03 CEF Dock display probe start");

    let runtime = match CefDockRuntime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("WV-11-03 CEF Dock display probe failed: {error}");
            std::process::exit(1);
        }
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("WV-11-03 Browser Surface Dock Probe")
            .with_inner_size([1000.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "WV-11-03 Browser Surface Dock Probe",
        native_options,
        Box::new(move |_cc| Ok(Box::new(DockProbeEframeApp::new(runtime)))),
    )
}
