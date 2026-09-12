//! WV-11-03 CEF Browser Surface Dock Resize Probe。
//!
//! 役割:
//! - TEX-IT-SPEC-005 の Dock サイズ同期を Windows 実機で検証する。
//! - egui_dock の表示領域を CEF OSR の `view_rect` へ反映し、`was_resized` 後の Paint と
//!   egui Texture サイズが追従することを確認する。
//!
//! 注意点:
//! - 本ファイルは技術検証用であり、正式 Surface API ではない。
//! - egui の表示サイズは論理座標として CEF `view_rect` へ渡す。
//! - DPI は `RenderHandler::screen_info` の `device_scale_factor` で CEF へ通知する。
//! - Windows では CEF の multi-threaded message loop を使用する。

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

const INITIAL_VIEW_WIDTH: i32 = 800;
const INITIAL_VIEW_HEIGHT: i32 = 600;
const MESSAGE_PUMP_INTERVAL: Duration = Duration::from_millis(10);
const BROWSER_CREATE_TIMEOUT: Duration = Duration::from_secs(3);
const CLOSE_TIMEOUT: Duration = Duration::from_secs(3);
const TEST_URL: &str = "data:text/html,<html><body style='margin:0;background:rgb(0,190,220);color:rgb(10,25,35);font-family:sans-serif;display:flex;align-items:center;justify-content:center;height:100vh;overflow:hidden'><div style='box-sizing:border-box;border:12px solid rgb(10,25,35);padding:36px;text-align:center;width:82vw'><h1 style='font-size:48px;margin:0 0 18px'>WV-11-03 RESIZE OK</h1><p style='font-size:24px;margin:0'>Resize the application window</p></div></body></html>";

/// CEF Paint の最新フレーム。
///
/// @hldocs.ref doc-20260911-120000Z-WV13#sec_b4j7k1s9d3wx
#[derive(Debug, Default)]
struct PaintState {
    width: i32,
    height: i32,
    rgba: Vec<u8>,
    generation: u64,
}

/// CEF OSR に要求する論理表示サイズと DPI スケール。
#[derive(Debug, Clone, Copy)]
struct ViewMetrics {
    width: i32,
    height: i32,
    device_scale_factor: f32,
}

impl Default for ViewMetrics {
    fn default() -> Self {
        Self {
            width: INITIAL_VIEW_WIDTH,
            height: INITIAL_VIEW_HEIGHT,
            device_scale_factor: 1.0,
        }
    }
}

#[derive(Clone)]
struct ResizeRenderHandler {
    paint_state: Arc<Mutex<PaintState>>,
    view_metrics: Arc<Mutex<ViewMetrics>>,
}

wrap_render_handler! {
    struct ResizeRenderHandlerBuilder {
        handler: ResizeRenderHandler,
    }

    impl RenderHandler {
        /// CEF へ現在の Dock 論理表示サイズを返す。
        fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut Rect>) {
            let Some(rect) = rect else {
                return;
            };
            let Ok(metrics) = self.handler.view_metrics.lock() else {
                return;
            };
            rect.width = metrics.width.max(1);
            rect.height = metrics.height.max(1);
        }

        /// CEF へ現在の DPI スケールを返す。
        fn screen_info(
            &self,
            _browser: Option<&mut Browser>,
            screen_info: Option<&mut ScreenInfo>,
        ) -> ::std::os::raw::c_int {
            let Some(screen_info) = screen_info else {
                return 0;
            };
            let Ok(metrics) = self.handler.view_metrics.lock() else {
                return 0;
            };
            screen_info.device_scale_factor = metrics.device_scale_factor.max(0.1);
            1
        }

        /// CEF BGRA Paint を RGBA8 に変換し、最新フレームとして保持する。
        ///
        /// @hldocs.ref doc-20260911-120000Z-WV13#sec_b4j7k1s9d3wx
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

            let Ok(mut state) = self.handler.paint_state.lock() else {
                return;
            };
            state.width = width;
            state.height = height;
            state.rgba = rgba;
            state.generation = state.generation.saturating_add(1);

            println!(
                "CEF OSR Paint: generation={}, size={}x{}",
                state.generation, state.width, state.height
            );
        }
    }
}

