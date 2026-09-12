//! WV-11-04 CEF Browser Surface Text 入力転送 Probe。
//!
//! 役割:
//! - INPUT-IT-SPEC-006 の文字入力を CEF OSR Browser の編集可能要素へ転送する。
//! - Browser Surface 上の input 要素を Pointer Button でフォーカスした後、
//!   egui の Text Event を CEF KEYEVENT_CHAR として送信する。
//! - Backspace / Delete は egui の Key Event を CEF RAWKEYDOWN / KEYUP として送信する。
//!
//! 注意点:
//! - 本ファイルは技術検証用であり、正式 Surface API ではない。
//! - IME 固有処理は本検証対象外。まず通常の確定文字列入力を対象とする。
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
const TEST_URL: &str = "data:text/html,<html><body style='margin:0;background:rgb(26,36,50);color:white;font-family:sans-serif;height:100vh;display:flex;align-items:center;justify-content:center'><div style='text-align:center;width:90%'><h1 style='font-size:42px'>WV-11-04 TEXT INPUT</h1><div style='font-size:22px;margin:18px'>Click the input below, then type text. Backspace / Delete should edit it.</div><input id='probe' type='text' value='' placeholder='TYPE HERE' style='width:80%;height:90px;font-size:38px;padding:12px;border:6px solid white;background:rgb(55,78,105);color:white;box-sizing:border-box'><div id='status' style='font-size:26px;margin-top:24px'>INPUT: waiting</div></div><script>let p=document.getElementById('probe');p.addEventListener('focus',function(){document.getElementById('status').textContent='INPUT: FOCUSED';});p.addEventListener('input',function(){document.getElementById('status').textContent='INPUT: '+p.value;});</script></body></html>";

/// 最新 CEF Paint を保持する共有状態。
///
/// @hldocs.ref doc-20260912-010000Z-WV14#sec_c3x7p1t9m5wf
#[derive(Debug, Default)]
struct InputTextState {
    width: i32,
    height: i32,
    rgba: Vec<u8>,
    generation: u64,
}

#[derive(Clone)]
struct InputTextRenderHandler {
    state: Arc<Mutex<InputTextState>>,
}

