//! WV-11-04 CEF Browser Surface Keyboard 転送 Probe。
//!
//! 役割:
//! - INPUT-IT-SPEC-005 の Keyboard 押下・解放を CEF OSR Browser へ転送する。
//! - Browser 側で keydown / keyup を表示し、転送結果を視覚確認する。
//!
//! 注意点:
//! - 本ファイルは技術検証用であり、正式 Surface API ではない。
//! - Text 入力は INPUT-IT-SPEC-006 で別途検証するため、本 Probe では文字生成を目的としない。
//! - Browser Surface をクリックした後のみ Keyboard Event を転送する。
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

const VIEW_WIDTH: i32 = 800;
const VIEW_HEIGHT: i32 = 600;
const MESSAGE_PUMP_INTERVAL: Duration = Duration::from_millis(10);
const BROWSER_CREATE_TIMEOUT: Duration = Duration::from_secs(3);
const CLOSE_TIMEOUT: Duration = Duration::from_secs(3);
const TEST_URL: &str = "data:text/html,<html><body tabindex='0' style='margin:0;background:rgb(26,36,50);color:white;font-family:sans-serif;height:100vh;display:flex;align-items:center;justify-content:center'><div style='text-align:center;width:90%'><h1 style='font-size:42px'>WV-11-04 KEYBOARD</h1><div style='font-size:24px;margin:24px'>Click the Browser Surface, then press A / Enter / Arrow keys.</div><div id='down' style='font-size:34px;margin:20px;padding:24px;border:4px solid white;background:rgb(55,78,105)'>KEY DOWN: waiting</div><div id='up' style='font-size:34px;margin:20px;padding:24px;border:4px solid white;background:rgb(55,78,105)'>KEY UP: waiting</div></div><script>document.body.focus();document.addEventListener('keydown',function(e){document.getElementById('down').textContent='KEY DOWN: '+e.key+' / code '+e.keyCode;});document.addEventListener('keyup',function(e){document.getElementById('up').textContent='KEY UP: '+e.key+' / code '+e.keyCode;});</script></body></html>";

/// 最新 CEF Paint を保持する共有状態。
///
/// @hldocs.ref doc-20260912-010000Z-WV14#sec_r5q9d2m8v1kc
#[derive(Debug, Default)]
struct InputKeyboardState {
    width: i32,
    height: i32,
    rgba: Vec<u8>,
    generation: u64,
}

#[derive(Clone)]
struct InputKeyboardRenderHandler {
    state: Arc<Mutex<InputKeyboardState>>,
}