impl ResizeRenderHandlerBuilder {
    /// 共有 Paint 状態と表示サイズを持つ RenderHandler を生成する。
    ///
    /// # 引数
    /// - `paint_state`: 最新 Paint の共有状態。
    /// - `view_metrics`: CEF へ通知する論理表示サイズと DPI。
    fn build(
        paint_state: Arc<Mutex<PaintState>>,
        view_metrics: Arc<Mutex<ViewMetrics>>,
    ) -> RenderHandler {
        Self::new(ResizeRenderHandler {
            paint_state,
            view_metrics,
        })
    }
}

#[derive(Clone)]
struct ResizeLifeSpanHandler {
    browser: Arc<Mutex<Option<Browser>>>,
    browser_created: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
}

wrap_life_span_handler! {
    struct ResizeLifeSpanHandlerBuilder {
        handler: ResizeLifeSpanHandler,
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

impl ResizeLifeSpanHandlerBuilder {
    fn build(
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
    ) -> LifeSpanHandler {
        Self::new(ResizeLifeSpanHandler {
            browser,
            browser_created,
            closed,
        })
    }
}

wrap_client! {
    struct ResizeClientBuilder {
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

impl ResizeClientBuilder {
    fn build(
        paint_state: Arc<Mutex<PaintState>>,
        view_metrics: Arc<Mutex<ViewMetrics>>,
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
    ) -> Client {
        Self::new(
            ResizeRenderHandlerBuilder::build(paint_state, view_metrics),
            ResizeLifeSpanHandlerBuilder::build(browser, browser_created, closed),
        )
    }
}

#[derive(Clone)]
struct ResizeBrowserProcessHandler {
    paint_state: Arc<Mutex<PaintState>>,
    view_metrics: Arc<Mutex<ViewMetrics>>,
    browser: Arc<Mutex<Option<Browser>>>,
    browser_created: Arc<AtomicBool>,
    browser_create_failed: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
}

wrap_browser_process_handler! {
    struct ResizeBrowserProcessHandlerBuilder {
        handler: ResizeBrowserProcessHandler,
    }

    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            let mut client = ResizeClientBuilder::build(
                self.handler.paint_state.clone(),
                self.handler.view_metrics.clone(),
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

impl ResizeBrowserProcessHandlerBuilder {
    fn build(handler: ResizeBrowserProcessHandler) -> BrowserProcessHandler {
        Self::new(handler)
    }
}

#[derive(Clone)]
struct ResizeCefApp {
    browser_process_handler: BrowserProcessHandler,
}

wrap_app! {
    struct ResizeCefAppBuilder {
        handler: ResizeCefApp,
    }

    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(self.handler.browser_process_handler.clone())
        }
    }
}

impl ResizeCefAppBuilder {
    fn build(handler: ResizeBrowserProcessHandler) -> App {
        Self::new(ResizeCefApp {
            browser_process_handler: ResizeBrowserProcessHandlerBuilder::build(handler),
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
        eprintln!("WV-11-03 resize subprocess failed: cef_execute_process returned {exit_code}");
        1
    }
}

#[derive(Clone)]
enum ResizeProbeTab {
    Browser,
}

struct ResizeProbeViewer<'a> {
    texture: Option<&'a egui::TextureHandle>,
    requested_view_size: &'a mut Option<(i32, i32)>,
}

impl<'a> TabViewer for ResizeProbeViewer<'a> {
    type Tab = ResizeProbeTab;

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn title(&mut self, _tab: &mut Self::Tab) -> egui::WidgetText {
        "Browser Surface Resize".into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, _tab: &mut Self::Tab) {
        let available = ui.available_size();
        let width = available.x.round().max(1.0) as i32;
        let height = available.y.round().max(1.0) as i32;
        *self.requested_view_size = Some((width, height));

        if let Some(texture) = self.texture {
            ui.image((texture.id(), available));
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("Waiting for CEF OSR Paint...");
            });
        }
    }
}

struct ResizeRuntime {
    paint_state: Arc<Mutex<PaintState>>,
    view_metrics: Arc<Mutex<ViewMetrics>>,
    browser: Arc<Mutex<Option<Browser>>>,
    closed: Arc<AtomicBool>,
}

impl ResizeRuntime {
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

        let paint_state = Arc::new(Mutex::new(PaintState::default()));
        let view_metrics = Arc::new(Mutex::new(ViewMetrics::default()));
        let browser = Arc::new(Mutex::new(None));
        let browser_created = Arc::new(AtomicBool::new(false));
        let browser_create_failed = Arc::new(AtomicBool::new(false));
        let closed = Arc::new(AtomicBool::new(false));

        let handler = ResizeBrowserProcessHandler {
            paint_state: paint_state.clone(),
            view_metrics: view_metrics.clone(),
            browser: browser.clone(),
            browser_created: browser_created.clone(),
            browser_create_failed: browser_create_failed.clone(),
            closed: closed.clone(),
        };
        let mut app = ResizeCefAppBuilder::build(handler);

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
            paint_state,
            view_metrics,
            browser,
            closed,
        })
    }

    /// 必要なプラットフォームで CEF メッセージを処理する。
    fn pump(&self) {
        #[cfg(not(target_os = "windows"))]
        do_message_loop_work();
    }

    /// Dock の論理表示サイズと DPI を CEF へ反映し、再描画を要求する。
    ///
    /// # 引数
    /// - `width`: Dock 表示領域の論理幅。
    /// - `height`: Dock 表示領域の論理高さ。
    /// - `device_scale_factor`: egui の pixels_per_point。
    ///
    /// # 戻り値
    /// - 設定値が変化し、CEF に Resize を要求した場合は `true`。
    fn request_resize(&self, width: i32, height: i32, device_scale_factor: f32) -> bool {
        let width = width.max(1);
        let height = height.max(1);
        let device_scale_factor = device_scale_factor.max(0.1);

        let changed = {
            let Ok(mut metrics) = self.view_metrics.lock() else {
                return false;
            };
            let changed = metrics.width != width
                || metrics.height != height
                || (metrics.device_scale_factor - device_scale_factor).abs() > 0.001;
            if changed {
                metrics.width = width;
                metrics.height = height;
                metrics.device_scale_factor = device_scale_factor;
            }
            changed
        };

        if !changed {
            return false;
        }

        let browser = self
            .browser
            .lock()
            .ok()
            .and_then(|slot| slot.as_ref().cloned());
        let Some(browser) = browser else {
            return false;
        };

        println!(
            "CEF OSR resize requested: view={}x{}, scale={:.3}",
            width, height, device_scale_factor
        );

        #[cfg(target_os = "windows")]
        {
            let mut task = ResizeBrowserTaskBuilder::build(browser);
            return post_task(ThreadId::UI, Some(&mut task)) == 1;
        }

        #[cfg(not(target_os = "windows"))]
        {
            if let Some(host) = browser.host() {
                host.was_resized();
                return true;
            }
            false
        }
    }

    /// 診断表示用に要求サイズと Paint 状態を取得する。
    fn diagnostics(&self) -> (ViewMetrics, u64, i32, i32, usize) {
        let metrics = self
            .view_metrics
            .lock()
            .map(|value| *value)
            .unwrap_or_default();
        let Ok(state) = self.paint_state.lock() else {
            return (metrics, 0, 0, 0, 0);
        };
        (
            metrics,
            state.generation,
            state.width,
            state.height,
            state.rgba.len(),
        )
    }
}

