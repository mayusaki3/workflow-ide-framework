//! WV-11-04 CEF Browser Surface Wheel 転送 Probe。
//!
//! 役割:
//! - INPUT-IT-SPEC-004 の egui Wheel 入力を CEF OSR Browser へ転送する。
//! - スクロール可能な Browser ページの位置変化を Dock 内 Texture で視覚確認する。
//!
//! 注意点:
//! - 本ファイルは技術検証用であり、正式 Surface API ではない。
//! - Wheel は Pointer が Browser Surface 上にある場合のみ転送する。
//! - Windows では CEF の multi-threaded message loop を使用する。
//! - 検証では Wheel 1入力を CEF の標準的な 120 delta 単位へ正規化する。

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
const WHEEL_DELTA_UNIT: i32 = 120;
const TEST_URL: &str = "data:text/html,<html><body style='margin:0;background:rgb(24,34,48);color:white;font-family:sans-serif'><div style='position:fixed;z-index:10;top:0;left:0;right:0;background:rgb(15,22,32);padding:18px;text-align:center;border-bottom:4px solid white'><div style='font-size:30px;font-weight:bold'>WV-11-04 WHEEL</div><div id='status' style='font-size:22px;margin-top:8px'>scrollY: 0</div></div><div style='padding-top:110px'><section style='height:520px;background:rgb(45,85,130);display:flex;align-items:center;justify-content:center;font-size:52px;font-weight:bold'>SECTION 1</section><section style='height:520px;background:rgb(120,70,145);display:flex;align-items:center;justify-content:center;font-size:52px;font-weight:bold'>SECTION 2</section><section style='height:520px;background:rgb(35,135,95);display:flex;align-items:center;justify-content:center;font-size:52px;font-weight:bold'>SECTION 3</section><section style='height:520px;background:rgb(165,90,45);display:flex;align-items:center;justify-content:center;font-size:52px;font-weight:bold'>SECTION 4</section></div><script>window.addEventListener('scroll',function(){document.getElementById('status').textContent='scrollY: '+Math.round(window.scrollY);});</script></body></html>";

/// 最新 CEF Paint を保持する共有状態。
///
/// @hldocs.ref doc-20260912-010000Z-WV14#sec_a6t1n9w4k3rb
#[derive(Debug, Default)]
struct InputWheelState {
    width: i32,
    height: i32,
    rgba: Vec<u8>,
    generation: u64,
}

#[derive(Clone)]
struct InputWheelRenderHandler {
    state: Arc<Mutex<InputWheelState>>,
}

