//! WV-11-04-01 CEF Browser Surface Pointer Drag 範囲選択 Probe。
//!
//! 役割:
//! - EDIT-IT-SPEC-002 の Pointer Drag による文字範囲選択を検証する。
//! - egui 上の Pointer Down / Move / Up を CEF OSR Browser へ転送し、
//!   Browser 内 input 要素の selectionStart / selectionEnd 変化を視覚確認する。
//!
//! 注意点:
//! - 本ファイルは技術検証用であり、正式 Surface API ではない。
//! - 左 Pointer Button の Drag のみを対象とする。
//! - Drag 中の Mouse Move には CEF EVENTFLAG_LEFT_MOUSE_BUTTON を付与する。
//! - Windows では CEF の multi-threaded message loop を使用する。
//! - Pointer 座標は 800x600 の Browser OSR 座標へ変換する。

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
const CEF_EVENTFLAG_LEFT_MOUSE_BUTTON: u32 = 1 << 4;
const TEST_URL: &str = "data:text/html,<html><body style='margin:0;background:rgb(26,36,50);color:white;font-family:sans-serif;height:100vh;display:flex;align-items:center;justify-content:center'><div style='text-align:center;width:92%'><h1 style='font-size:38px'>WV-11-04-01 POINTER DRAG SELECT</h1><div style='font-size:21px;margin:14px'>Drag across characters in the input below.</div><input id='probe' type='text' value='DRAG-SELECT-12345' style='width:84%;height:92px;font-size:38px;padding:12px;border:6px solid white;background:rgb(55,78,105);color:white;box-sizing:border-box'><div id='range' style='font-size:26px;margin-top:22px'>SELECTION: waiting</div><div id='text' style='font-size:24px;margin-top:10px'>TEXT: waiting</div></div><script>let p=document.getElementById('probe');let r=document.getElementById('range');let t=document.getElementById('text');function u(){let s=p.selectionStart??0;let e=p.selectionEnd??0;r.textContent='SELECTION: '+s+' - '+e;t.textContent='TEXT: '+p.value.substring(s,e);}p.addEventListener('focus',u);p.addEventListener('select',u);p.addEventListener('mouseup',u);p.addEventListener('mousemove',function(e){if(e.buttons===1)u();});document.addEventListener('selectionchange',u);</script></body></html>";

/// 最新 CEF Paint を保持する共有状態。
///
/// @hldocs.ref doc-20260912-104800Z-WV15#sec_j7c1w9r4p5tx
#[derive(Debug, Default)]
struct DragSelectState {
    width: i32,
    height: i32,
    rgba: Vec<u8>,
    generation: u64,
}

#[derive(Clone)]
struct DragSelectRenderHandler {
    state: Arc<Mutex<DragSelectState>>,
}