wrap_render_handler! {
    struct InputTextRenderHandlerBuilder {
        handler: InputTextRenderHandler,
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

impl InputTextRenderHandlerBuilder {
    fn build(state: Arc<Mutex<InputTextState>>) -> RenderHandler {
        Self::new(InputTextRenderHandler { state })
    }
}

#[derive(Clone)]
struct InputTextLifeSpanHandler {
    browser: Arc<Mutex<Option<Browser>>>,
    browser_created: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
}

wrap_life_span_handler! {
    struct InputTextLifeSpanHandlerBuilder {
        handler: InputTextLifeSpanHandler,
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

impl InputTextLifeSpanHandlerBuilder {
    fn build(
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
    ) -> LifeSpanHandler {
        Self::new(InputTextLifeSpanHandler {
            browser,
            browser_created,
            closed,
        })
    }
}

wrap_client! {
    struct InputTextClientBuilder {
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

impl InputTextClientBuilder {
    fn build(
        state: Arc<Mutex<InputTextState>>,
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
    ) -> Client {
        Self::new(
            InputTextRenderHandlerBuilder::build(state),
            InputTextLifeSpanHandlerBuilder::build(browser, browser_created, closed),
        )
    }
}

#[derive(Clone)]
struct InputTextBrowserProcessHandler {
    state: Arc<Mutex<InputTextState>>,
    browser: Arc<Mutex<Option<Browser>>>,
    browser_created: Arc<AtomicBool>,
    browser_create_failed: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
}

wrap_browser_process_handler! {
    struct InputTextBrowserProcessHandlerBuilder {
        handler: InputTextBrowserProcessHandler,
    }

    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            let mut client = InputTextClientBuilder::build(
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

impl InputTextBrowserProcessHandlerBuilder {
    fn build(handler: InputTextBrowserProcessHandler) -> BrowserProcessHandler {
        Self::new(handler)
    }
}

#[derive(Clone)]
struct InputTextCefApp {
    browser_process_handler: BrowserProcessHandler,
}

wrap_app! {
    struct InputTextCefAppBuilder {
        handler: InputTextCefApp,
    }

    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(self.handler.browser_process_handler.clone())
        }
    }
}

impl InputTextCefAppBuilder {
    fn build(handler: InputTextBrowserProcessHandler) -> App {
        Self::new(InputTextCefApp {
            browser_process_handler: InputTextBrowserProcessHandlerBuilder::build(handler),
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
        eprintln!("WV-11-04 text subprocess failed: cef_execute_process returned {exit_code}");
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
struct ClickTransfer {
    x: i32,
    y: i32,
}

#[derive(Clone, Debug)]
struct TextTransfer {
    text: String,
}

#[derive(Clone, Copy, Debug)]
struct EditKeyTransfer {
    windows_key_code: i32,
    pressed: bool,
    name: &'static str,
}

#[derive(Clone)]
enum InputTextTab {
    Browser,
}

struct InputTextViewer<'a> {
    texture: Option<&'a egui::TextureHandle>,
    click_transfer: &'a mut Option<ClickTransfer>,
    browser_active: &'a mut bool,
}

impl<'a> TabViewer for InputTextViewer<'a> {
    type Tab = InputTextTab;

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn title(&mut self, _tab: &mut Self::Tab) -> egui::WidgetText {
        "Browser Surface Text Input".into()
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
            if let Some(pointer) = response.interact_pointer_pos() {
                if let Some((x, y)) = map_pointer_to_browser(response.rect, pointer) {
                    *self.click_transfer = Some(ClickTransfer { x, y });
                    *self.browser_active = true;
                }
            }
        }
    }
}

struct InputTextRuntime {
    state: Arc<Mutex<InputTextState>>,
    browser: Arc<Mutex<Option<Browser>>>,
    closed: Arc<AtomicBool>,
}

impl InputTextRuntime {
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

        let state = Arc::new(Mutex::new(InputTextState::default()));
        let browser = Arc::new(Mutex::new(None));
        let browser_created = Arc::new(AtomicBool::new(false));
        let browser_create_failed = Arc::new(AtomicBool::new(false));
        let closed = Arc::new(AtomicBool::new(false));

        let handler = InputTextBrowserProcessHandler {
            state: state.clone(),
            browser: browser.clone(),
            browser_created: browser_created.clone(),
            browser_create_failed: browser_create_failed.clone(),
            closed: closed.clone(),
        };
        let mut app = InputTextCefAppBuilder::build(handler);

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

    /// Browser の編集可能要素へフォーカスさせるため Pointer click を転送する。
    fn request_click(&self, transfer: ClickTransfer) -> bool {
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
            let mut task = ClickTaskBuilder::build(browser, transfer);
            return post_task(ThreadId::UI, Some(&mut task)) == 1;
        }

        #[cfg(not(target_os = "windows"))]
        {
            send_click(&browser, transfer);
            true
        }
    }

    /// 確定済み文字列を CEF Browser へ転送する。
    fn request_text(&self, transfer: TextTransfer) -> bool {
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
            let mut task = TextTaskBuilder::build(browser, transfer);
            return post_task(ThreadId::UI, Some(&mut task)) == 1;
        }

        #[cfg(not(target_os = "windows"))]
        {
            send_text(&browser, &transfer.text);
            true
        }
    }

    /// Backspace / Delete の Keyboard Event を CEF Browser へ転送する。
    fn request_edit_key(&self, transfer: EditKeyTransfer) -> bool {
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
            let mut task = EditKeyTaskBuilder::build(browser, transfer);
            return post_task(ThreadId::UI, Some(&mut task)) == 1;
        }

        #[cfg(not(target_os = "windows"))]
        {
            send_edit_key(&browser, transfer);
            true
        }
    }
}

fn send_click(browser: &Browser, transfer: ClickTransfer) {
    let Some(host) = browser.host() else {
        return;
    };
    let event = MouseEvent {
        x: transfer.x,
        y: transfer.y,
        ..Default::default()
    };
    host.send_mouse_move_event(Some(&event), 0);
    host.send_mouse_click_event(Some(&event), MouseButtonType::LEFT, 0, 1);
    host.send_mouse_click_event(Some(&event), MouseButtonType::LEFT, 1, 1);
    println!("CEF text focus click sent: x={}, y={}", transfer.x, transfer.y);
}

/// CEF BrowserHost へ CHAR KeyEvent を UTF-16 code unit 単位で送信する。
///
/// @hldocs.ref doc-20260912-010000Z-WV14#sec_c3x7p1t9m5wf
fn send_text(browser: &Browser, text: &str) {
    let Some(host) = browser.host() else {
        return;
    };

    for unit in text.encode_utf16() {
        let event = KeyEvent {
            type_: KeyEventType::CHAR,
            windows_key_code: unit as i32,
            native_key_code: unit as i32,
            character: unit,
            unmodified_character: unit,
            ..Default::default()
        };
        host.send_key_event(Some(&event));
    }
    println!("CEF text sent: {:?}", text);
}

/// CEF BrowserHost へ編集キーの DOWN / UP を送信する。
fn send_edit_key(browser: &Browser, transfer: EditKeyTransfer) {
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
        "CEF edit key sent: {} key={} vk={}",
        if transfer.pressed { "DOWN" } else { "UP" },
        transfer.name,
        transfer.windows_key_code
    );
}

#[derive(Clone)]
struct ClickTask {
    browser: Browser,
    transfer: ClickTransfer,
}

wrap_task! {
    struct ClickTaskBuilder {
        task: ClickTask,
    }

    impl Task {
        fn execute(&self) {
            send_click(&self.task.browser, self.task.transfer);
        }
    }
}

impl ClickTaskBuilder {
    fn build(browser: Browser, transfer: ClickTransfer) -> Task {
        Self::new(ClickTask { browser, transfer })
    }
}

#[derive(Clone)]
struct TextTask {
    browser: Browser,
    transfer: TextTransfer,
}

wrap_task! {
    struct TextTaskBuilder {
        task: TextTask,
    }

    impl Task {
        fn execute(&self) {
            send_text(&self.task.browser, &self.task.transfer.text);
        }
    }
}

impl TextTaskBuilder {
    fn build(browser: Browser, transfer: TextTransfer) -> Task {
        Self::new(TextTask { browser, transfer })
    }
}

#[derive(Clone)]
struct EditKeyTask {
    browser: Browser,
    transfer: EditKeyTransfer,
}

wrap_task! {
    struct EditKeyTaskBuilder {
        task: EditKeyTask,
    }

    impl Task {
        fn execute(&self) {
            send_edit_key(&self.task.browser, self.task.transfer);
        }
    }
}

impl EditKeyTaskBuilder {
    fn build(browser: Browser, transfer: EditKeyTransfer) -> Task {
        Self::new(EditKeyTask { browser, transfer })
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

impl Drop for InputTextRuntime {
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
                    eprintln!("WV-11-04 text probe: failed to post browser close task");
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
            eprintln!("WV-11-04 text probe: browser close timeout");
        }
        shutdown();
    }
}

struct InputTextEframeApp {
    dock_state: DockState<InputTextTab>,
    runtime: InputTextRuntime,
    texture: Option<egui::TextureHandle>,
    applied_generation: u64,
    browser_active: bool,
    current_click: Option<ClickTransfer>,
    last_text: String,
}

impl InputTextEframeApp {
    fn new(runtime: InputTextRuntime) -> Self {
        Self {
            dock_state: DockState::new(vec![InputTextTab::Browser]),
            runtime,
            texture: None,
            applied_generation: 0,
            browser_active: false,
            current_click: None,
            last_text: "waiting".to_string(),
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
                    "wv11_04_text_probe",
                    image,
                    egui::TextureOptions::LINEAR,
                ));
            }
        }
        self.applied_generation = generation;
    }

    /// egui の Text / 編集キー Event を検出して CEF へ転送する。
    fn collect_input(&mut self, ctx: &egui::Context) {
        if !self.browser_active {
            return;
        }

        let events = ctx.input(|input| input.events.clone());
        for event in events {
            match event {
                egui::Event::Text(text) if !text.is_empty() => {
                    if self.runtime.request_text(TextTransfer { text: text.clone() }) {
                        self.last_text = text;
                    }
                }
                egui::Event::Key {
                    key,
                    pressed,
                    ..
                } => {
                    let mapped = match key {
                        egui::Key::Backspace => Some((0x08, "Backspace")),
                        egui::Key::Delete => Some((0x2E, "Delete")),
                        _ => None,
                    };
                    let Some((windows_key_code, name)) = mapped else {
                        continue;
                    };
                    if self.runtime.request_edit_key(EditKeyTransfer {
                        windows_key_code,
                        pressed,
                        name,
                    }) {
                        self.last_text = format!(
                            "{} {}",
                            if pressed { "DOWN" } else { "UP" },
                            name
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

impl eframe::App for InputTextEframeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.runtime.pump();
        self.update_texture(ctx);

        let generation = self
            .runtime
            .state
            .lock()
            .map(|state| state.generation)
            .unwrap_or(0);

        egui::TopBottomPanel::top("wv11_04_text_status").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("WV-11-04 INPUT-IT-SPEC-006");
                ui.separator();
                ui.label(format!("Paint generation: {generation}"));
                ui.separator();
                ui.label(format!(
                    "Browser input: {}",
                    if self.browser_active { "ACTIVE" } else { "click input" }
                ));
                ui.separator();
                ui.label(format!("Last input: {}", self.last_text));
            });
        });

        self.current_click = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut viewer = InputTextViewer {
                texture: self.texture.as_ref(),
                click_transfer: &mut self.current_click,
                browser_active: &mut self.browser_active,
            };
            DockArea::new(&mut self.dock_state).show_inside(ui, &mut viewer);
        });

        if let Some(click) = self.current_click {
            self.runtime.request_click(click);
        }
        self.collect_input(ctx);

        ctx.request_repaint_after(MESSAGE_PUMP_INTERVAL);
    }
}

fn main() -> eframe::Result<()> {
    let cli_args: Vec<String> = std::env::args().collect();
    if is_cef_subprocess(&cli_args) {
        std::process::exit(run_subprocess());
    }

    println!("WV-11-04 CEF text input probe start");
    let runtime = match InputTextRuntime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("WV-11-04 CEF text input probe failed: {error}");
            std::process::exit(1);
        }
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("WV-11-04 Browser Surface Text Input Probe")
            .with_inner_size([1000.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "WV-11-04 Browser Surface Text Input Probe",
        native_options,
        Box::new(move |_cc| Ok(Box::new(InputTextEframeApp::new(runtime)))),
    )
}