wrap_render_handler! {
    struct InputKeyboardRenderHandlerBuilder {
        handler: InputKeyboardRenderHandler,
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

impl InputKeyboardRenderHandlerBuilder {
    fn build(state: Arc<Mutex<InputKeyboardState>>) -> RenderHandler {
        Self::new(InputKeyboardRenderHandler { state })
    }
}

#[derive(Clone)]
struct InputKeyboardLifeSpanHandler {
    browser: Arc<Mutex<Option<Browser>>>,
    browser_created: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
}

wrap_life_span_handler! {
    struct InputKeyboardLifeSpanHandlerBuilder {
        handler: InputKeyboardLifeSpanHandler,
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

impl InputKeyboardLifeSpanHandlerBuilder {
    fn build(
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
    ) -> LifeSpanHandler {
        Self::new(InputKeyboardLifeSpanHandler {
            browser,
            browser_created,
            closed,
        })
    }
}

wrap_client! {
    struct InputKeyboardClientBuilder {
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

impl InputKeyboardClientBuilder {
    fn build(
        state: Arc<Mutex<InputKeyboardState>>,
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
    ) -> Client {
        Self::new(
            InputKeyboardRenderHandlerBuilder::build(state),
            InputKeyboardLifeSpanHandlerBuilder::build(browser, browser_created, closed),
        )
    }
}

#[derive(Clone)]
struct InputKeyboardBrowserProcessHandler {
    state: Arc<Mutex<InputKeyboardState>>,
    browser: Arc<Mutex<Option<Browser>>>,
    browser_created: Arc<AtomicBool>,
    browser_create_failed: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
}

wrap_browser_process_handler! {
    struct InputKeyboardBrowserProcessHandlerBuilder {
        handler: InputKeyboardBrowserProcessHandler,
    }

    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            let mut client = InputKeyboardClientBuilder::build(
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

impl InputKeyboardBrowserProcessHandlerBuilder {
    fn build(handler: InputKeyboardBrowserProcessHandler) -> BrowserProcessHandler {
        Self::new(handler)
    }
}

#[derive(Clone)]
struct InputKeyboardCefApp {
    browser_process_handler: BrowserProcessHandler,
}

wrap_app! {
    struct InputKeyboardCefAppBuilder {
        handler: InputKeyboardCefApp,
    }

    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(self.handler.browser_process_handler.clone())
        }
    }
}

impl InputKeyboardCefAppBuilder {
    fn build(handler: InputKeyboardBrowserProcessHandler) -> App {
        Self::new(InputKeyboardCefApp {
            browser_process_handler: InputKeyboardBrowserProcessHandlerBuilder::build(handler),
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
        eprintln!("WV-11-04 keyboard subprocess failed: cef_execute_process returned {exit_code}");
        1
    }
}

/// egui Key を Windows Virtual-Key Code へ変換する。
///
/// # 引数
/// - `key`: egui が通知した論理キー。
///
/// # 戻り値
/// - 本 Probe が検証対象とするキーなら Virtual-Key Code と表示名を返す。
fn map_key(key: egui::Key) -> Option<(i32, &'static str)> {
    let mapped = match key {
        egui::Key::A => (0x41, "A"),
        egui::Key::B => (0x42, "B"),
        egui::Key::C => (0x43, "C"),
        egui::Key::D => (0x44, "D"),
        egui::Key::E => (0x45, "E"),
        egui::Key::F => (0x46, "F"),
        egui::Key::G => (0x47, "G"),
        egui::Key::H => (0x48, "H"),
        egui::Key::I => (0x49, "I"),
        egui::Key::J => (0x4A, "J"),
        egui::Key::K => (0x4B, "K"),
        egui::Key::L => (0x4C, "L"),
        egui::Key::M => (0x4D, "M"),
        egui::Key::N => (0x4E, "N"),
        egui::Key::O => (0x4F, "O"),
        egui::Key::P => (0x50, "P"),
        egui::Key::Q => (0x51, "Q"),
        egui::Key::R => (0x52, "R"),
        egui::Key::S => (0x53, "S"),
        egui::Key::T => (0x54, "T"),
        egui::Key::U => (0x55, "U"),
        egui::Key::V => (0x56, "V"),
        egui::Key::W => (0x57, "W"),
        egui::Key::X => (0x58, "X"),
        egui::Key::Y => (0x59, "Y"),
        egui::Key::Z => (0x5A, "Z"),
        egui::Key::Enter => (0x0D, "Enter"),
        egui::Key::Space => (0x20, "Space"),
        egui::Key::Backspace => (0x08, "Backspace"),
        egui::Key::Tab => (0x09, "Tab"),
        egui::Key::Escape => (0x1B, "Escape"),
        egui::Key::ArrowLeft => (0x25, "ArrowLeft"),
        egui::Key::ArrowUp => (0x26, "ArrowUp"),
        egui::Key::ArrowRight => (0x27, "ArrowRight"),
        egui::Key::ArrowDown => (0x28, "ArrowDown"),
        _ => return None,
    };
    Some(mapped)
}

#[derive(Clone, Copy, Debug)]
struct KeyboardTransfer {
    windows_key_code: i32,
    pressed: bool,
    repeat: bool,
    name: &'static str,
}

#[derive(Clone)]
enum InputKeyboardTab {
    Browser,
}

struct InputKeyboardViewer<'a> {
    texture: Option<&'a egui::TextureHandle>,
    browser_active: &'a mut bool,
}

impl<'a> TabViewer for InputKeyboardViewer<'a> {
    type Tab = InputKeyboardTab;

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn title(&mut self, _tab: &mut Self::Tab) -> egui::WidgetText {
        "Browser Surface Keyboard".into()
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
                .sense(egui::Sense::click()),
        );
        if response.clicked() {
            *self.browser_active = true;
        }
    }
}

struct InputKeyboardRuntime {
    state: Arc<Mutex<InputKeyboardState>>,
    browser: Arc<Mutex<Option<Browser>>>,
    closed: Arc<AtomicBool>,
}

impl InputKeyboardRuntime {
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

        let state = Arc::new(Mutex::new(InputKeyboardState::default()));
        let browser = Arc::new(Mutex::new(None));
        let browser_created = Arc::new(AtomicBool::new(false));
        let browser_create_failed = Arc::new(AtomicBool::new(false));
        let closed = Arc::new(AtomicBool::new(false));

        let handler = InputKeyboardBrowserProcessHandler {
            state: state.clone(),
            browser: browser.clone(),
            browser_created: browser_created.clone(),
            browser_create_failed: browser_create_failed.clone(),
            closed: closed.clone(),
        };
        let mut app = InputKeyboardCefAppBuilder::build(handler);

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

    /// Keyboard 押下・解放を CEF Browser へ転送する。
    ///
    /// # 引数
    /// - `transfer`: Virtual-Key Code と押下状態。
    ///
    /// # 戻り値
    /// - CEF UI thread への送信要求を受理できた場合 `true`。
    fn request_keyboard(&self, transfer: KeyboardTransfer) -> bool {
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
            let mut task = KeyboardTaskBuilder::build(browser, transfer);
            return post_task(ThreadId::UI, Some(&mut task)) == 1;
        }

        #[cfg(not(target_os = "windows"))]
        {
            send_keyboard(&browser, transfer);
            true
        }
    }
}

/// CEF BrowserHost へ Keyboard Event を送信する。
///
/// @hldocs.ref doc-20260912-010000Z-WV14#sec_r5q9d2m8v1kc
fn send_keyboard(browser: &Browser, transfer: KeyboardTransfer) {
    let Some(host) = browser.host() else {
        return;
    };

    let event = KeyEvent {
        type_: if transfer.pressed {
            KeyEventType::RAWKEYDOWN
        } else {
            KeyEventType::KEYUP
        },
        windows_key_code: transfer.windows_key_code,
        native_key_code: transfer.windows_key_code,
        ..Default::default()
    };
    host.send_key_event(Some(&event));

    println!(
        "CEF key sent: {} key={} vk={} repeat={}",
        if transfer.pressed { "DOWN" } else { "UP" },
        transfer.name,
        transfer.windows_key_code,
        transfer.repeat
    );
}

#[derive(Clone)]
struct KeyboardTask {
    browser: Browser,
    transfer: KeyboardTransfer,
}

wrap_task! {
    struct KeyboardTaskBuilder {
        task: KeyboardTask,
    }

    impl Task {
        fn execute(&self) {
            send_keyboard(&self.task.browser, self.task.transfer);
        }
    }
}

impl KeyboardTaskBuilder {
    fn build(browser: Browser, transfer: KeyboardTransfer) -> Task {
        Self::new(KeyboardTask { browser, transfer })
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

impl Drop for InputKeyboardRuntime {
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
                    eprintln!("WV-11-04 keyboard probe: failed to post browser close task");
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
            eprintln!("WV-11-04 keyboard probe: browser close timeout");
        }
        shutdown();
    }
}

struct InputKeyboardEframeApp {
    dock_state: DockState<InputKeyboardTab>,
    runtime: InputKeyboardRuntime,
    texture: Option<egui::TextureHandle>,
    applied_generation: u64,
    browser_active: bool,
    last_key_status: String,
}

impl InputKeyboardEframeApp {
    fn new(runtime: InputKeyboardRuntime) -> Self {
        Self {
            dock_state: DockState::new(vec![InputKeyboardTab::Browser]),
            runtime,
            texture: None,
            applied_generation: 0,
            browser_active: false,
            last_key_status: "waiting".to_string(),
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
                    "wv11_04_keyboard_probe",
                    image,
                    egui::TextureOptions::LINEAR,
                ));
            }
        }
        self.applied_generation = generation;
    }

    /// egui の Keyboard Event を検出して CEF へ転送する。
    fn send_keyboard_events(&mut self, ctx: &egui::Context) {
        if !self.browser_active {
            return;
        }

        let events = ctx.input(|input| input.events.clone());
        for event in events {
            let egui::Event::Key {
                key,
                pressed,
                repeat,
                ..
            } = event
            else {
                continue;
            };
            let Some((windows_key_code, name)) = map_key(key) else {
                continue;
            };

            let transfer = KeyboardTransfer {
                windows_key_code,
                pressed,
                repeat,
                name,
            };
            if self.runtime.request_keyboard(transfer) {
                self.last_key_status = format!(
                    "{} {}",
                    if pressed { "DOWN" } else { "UP" },
                    name
                );
            }
        }
    }
}

impl eframe::App for InputKeyboardEframeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.runtime.pump();
        self.update_texture(ctx);

        let generation = self
            .runtime
            .state
            .lock()
            .map(|state| state.generation)
            .unwrap_or(0);

        egui::TopBottomPanel::top("wv11_04_keyboard_status").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("WV-11-04 INPUT-IT-SPEC-005");
                ui.separator();
                ui.label(format!("Paint generation: {generation}"));
                ui.separator();
                ui.label(format!(
                    "Browser input: {}",
                    if self.browser_active { "ACTIVE" } else { "click surface to activate" }
                ));
                ui.separator();
                ui.label(format!("Last key: {}", self.last_key_status));
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut viewer = InputKeyboardViewer {
                texture: self.texture.as_ref(),
                browser_active: &mut self.browser_active,
            };
            DockArea::new(&mut self.dock_state).show_inside(ui, &mut viewer);
        });

        self.send_keyboard_events(ctx);
        ctx.request_repaint_after(MESSAGE_PUMP_INTERVAL);
    }
}

fn main() -> eframe::Result<()> {
    let cli_args: Vec<String> = std::env::args().collect();
    if is_cef_subprocess(&cli_args) {
        std::process::exit(run_subprocess());
    }

    println!("WV-11-04 CEF keyboard probe start");
    let runtime = match InputKeyboardRuntime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("WV-11-04 CEF keyboard probe failed: {error}");
            std::process::exit(1);
        }
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("WV-11-04 Browser Surface Keyboard Probe")
            .with_inner_size([1000.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "WV-11-04 Browser Surface Keyboard Probe",
        native_options,
        Box::new(move |_cc| Ok(Box::new(InputKeyboardEframeApp::new(runtime)))),
    )
}