wrap_render_handler! {
    struct DragSelectRenderHandlerBuilder {
        handler: DragSelectRenderHandler,
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

impl DragSelectRenderHandlerBuilder {
    fn build(state: Arc<Mutex<DragSelectState>>) -> RenderHandler {
        Self::new(DragSelectRenderHandler { state })
    }
}

#[derive(Clone)]
struct DragSelectLifeSpanHandler {
    browser: Arc<Mutex<Option<Browser>>>,
    browser_created: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
}

wrap_life_span_handler! {
    struct DragSelectLifeSpanHandlerBuilder {
        handler: DragSelectLifeSpanHandler,
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

impl DragSelectLifeSpanHandlerBuilder {
    fn build(
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
    ) -> LifeSpanHandler {
        Self::new(DragSelectLifeSpanHandler {
            browser,
            browser_created,
            closed,
        })
    }
}

wrap_client! {
    struct DragSelectClientBuilder {
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

impl DragSelectClientBuilder {
    fn build(
        state: Arc<Mutex<DragSelectState>>,
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
    ) -> Client {
        Self::new(
            DragSelectRenderHandlerBuilder::build(state),
            DragSelectLifeSpanHandlerBuilder::build(browser, browser_created, closed),
        )
    }
}

#[derive(Clone)]
struct DragSelectBrowserProcessHandler {
    state: Arc<Mutex<DragSelectState>>,
    browser: Arc<Mutex<Option<Browser>>>,
    browser_created: Arc<AtomicBool>,
    browser_create_failed: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
}

wrap_browser_process_handler! {
    struct DragSelectBrowserProcessHandlerBuilder {
        handler: DragSelectBrowserProcessHandler,
    }

    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            let mut client = DragSelectClientBuilder::build(
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

impl DragSelectBrowserProcessHandlerBuilder {
    fn build(handler: DragSelectBrowserProcessHandler) -> BrowserProcessHandler {
        Self::new(handler)
    }
}

#[derive(Clone)]
struct DragSelectCefApp {
    browser_process_handler: BrowserProcessHandler,
}

wrap_app! {
    struct DragSelectCefAppBuilder {
        handler: DragSelectCefApp,
    }

    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(self.handler.browser_process_handler.clone())
        }
    }
}

impl DragSelectCefAppBuilder {
    fn build(handler: DragSelectBrowserProcessHandler) -> App {
        Self::new(DragSelectCefApp {
            browser_process_handler: DragSelectBrowserProcessHandlerBuilder::build(handler),
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
        eprintln!("WV-11-04-01 drag select subprocess failed: cef_execute_process returned {exit_code}");
        1
    }
}

/// Dock 表示座標を Browser OSR 座標へ変換する。
///
/// # 引数
/// - `rect`: Browser Surface の egui 表示矩形。
/// - `position`: egui 上の Pointer 座標。
///
/// # 戻り値
/// - Surface 内なら Browser OSR 座標、外なら `None`。
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
    dragging: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ButtonTransfer {
    x: i32,
    y: i32,
    mouse_up: bool,
}

#[derive(Clone)]
enum DragSelectTab {
    Browser,
}

struct DragSelectViewer<'a> {
    texture: Option<&'a egui::TextureHandle>,
    pointer_transfer: &'a mut Option<PointerTransfer>,
    button_transfer: &'a mut Option<ButtonTransfer>,
    dragging: &'a mut bool,
}

impl<'a> TabViewer for DragSelectViewer<'a> {
    type Tab = DragSelectTab;

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn title(&mut self, _tab: &mut Self::Tab) -> egui::WidgetText {
        "Browser Surface Pointer Drag Selection".into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, _tab: &mut Self::Tab) {
        let available = ui.available_size();
        let Some(texture) = self.texture else {
            ui.centered_and_justified(|ui| {
                ui.label("Waiting for CEF OSR Paint...");
            });
            return;
        };

        let response = ui.add(
            egui::Image::new(texture)
                .fit_to_exact_size(available)
                .sense(egui::Sense::click_and_drag()),
        );

        let pressed = ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary));
        let released = ui.input(|i| i.pointer.button_released(egui::PointerButton::Primary));

        if let Some(pointer) = response.interact_pointer_pos().or_else(|| response.hover_pos()) {
            if let Some((x, y)) = map_pointer_to_browser(response.rect, pointer) {
                if pressed && response.hovered() {
                    *self.dragging = true;
                    *self.button_transfer = Some(ButtonTransfer {
                        x,
                        y,
                        mouse_up: false,
                    });
                }

                *self.pointer_transfer = Some(PointerTransfer {
                    x,
                    y,
                    leave: false,
                    dragging: *self.dragging,
                });

                if released && *self.dragging {
                    *self.button_transfer = Some(ButtonTransfer {
                        x,
                        y,
                        mouse_up: true,
                    });
                    *self.dragging = false;
                }
                return;
            }
        }

        if released && *self.dragging {
            *self.dragging = false;
        }
        *self.pointer_transfer = Some(PointerTransfer {
            x: 0,
            y: 0,
            leave: true,
            dragging: false,
        });
    }
}

struct DragSelectRuntime {
    state: Arc<Mutex<DragSelectState>>,
    browser: Arc<Mutex<Option<Browser>>>,
    closed: Arc<AtomicBool>,
}

impl DragSelectRuntime {
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

        let state = Arc::new(Mutex::new(DragSelectState::default()));
        let browser = Arc::new(Mutex::new(None));
        let browser_created = Arc::new(AtomicBool::new(false));
        let browser_create_failed = Arc::new(AtomicBool::new(false));
        let closed = Arc::new(AtomicBool::new(false));

        let handler = DragSelectBrowserProcessHandler {
            state: state.clone(),
            browser: browser.clone(),
            browser_created: browser_created.clone(),
            browser_create_failed: browser_create_failed.clone(),
            closed: closed.clone(),
        };
        let mut app = DragSelectCefAppBuilder::build(handler);

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

    /// Pointer Move を CEF Browser へ転送する。
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

/// CEF BrowserHost へ Pointer Move を送信する。
///
/// Drag 中は Left Mouse Button modifier を付与する。
///
/// @hldocs.ref doc-20260912-104800Z-WV15#sec_j7c1w9r4p5tx
fn send_pointer_move(browser: &Browser, transfer: PointerTransfer) {
    let Some(host) = browser.host() else {
        return;
    };
    let event = MouseEvent {
        x: transfer.x,
        y: transfer.y,
        modifiers: if transfer.dragging {
            CEF_EVENTFLAG_LEFT_MOUSE_BUTTON
        } else {
            0
        },
        ..Default::default()
    };
    host.send_mouse_move_event(Some(&event), transfer.leave.into());
    if transfer.dragging && !transfer.leave {
        println!("CEF drag move sent: x={}, y={}", transfer.x, transfer.y);
    }
}

/// CEF BrowserHost へ左 Pointer Button イベントを送信する。
///
/// @hldocs.ref doc-20260912-104800Z-WV15#sec_j7c1w9r4p5tx
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
        "CEF drag button sent: {} x={}, y={}",
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

impl Drop for DragSelectRuntime {
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
                    eprintln!("WV-11-04-01 drag select probe: failed to post browser close task");
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
            eprintln!("WV-11-04-01 drag select probe: browser close timeout");
        }
        shutdown();
    }
}

struct DragSelectEframeApp {
    dock_state: DockState<DragSelectTab>,
    runtime: DragSelectRuntime,
    texture: Option<egui::TextureHandle>,
    applied_generation: u64,
    current_pointer: Option<PointerTransfer>,
    last_pointer: Option<PointerTransfer>,
    current_button: Option<ButtonTransfer>,
    dragging: bool,
    last_status: String,
}

impl DragSelectEframeApp {
    fn new(runtime: DragSelectRuntime) -> Self {
        Self {
            dock_state: DockState::new(vec![DragSelectTab::Browser]),
            runtime,
            texture: None,
            applied_generation: 0,
            current_pointer: None,
            last_pointer: None,
            current_button: None,
            dragging: false,
            last_status: "waiting".to_string(),
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
                    "wv11_04_01_drag_select_probe",
                    image,
                    egui::TextureOptions::LINEAR,
                ));
            }
        }
        self.applied_generation = generation;
    }

    fn send_inputs(&mut self) {
        if let Some(button) = self.current_button {
            if self.runtime.request_button(button) {
                self.last_status = format!(
                    "{} at {},{}",
                    if button.mouse_up { "UP" } else { "DOWN" },
                    button.x,
                    button.y
                );
            }
        }

        if let Some(pointer) = self.current_pointer {
            if self.last_pointer != Some(pointer) && self.runtime.request_pointer_move(pointer) {
                self.last_pointer = Some(pointer);
                if pointer.dragging && !pointer.leave {
                    self.last_status = format!("DRAG at {},{}", pointer.x, pointer.y);
                }
            }
        }
    }
}

impl eframe::App for DragSelectEframeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.runtime.pump();
        self.update_texture(ctx);

        let generation = self
            .runtime
            .state
            .lock()
            .map(|state| state.generation)
            .unwrap_or(0);

        egui::TopBottomPanel::top("wv11_04_01_drag_select_status").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("WV-11-04-01 EDIT-IT-SPEC-002");
                ui.separator();
                ui.label(format!("Paint generation: {generation}"));
                ui.separator();
                ui.label(format!("Dragging: {}", self.dragging));
                ui.separator();
                ui.label(format!("Last input: {}", self.last_status));
            });
        });

        self.current_pointer = None;
        self.current_button = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut viewer = DragSelectViewer {
                texture: self.texture.as_ref(),
                pointer_transfer: &mut self.current_pointer,
                button_transfer: &mut self.current_button,
                dragging: &mut self.dragging,
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

    println!("WV-11-04-01 CEF pointer drag selection probe start");
    let runtime = match DragSelectRuntime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("WV-11-04-01 CEF pointer drag selection probe failed: {error}");
            std::process::exit(1);
        }
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("WV-11-04-01 Browser Surface Pointer Drag Selection Probe")
            .with_inner_size([1000.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "WV-11-04-01 Browser Surface Pointer Drag Selection Probe",
        native_options,
        Box::new(move |_cc| Ok(Box::new(DragSelectEframeApp::new(runtime)))),
    )
}