#[derive(Clone)]
struct ResizeBrowserTask {
    browser: Browser,
}

wrap_task! {
    struct ResizeBrowserTaskBuilder {
        task: ResizeBrowserTask,
    }

    impl Task {
        fn execute(&self) {
            if let Some(host) = self.task.browser.host() {
                host.was_resized();
            }
        }
    }
}

impl ResizeBrowserTaskBuilder {
    fn build(browser: Browser) -> Task {
        Self::new(ResizeBrowserTask { browser })
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

impl Drop for ResizeRuntime {
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
                    eprintln!("WV-11-03 resize probe: failed to post browser close task");
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
            eprintln!("WV-11-03 resize probe: browser close timeout");
        }

        shutdown();
    }
}

struct ResizeEframeApp {
    dock_state: DockState<ResizeProbeTab>,
    runtime: ResizeRuntime,
    texture: Option<egui::TextureHandle>,
    applied_generation: u64,
}

impl ResizeEframeApp {
    fn new(runtime: ResizeRuntime) -> Self {
        Self {
            dock_state: DockState::new(vec![ResizeProbeTab::Browser]),
            runtime,
            texture: None,
            applied_generation: 0,
        }
    }

    /// 最新 CEF Paint を egui Texture へ反映する。
    fn update_texture(&mut self, ctx: &egui::Context) {
        let snapshot = {
            let Ok(state) = self.runtime.paint_state.lock() else {
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
                    "wv11_03_resize_probe",
                    image,
                    egui::TextureOptions::LINEAR,
                ));
            }
        }
        self.applied_generation = generation;
    }
}

