//! WV-11-04 CEF Browser Surface Pointer Button 転送 Probe。
//!
//! 役割:
//! - INPUT-IT-SPEC-003 の Pointer Button 押下・解放を CEF OSR Browser へ転送する。
//! - Browser 側の clickable 要素で click 発火を視覚確認する。
//!
//! 注意点:
//! - 本ファイルは技術検証用であり、正式 Surface API ではない。
//! - 左ボタンのみを対象とする。右・中ボタンは本検証対象外。
//! - Windows では CEF の multi-threaded message loop を使用する。
//! - Pointer 座標は INPUT-IT-SPEC-001/002 と同じ 800x600 OSR 座標へ変換する。

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
const TEST_URL: &str = "data:text/html,<html><body style='margin:0;background:rgb(28,38,52);color:white;font-family:sans-serif;height:100vh;overflow:hidden;display:flex;align-items:center;justify-content:center'><div style='text-align:center'><h1>WV-11-04 POINTER BUTTON</h1><div id='status' style='font-size:30px;margin:24px'>Click the button below</div><button id='probe' style='width:360px;height:180px;font-size:42px;font-weight:bold;background:rgb(65,95,130);color:white;border:8px solid white'>CLICK ME</button></div><script>let n=0;let b=document.getElementById('probe');b.addEventListener('mousedown',function(){document.getElementById('status').textContent='MOUSE DOWN';b.style.backgroundColor='rgb(220,150,0)';});b.addEventListener('mouseup',function(){document.getElementById('status').textContent='MOUSE UP';b.style.backgroundColor='rgb(80,140,190)';});b.addEventListener('click',function(){n=n+1;document.getElementById('status').textContent='CLICK OK '+n;b.textContent='CLICKED '+n;b.style.backgroundColor='rgb(30,170,80)';});</script></body></html>";

/// 最新 CEF Paint を保持する共有状態。
///
/// @hldocs.ref doc-20260912-010000Z-WV14#sec_p2m8x5c1q7vz
#[derive(Debug, Default)]
struct InputButtonState {
    width: i32,
    height: i32,
    rgba: Vec<u8>,
    generation: u64,
}

#[derive(Clone)]
struct InputButtonRenderHandler {
    state: Arc<Mutex<InputButtonState>>,
}