wrap_render_handler! {
    struct InputWheelRenderHandlerBuilder {
        handler: InputWheelRenderHandler,
    }

    impl RenderHandler {
        fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut Rect>) {
            if let Some(rect) = rect {
                rect.width = VIEW_WIDTH;
                rect.height = VIEW_HEIGHT;
            }
        }

        /// CEF BGRA Paint を RGBA8 に変換して最新フレームとして保持する。
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

impl InputWheelRenderHandlerBuilder {
    fn build(state: Arc<Mutex<InputWheelState>>) -> RenderHandler {
        Self::new(InputWheelRenderHandler { state })
    }
}

#[derive(Clone)]
struct InputWheelLifeSpanHandler {
    browser: Arc<Mutex<Option<Browser>>>,
    browser_created: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
}

wrap_life_span_handler! {
    struct InputWheelLifeSpanHandlerBuilder {
        handler: InputWheelLifeSpanHandler,
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

impl InputWheelLifeSpanHandlerBuilder {
    fn build(
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
    ) -> LifeSpanHandler {
        Self::new(InputWheelLifeSpanHandler {
            browser,
            browser_created,
            closed,
        })
    }
}

wrap_client! {
    struct InputWheelClientBuilder {
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

impl InputWheelClientBuilder {
    fn build(
        state: Arc<Mutex<InputWheelState>>,
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
    ) -> Client {
        Self::new(
            InputWheelRenderHandlerBuilder::build(state),
            InputWheelLifeSpanHandlerBuilder::build(browser, browser_created, closed),
        )
    }
}

#[derive(Clone)]
struct InputWheelBrowserProcessHandler {
    state: Arc<Mutex<InputWheelState>>,
    browser: Arc<Mutex<Option<Browser>>>,
    browser_created: Arc<AtomicBool>,
    browser_create_failed: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
}

wrap_browser_process_handler! {
    struct InputWheelBrowserProcessHandlerBuilder {
        handler: InputWheelBrowserProcessHandler,
    }

    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            let mut client = InputWheelClientBuilder::build(
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

impl InputWheelBrowserProcessHandlerBuilder {
    fn build(handler: InputWheelBrowserProcessHandler) -> BrowserProcessHandler {
        Self::new(handler)
    }
}

#[derive(Clone)]
struct InputWheelCefApp {
    browser_process_handler: BrowserProcessHandler,
}

wrap_app! {
    struct InputWheelCefAppBuilder {
        handler: InputWheelCefApp,
    }

    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(self.handler.browser_process_handler.clone())
        }
    }
}

impl InputWheelCefAppBuilder {
    fn build(handler: InputWheelBrowserProcessHandler) -> App {
        Self::new(InputWheelCefApp {
            browser_process_handler: InputWheelBrowserProcessHandlerBuilder::build(handler),
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
        eprintln!("WV-11-04 wheel subprocess failed: cef_execute_process returned {exit_code}");
        1
    }
}

/// Dock 表示座標を Browser OSR 座標へ変換する。
fn map_pointer_to_browser(rect: egui::Rect, position: egui::Pos2) -> Option<(i32, i32)> {
    if !rect.contains(position) || rect.width() <= 0.0 || rect.height() <= 0.0 {
        return None;
    }
    let x = ((position.x - rect.left()) / rect.width() * VIEW_WIDTH as f32)
        .clamp(0.0, VIEW_WIDTH as f32 - 1.0)
        .round() as i32;
    let y = ((position.y - rect.top()) / rect.height() * VIEW_HEIGHT as f32)
        .clamp(0.0, VIEW_HEIGHT as f32 - 1.0)
        .round() as i32;
    Some((x, y))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PointerTransfer {
    x: i32,
    y: i32,
    leave: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WheelTransfer {
    x: i32,
    y: i32,
    delta_x: i32,
    delta_y: i32,
}

#[derive(Clone)]
enum InputWheelTab {
    Browser,
}

struct InputWheelViewer<'a> {
    texture: Option<&'a egui::TextureHandle>,
    pointer_transfer: &'a mut Option<PointerTransfer>,
    wheel_transfer: &'a mut Option<WheelTransfer>,
}

impl<'a> TabViewer for InputWheelViewer<'a> {
    type Tab = InputWheelTab;

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn title(&mut self, _tab: &mut Self::Tab) -> egui::WidgetText {
        "Browser Surface Wheel".into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, _tab: &mut Self::Tab) {
        let available = ui.available_size();
        let Some(texture) = self.texture else {
            ui.centered_and_justified(|ui| {
                ui.label("Waiting for CEF OSR Paint...");
            });
            *self.pointer_transfer = None;
            *self.wheel_transfer = None;
            return;
        };

        let response = ui.add(
            egui::Image::new(texture)
                .fit_to_exact_size(available)
                .sense(egui::Sense::hover()),
        );

        if let Some(pointer) = response.hover_pos() {
            if let Some((x, y)) = map_pointer_to_browser(response.rect, pointer) {
                *self.pointer_transfer = Some(PointerTransfer { x, y, leave: false });

                let scroll = ui.input(|i| i.raw_scroll_delta);
                if scroll.x != 0.0 || scroll.y != 0.0 {
                    let delta_x = if scroll.x > 0.0 {
                        WHEEL_DELTA_UNIT
                    } else if scroll.x < 0.0 {
                        -WHEEL_DELTA_UNIT
                    } else {
                        0
                    };
                    let delta_y = if scroll.y > 0.0 {
                        WHEEL_DELTA_UNIT
                    } else if scroll.y < 0.0 {
                        -WHEEL_DELTA_UNIT
                    } else {
                        0
                    };
                    *self.wheel_transfer = Some(WheelTransfer {
                        x,
                        y,
                        delta_x,
                        delta_y,
                    });
                }
                return;
            }
        }

        *self.pointer_transfer = Some(PointerTransfer {
            x: 0,
            y: 0,
            leave: true,
        });
    }
}

struct InputWheelRuntime {
    state: Arc<Mutex<InputWheelState>>,
    browser: Arc<Mutex<Option<Browser>>>,
    closed: Arc<AtomicBool>,
}

impl InputWheelRuntime {
    fn new() -> Result<Self, String> {
        let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
        let args = args::Args::new();
        let process_result = execute_process(Some(args.as_main_args()), None, ptr::null_mut());
        if process_result >= 0 {
            return Err(format!("Unexpected subprocess result in browser process: {process_result}"));
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

        let state = Arc::new(Mutex::new(InputWheelState::default()));
        let browser = Arc::new(Mutex::new(None));
        let browser_created = Arc::new(AtomicBool::new(false));
        let browser_create_failed = Arc::new(AtomicBool::new(false));
        let closed = Arc::new(AtomicBool::new(false));

        let handler = InputWheelBrowserProcessHandler {
            state: state.clone(),
            browser: browser.clone(),
            browser_created: browser_created.clone(),
            browser_create_failed: browser_create_failed.clone(),
            closed: closed.clone(),
        };
        let mut app = InputWheelCefAppBuilder::build(handler);

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

        Ok(Self { state, browser, closed })
    }

    fn pump(&self) {
        #[cfg(not(target_os = "windows"))]
        do_message_loop_work();
    }

    fn request_pointer_move(&self, transfer: PointerTransfer) -> bool {
        let Some(browser) = self
            .browser
            .lock()
            .ok()
            .and_then(|slot| slot.as_ref().cloned())
        else {
            return false;
        };

        #[cfg(target_os = "windows")]
        {
            let mut task = PointerMoveTaskBuilder::build(browser, transfer);
            return post_task(ThreadId::UI, Some(&mut task)) == 1;
        }

        #[cfg(not(target_os = "windows"))]
        {
            send_pointer_move(&browser, transfer);
            true
        }
    }

    /// Wheel 入力を CEF Browser へ転送する。
    ///
    /// # 引数
    /// - `transfer`: Browser 座標と Wheel delta。
    ///
    /// # 戻り値
    /// - CEF UI thread への送信要求を受理できた場合 `true`。
    fn request_wheel(&self, transfer: WheelTransfer) -> bool {
        let Some(browser) = self
            .browser
            .lock()
            .ok()
            .and_then(|slot| slot.as_ref().cloned())
        else {
            return false;
        };

        #[cfg(target_os = "windows")]
        {
            let mut task = WheelTaskBuilder::build(browser, transfer);
            return post_task(ThreadId::UI, Some(&mut task)) == 1;
        }

        #[cfg(not(target_os = "windows"))]
        {
            send_wheel(&browser, transfer);
            true
        }
    }
}

fn send_pointer_move(browser: &Browser, transfer: PointerTransfer) {
    let Some(host) = browser.host() else {
        return;
    };
    let event = MouseEvent {
        x: transfer.x,
        y: transfer.y,
        ..Default::default()
    };
    host.send_mouse_move_event(Some(&event), transfer.leave.into());
}

/// CEF BrowserHost へ Wheel イベントを送信する。
///
/// @hldocs.ref doc-20260912-010000Z-WV14#sec_a6t1n9w4k3rb
fn send_wheel(browser: &Browser, transfer: WheelTransfer) {
    let Some(host) = browser.host() else {
        return;
    };
    let event = MouseEvent {
        x: transfer.x,
        y: transfer.y,
        ..Default::default()
    };
    host.send_mouse_wheel_event(Some(&event), transfer.delta_x, transfer.delta_y);
    host.invalidate(PaintElementType::VIEW);
    println!(
        "CEF wheel sent: x={}, y={}, delta_x={}, delta_y={}",
        transfer.x, transfer.y, transfer.delta_x, transfer.delta_y
    );
}

#[derive(Clone)]
struct PointerMoveTask {
    browser: Browser,
    transfer: PointerTransfer,
}

wrap_task! {
    struct PointerMoveTaskBuilder {
        task: PointerMoveTask,
    }

    impl Task {
        fn execute(&self) {
            send_pointer_move(&self.task.browser, self.task.transfer);
        }
    }
}

impl PointerMoveTaskBuilder {
    fn build(browser: Browser, transfer: PointerTransfer) -> Task {
        Self::new(PointerMoveTask { browser, transfer })
    }
}

#[derive(Clone)]
struct WheelTask {
    browser: Browser,
    transfer: WheelTransfer,
}

wrap_task! {
    struct WheelTaskBuilder {
        task: WheelTask,
    }

    impl Task {
        fn execute(&self) {
            send_wheel(&self.task.browser, self.task.transfer);
        }
    }
}

impl WheelTaskBuilder {
    fn build(browser: Browser, transfer: WheelTransfer) -> Task {
        Self::new(WheelTask { browser, transfer })
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

impl Drop for InputWheelRuntime {
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
                    eprintln!("WV-11-04 wheel probe: failed to post browser close task");
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
            eprintln!("WV-11-04 wheel probe: browser close timeout");
        }
        shutdown();
    }
}

struct InputWheelEframeApp {
    dock_state: DockState<InputWheelTab>,
    runtime: InputWheelRuntime,
    texture: Option<egui::TextureHandle>,
    applied_generation: u64,
    current_pointer: Option<PointerTransfer>,
    last_pointer: Option<PointerTransfer>,
    current_wheel: Option<WheelTransfer>,
    last_wheel_status: String,
}

impl InputWheelEframeApp {
    fn new(runtime: InputWheelRuntime) -> Self {
        Self {
            dock_state: DockState::new(vec![InputWheelTab::Browser]),
            runtime,
            texture: None,
            applied_generation: 0,
            current_pointer: None,
            last_pointer: None,
            current_wheel: None,
            last_wheel_status: "waiting".to_string(),
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
                    "wv11_04_wheel_probe",
                    image,
                    egui::TextureOptions::LINEAR,
                ));
            }
        }
        self.applied_generation = generation;
    }

    fn send_inputs(&mut self) {
        if let Some(pointer) = self.current_pointer {
            if self.last_pointer != Some(pointer) && self.runtime.request_pointer_move(pointer) {
                self.last_pointer = Some(pointer);
            }
        }
        if let Some(wheel) = self.current_wheel {
            if self.runtime.request_wheel(wheel) {
                self.last_wheel_status = format!(
                    "dx={}, dy={} at {},{}",
                    wheel.delta_x, wheel.delta_y, wheel.x, wheel.y
                );
            }
        }
    }
}

impl eframe::App for InputWheelEframeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.runtime.pump();
        self.update_texture(ctx);

        let generation = self
            .runtime
            .state
            .lock()
            .map(|state| state.generation)
            .unwrap_or(0);

        egui::TopBottomPanel::top("wv11_04_wheel_status").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("WV-11-04 INPUT-IT-SPEC-004");
                ui.separator();
                ui.label(format!("Paint generation: {generation}"));
                ui.separator();
                ui.label(format!("Last wheel: {}", self.last_wheel_status));
            });
        });

        self.current_pointer = None;
        self.current_wheel = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut viewer = InputWheelViewer {
                texture: self.texture.as_ref(),
                pointer_transfer: &mut self.current_pointer,
                wheel_transfer: &mut self.current_wheel,
            };
            DockArea::new(&mut self.dock_state).show_inside(ui, &mut viewer);
        });
        self.send_inputs();

        ctx.request_repaint_after(MESSAGE_PUMP_INTERVAL);
    }
}

fn main() -> eframe::Result<()> {
    let cli_args: Vec<String> = std::env::args().collect();
    if is_cef_subprocess(&cli_args) {
        std::process::exit(run_subprocess());
    }

    println!("WV-11-04 CEF wheel probe start");
    let runtime = match InputWheelRuntime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("WV-11-04 CEF wheel probe failed: {error}");
            std::process::exit(1);
        }
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("WV-11-04 Browser Surface Wheel Probe")
            .with_inner_size([1000.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "WV-11-04 Browser Surface Wheel Probe",
        native_options,
        Box::new(move |_cc| Ok(Box::new(InputWheelEframeApp::new(runtime)))),
    )
}