impl eframe::App for ResizeEframeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.runtime.pump();
        self.update_texture(ctx);

        let (metrics, generation, paint_width, paint_height, rgba_bytes) =
            self.runtime.diagnostics();
        let texture_size = self
            .texture
            .as_ref()
            .map(|texture| texture.size())
            .unwrap_or([0, 0]);

        egui::TopBottomPanel::top("wv11_03_resize_status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("WV-11-03 TEX-IT-SPEC-005");
                ui.separator();
                ui.label(format!("Requested: {}x{}", metrics.width, metrics.height));
                ui.separator();
                ui.label(format!("Paint: {}x{}", paint_width, paint_height));
                ui.separator();
                ui.label(format!("Texture: {}x{}", texture_size[0], texture_size[1]));
                ui.separator();
                ui.label(format!("Generation: {generation}"));
                ui.separator();
                ui.label(format!("Scale: {:.2}", metrics.device_scale_factor));
                ui.separator();
                ui.label(format!("RGBA: {rgba_bytes}"));
            });
        });

        let mut requested_view_size = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut viewer = ResizeProbeViewer {
                texture: self.texture.as_ref(),
                requested_view_size: &mut requested_view_size,
            };
            DockArea::new(&mut self.dock_state).show_inside(ui, &mut viewer);
        });

        if let Some((width, height)) = requested_view_size {
            self.runtime
                .request_resize(width, height, ctx.pixels_per_point());
        }

        ctx.request_repaint_after(MESSAGE_PUMP_INTERVAL);
    }
}

fn main() -> eframe::Result<()> {
    let cli_args: Vec<String> = std::env::args().collect();
    if is_cef_subprocess(&cli_args) {
        std::process::exit(run_subprocess());
    }

    println!("WV-11-03 CEF Dock resize probe start");

    let runtime = match ResizeRuntime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("WV-11-03 CEF Dock resize probe failed: {error}");
            std::process::exit(1);
        }
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("WV-11-03 Browser Surface Resize Probe")
            .with_inner_size([1000.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "WV-11-03 Browser Surface Resize Probe",
        native_options,
        Box::new(move |_cc| Ok(Box::new(ResizeEframeApp::new(runtime)))),
    )
}