wrap_render_handler! {
    struct InputButtonRenderHandlerBuilder {
        handler: InputButtonRenderHandler,
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

impl InputButtonRenderHandlerBuilder {
    fn build(state: Arc<Mutex<InputButtonState>>) -> RenderHandler {
        Self::new(InputButtonRenderHandler { state })
    }
}

#[derive(Clone)]
struct InputButtonLifeSpanHandler {
    browser: Arc<Mutex<Option<Browser>>>,
    browser_created: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
}

wrap_life_span_handler! {
    struct InputButtonLifeSpanHandlerBuilder {
        handler: InputButtonLifeSpanHandler,
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

impl InputButtonLifeSpanHandlerBuilder {
    fn build(
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
    ) -> LifeSpanHandler {
        Self::new(InputButtonLifeSpanHandler {
            browser,
            browser_created,
            closed,
        })
    }
}

wrap_client! {
    struct InputButtonClientBuilder {
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

impl InputButtonClientBuilder {
    fn build(
        state: Arc<Mutex<InputButtonState>>,
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
    ) -> Client {
        Self::new(
            InputButtonRenderHandlerBuilder::build(state),
            InputButtonLifeSpanHandlerBuilder::build(browser, browser_created, closed),
        )
    }
}

#[derive(Clone)]
struct InputButtonBrowserProcessHandler {
    state: Arc<Mutex<InputButtonState>>,
    browser: Arc<Mutex<Option<Browser>>>,
    browser_created: Arc<AtomicBool>,
    browser_create_failed: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
}

wrap_browser_process_handler! {
    struct InputButtonBrowserProcessHandlerBuilder {
        handler: InputButtonBrowserProcessHandler,
    }

    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            let mut client = InputButtonClientBuilder::build(
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

impl InputButtonBrowserProcessHandlerBuilder {
    fn build(handler: InputButtonBrowserProcessHandler) -> BrowserProcessHandler {
        Self::new(handler)
    }
}

#[derive(Clone)]
struct InputButtonCefApp {
    browser_process_handler: BrowserProcessHandler,
}

wrap_app! {
    struct InputButtonCefAppBuilder {
        handler: InputButtonCefApp,
    }

    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(self.handler.browser_process_handler.clone())
        }
    }
}

impl InputButtonCefAppBuilder {
    fn build(handler: InputButtonBrowserProcessHandler) -> App {
        Self::new(InputButtonCefApp {
            browser_process_handler: InputButtonBrowserProcessHandlerBuilder::build(handler),
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
        eprintln!("WV-11-04 pointer button subprocess failed: cef_execute_process returned {exit_code}");
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
struct ButtonTransfer {
    x: i32,
    y: i32,
    mouse_up: bool,
}

#[derive(Clone)]
enum InputButtonTab {
    Browser,
}

struct InputButtonViewer<'a> {
    texture: Option<&'a egui::TextureHandle>,
    pointer_transfer: &'a mut Option<PointerTransfer>,
    button_transfer: &'a mut Option<ButtonTransfer>,
}

impl<'a> TabViewer for InputButtonViewer<'a> {
    type Tab = InputButtonTab;

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn title(&mut self, _tab: &mut Self::Tab) -> egui::WidgetText {
        "Browser Surface Pointer Button".into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, _tab: &mut Self::Tab) {
        let available = ui.available_size();
        let Some(texture) = self.texture else {
            ui.centered_and_justified(|ui| {
                ui.label("Waiting for CEF OSR Paint...");
            });
            *self.pointer_transfer = None;
            *self.button_transfer = None;
            return;
        };

        let response = ui.add(
            egui::Image::new(texture)
                .fit_to_exact_size(available)
                .sense(egui::Sense::click_and_drag()),
        );

        if let Some(pointer) = response.hover_pos() {
            if let Some((x, y)) = map_pointer_to_browser(response.rect, pointer) {
                *self.pointer_transfer = Some(PointerTransfer { x, y, leave: false });

                let pressed = ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary));
                let released = ui.input(|i| i.pointer.button_released(egui::PointerButton::Primary));
                if pressed {
                    *self.button_transfer = Some(ButtonTransfer {
                        x,
                        y,
                        mouse_up: false,
                    });
                } else if released {
                    *self.button_transfer = Some(ButtonTransfer {
                        x,
                        y,
                        mouse_up: true,
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

struct InputButtonRuntime {
    state: Arc<Mutex<InputButtonState>>,
    browser: Arc<Mutex<Option<Browser>>>,
    closed: Arc<AtomicBool>,
}

impl InputButtonRuntime {
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

        let state = Arc::new(Mutex::new(InputButtonState::default()));
        let browser = Arc::new(Mutex::new(None));
        let browser_created = Arc::new(AtomicBool::new(false));
        let browser_create_failed = Arc::new(AtomicBool::new(false));
        let closed = Arc::new(AtomicBool::new(false));

        let handler = InputButtonBrowserProcessHandler {
            state: state.clone(),
            browser: browser.clone(),
            browser_created: browser_created.clone(),
            browser_create_failed: browser_create_failed.clone(),
            closed: closed.clone(),
        };
        let mut app = InputButtonCefAppBuilder::build(handler);

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

    /// 左 Pointer Button の押下・解放を CEF Browser へ転送する。
    ///
    /// # 引数
    /// - `transfer`: Browser 座標と mouse-up 状態。
    ///
    /// # 戻り値
    /// - CEF UI thread への送信要求を受理できた場合 `true`。
    fn request_button(&self, transfer: ButtonTransfer) -> bool {
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
            let mut task = PointerButtonTaskBuilder::build(browser, transfer);
            return post_task(ThreadId::UI, Some(&mut task)) == 1;
        }

        #[cfg(not(target_os = "windows"))]
        {
            send_pointer_button(&browser, transfer);
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

/// CEF BrowserHost へ左 Pointer Button イベントを送信する。
///
/// @hldocs.ref doc-20260912-010000Z-WV14#sec_p2m8x5c1q7vz
fn send_pointer_button(browser: &Browser, transfer: ButtonTransfer) {
    let Some(host) = browser.host() else {
        return;
    };
    let event = MouseEvent {
        x: transfer.x,
        y: transfer.y,
        ..Default::default()
    };
    host.send_mouse_click_event(
        Some(&event),
        MouseButtonType::LEFT,
        transfer.mouse_up.into(),
        1,
    );
    println!(
        "CEF pointer button sent: {} x={}, y={}",
        if transfer.mouse_up { "UP" } else { "DOWN" },
        transfer.x,
        transfer.y
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
struct PointerButtonTask {
    browser: Browser,
    transfer: ButtonTransfer,
}

wrap_task! {
    struct PointerButtonTaskBuilder {
        task: PointerButtonTask,
    }

    impl Task {
        fn execute(&self) {
            send_pointer_button(&self.task.browser, self.task.transfer);
        }
    }
}

impl PointerButtonTaskBuilder {
    fn build(browser: Browser, transfer: ButtonTransfer) -> Task {
        Self::new(PointerButtonTask { browser, transfer })
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

impl Drop for InputButtonRuntime {
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
                    eprintln!("WV-11-04 pointer button probe: failed to post browser close task");
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
            eprintln!("WV-11-04 pointer button probe: browser close timeout");
        }
        shutdown();
    }
}

struct InputButtonEframeApp {
    dock_state: DockState<InputButtonTab>,
    runtime: InputButtonRuntime,
    texture: Option<egui::TextureHandle>,
    applied_generation: u64,
    current_pointer: Option<PointerTransfer>,
    last_pointer: Option<PointerTransfer>,
    current_button: Option<ButtonTransfer>,
    last_button_status: String,
}

impl InputButtonEframeApp {
    fn new(runtime: InputButtonRuntime) -> Self {
        Self {
            dock_state: DockState::new(vec![InputButtonTab::Browser]),
            runtime,
            texture: None,
            applied_generation: 0,
            current_pointer: None,
            last_pointer: None,
            current_button: None,
            last_button_status: "waiting".to_string(),
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
                    "wv11_04_pointer_button_probe",
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
        if let Some(button) = self.current_button {
            if self.runtime.request_button(button) {
                self.last_button_status = format!(
                    "{} at {},{}",
                    if button.mouse_up { "UP" } else { "DOWN" },
                    button.x,
                    button.y
                );
            }
        }
    }
}

impl eframe::App for InputButtonEframeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.runtime.pump();
        self.update_texture(ctx);

        let generation = self
            .runtime
            .state
            .lock()
            .map(|state| state.generation)
            .unwrap_or(0);

        egui::TopBottomPanel::top("wv11_04_pointer_button_status").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("WV-11-04 INPUT-IT-SPEC-003");
                ui.separator();
                ui.label(format!("Paint generation: {generation}"));
                ui.separator();
                ui.label(format!("Last button: {}", self.last_button_status));
            });
        });

        self.current_pointer = None;
        self.current_button = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut viewer = InputButtonViewer {
                texture: self.texture.as_ref(),
                pointer_transfer: &mut self.current_pointer,
                button_transfer: &mut self.current_button,
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

    println!("WV-11-04 CEF pointer button probe start");
    let runtime = match InputButtonRuntime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("WV-11-04 CEF pointer button probe failed: {error}");
            std::process::exit(1);
        }
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("WV-11-04 Browser Surface Pointer Button Probe")
            .with_inner_size([1000.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "WV-11-04 Browser Surface Pointer Button Probe",
        native_options,
        Box::new(move |_cc| Ok(Box::new(InputButtonEframeApp::new(runtime)))),
    )
}
