//! WV-11-04-02 CEF Browser Surface IME Composition Probe。
//!
//! 役割:
//! - IME-IT-SPEC-001 / 002 の IME 開始・Composition 更新を検証する。
//! - egui が受信した IME Preedit を selection range 付きで CEF OSR Browser へ転送する。
//! - CEF の `on_ime_composition_range_changed` callback を監視し、Browser 側で
//!   Composition range と character bounds が成立しているかを確認する。
//! - Candidate 位置同期は本 Probe では判定しない。
//!
//! 注意点:
//! - 本ファイルは技術検証用であり、正式 Surface API ではない。
//! - OS 固有 IME 差異をアプリ利用側へ公開する正式設計は本検証では定義しない。
//! - Windows では CEF の multi-threaded message loop を使用する。
//! - Commit / Cancel は後続検証で扱う。

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
const TEST_URL: &str = "data:text/html,<html><body style='margin:0;background:rgb(26,36,50);color:white;font-family:sans-serif;height:100vh;display:flex;align-items:center;justify-content:center'><div style='text-align:center;width:92%'><h1 style='font-size:38px'>WV-11-04-02 IME COMPOSITION</h1><div style='font-size:21px;margin:16px'>Click the input, enable the OS IME, and type without confirming.</div><input id='probe' type='text' value='' style='width:82%;height:88px;font-size:38px;padding:12px;border:6px solid white;background:rgb(55,78,105);color:white;box-sizing:border-box'><div id='value' style='font-size:24px;margin-top:18px'>VALUE: empty</div></div><script>let p=document.getElementById('probe');let value=document.getElementById('value');p.addEventListener('input',function(){value.textContent='VALUE: '+(p.value||'<empty>');});</script></body></html>";

#[derive(Debug, Default, Clone)]
struct CompositionInfo {
    selected_range: Option<Range>,
    character_bounds: Vec<Rect>,
    callback_count: u64,
}

/// 最新 CEF Paint と IME range callback を保持する共有状態。
///
/// @hldocs.ref doc-20260912-104801Z-WV16#sec_b6m1r9p3x7da
#[derive(Debug, Default)]
struct ImeCompositionState {
    width: i32,
    height: i32,
    rgba: Vec<u8>,
    generation: u64,
    composition: CompositionInfo,
}

#[derive(Clone)]
struct ImeCompositionRenderHandler {
    state: Arc<Mutex<ImeCompositionState>>,
}

wrap_render_handler! {
    struct ImeCompositionRenderHandlerBuilder {
        handler: ImeCompositionRenderHandler,
    }

    impl RenderHandler {
        fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut Rect>) {
            if let Some(rect) = rect {
                rect.width = VIEW_WIDTH;
                rect.height = VIEW_HEIGHT;
            }
        }

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

        fn on_ime_composition_range_changed(
            &self,
            _browser: Option<&mut Browser>,
            selected_range: Option<&Range>,
            character_bounds: Option<&[Rect]>,
        ) {
            let Ok(mut state) = self.handler.state.lock() else {
                return;
            };
            state.composition.selected_range = selected_range.cloned();
            state.composition.character_bounds = character_bounds.unwrap_or_default().to_vec();
            state.composition.callback_count = state.composition.callback_count.saturating_add(1);

            println!(
                "CEF IME range changed: selected={:?} bounds={} callback_count={}",
                state.composition.selected_range,
                state.composition.character_bounds.len(),
                state.composition.callback_count
            );
            if let Some(first) = state.composition.character_bounds.first() {
                println!(
                    "CEF IME first bound: x={} y={} w={} h={}",
                    first.x, first.y, first.width, first.height
                );
            }
        }
    }
}

impl ImeCompositionRenderHandlerBuilder {
    fn build(state: Arc<Mutex<ImeCompositionState>>) -> RenderHandler {
        Self::new(ImeCompositionRenderHandler { state })
    }
}

#[derive(Clone)]
struct ImeCompositionLifeSpanHandler {
    browser: Arc<Mutex<Option<Browser>>>,
    browser_created: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
}

wrap_life_span_handler! {
    struct ImeCompositionLifeSpanHandlerBuilder {
        handler: ImeCompositionLifeSpanHandler,
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

impl ImeCompositionLifeSpanHandlerBuilder {
    fn build(
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
    ) -> LifeSpanHandler {
        Self::new(ImeCompositionLifeSpanHandler {
            browser,
            browser_created,
            closed,
        })
    }
}

wrap_client! {
    struct ImeCompositionClientBuilder {
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

impl ImeCompositionClientBuilder {
    fn build(
        state: Arc<Mutex<ImeCompositionState>>,
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
    ) -> Client {
        Self::new(
            ImeCompositionRenderHandlerBuilder::build(state),
            ImeCompositionLifeSpanHandlerBuilder::build(browser, browser_created, closed),
        )
    }
}

#[derive(Clone)]
struct ImeCompositionBrowserProcessHandler {
    state: Arc<Mutex<ImeCompositionState>>,
    browser: Arc<Mutex<Option<Browser>>>,
    browser_created: Arc<AtomicBool>,
    browser_create_failed: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
}

wrap_browser_process_handler! {
    struct ImeCompositionBrowserProcessHandlerBuilder {
        handler: ImeCompositionBrowserProcessHandler,
    }

    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            let mut client = ImeCompositionClientBuilder::build(
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
                self.handler.browser_create_failed.store(true, Ordering::Release);
            }
        }
    }
}

impl ImeCompositionBrowserProcessHandlerBuilder {
    fn build(handler: ImeCompositionBrowserProcessHandler) -> BrowserProcessHandler {
        Self::new(handler)
    }
}

#[derive(Clone)]
struct ImeCompositionCefApp {
    browser_process_handler: BrowserProcessHandler,
}

wrap_app! {
    struct ImeCompositionCefAppBuilder {
        handler: ImeCompositionCefApp,
    }

    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(self.handler.browser_process_handler.clone())
        }
    }
}

impl ImeCompositionCefAppBuilder {
    fn build(handler: ImeCompositionBrowserProcessHandler) -> App {
        Self::new(ImeCompositionCefApp {
            browser_process_handler: ImeCompositionBrowserProcessHandlerBuilder::build(handler),
        })
    }
}

fn is_cef_subprocess(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--type" || arg.starts_with("--type="))
}

fn run_subprocess() -> i32 {
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    let args = args::Args::new();
    let exit_code = execute_process(Some(args.as_main_args()), None, ptr::null_mut());
    if exit_code >= 0 { exit_code } else { 1 }
}

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
struct ClickTransfer { x: i32, y: i32 }

#[derive(Clone, Debug)]
struct ImePreeditTransfer { text: String }

#[derive(Clone)]
enum ImeCompositionTab { Browser }

struct ImeCompositionViewer<'a> {
    texture: Option<&'a egui::TextureHandle>,
    click_transfer: &'a mut Option<ClickTransfer>,
    browser_active: &'a mut bool,
    browser_rect: &'a mut Option<egui::Rect>,
}

impl<'a> TabViewer for ImeCompositionViewer<'a> {
    type Tab = ImeCompositionTab;

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool { false }
    fn title(&mut self, _tab: &mut Self::Tab) -> egui::WidgetText { "Browser Surface IME Composition".into() }

    fn ui(&mut self, ui: &mut egui::Ui, _tab: &mut Self::Tab) {
        let available = ui.available_size();
        let Some(texture) = self.texture else {
            ui.centered_and_justified(|ui| ui.label("Waiting for CEF OSR Paint..."));
            return;
        };
        let response = ui.add(
            egui::Image::new(texture)
                .fit_to_exact_size(available)
                .sense(egui::Sense::click()),
        );
        *self.browser_rect = Some(response.rect);
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

struct ImeCompositionRuntime {
    state: Arc<Mutex<ImeCompositionState>>,
    browser: Arc<Mutex<Option<Browser>>>,
    closed: Arc<AtomicBool>,
}

impl ImeCompositionRuntime {
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

        let state = Arc::new(Mutex::new(ImeCompositionState::default()));
        let browser = Arc::new(Mutex::new(None));
        let browser_created = Arc::new(AtomicBool::new(false));
        let browser_create_failed = Arc::new(AtomicBool::new(false));
        let closed = Arc::new(AtomicBool::new(false));

        let handler = ImeCompositionBrowserProcessHandler {
            state: state.clone(),
            browser: browser.clone(),
            browser_created: browser_created.clone(),
            browser_create_failed: browser_create_failed.clone(),
            closed: closed.clone(),
        };
        let mut app = ImeCompositionCefAppBuilder::build(handler);
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

    fn request_click(&self, transfer: ClickTransfer) -> bool {
        let Some(browser) = self.browser.lock().ok().and_then(|slot| slot.as_ref().cloned()) else { return false; };
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

    fn request_ime_preedit(&self, transfer: ImePreeditTransfer) -> bool {
        let Some(browser) = self.browser.lock().ok().and_then(|slot| slot.as_ref().cloned()) else { return false; };
        #[cfg(target_os = "windows")]
        {
            let mut task = ImePreeditTaskBuilder::build(browser, transfer);
            return post_task(ThreadId::UI, Some(&mut task)) == 1;
        }
        #[cfg(not(target_os = "windows"))]
        {
            send_ime_preedit(&browser, &transfer);
            true
        }
    }
}

fn send_click(browser: &Browser, transfer: ClickTransfer) {
    let Some(host) = browser.host() else { return; };
    let event = MouseEvent { x: transfer.x, y: transfer.y, ..Default::default() };
    host.send_mouse_move_event(Some(&event), 0);
    host.send_mouse_click_event(Some(&event), MouseButtonType::LEFT, 0, 1);
    host.send_mouse_click_event(Some(&event), MouseButtonType::LEFT, 1, 1);
    println!("CEF IME composition focus click sent: x={}, y={}", transfer.x, transfer.y);
}

/// CEF BrowserHost へ selection range 付き IME Preedit を送信する。
///
/// @hldocs.ref doc-20260912-104801Z-WV16#sec_b6m1r9p3x7da
fn send_ime_preedit(browser: &Browser, transfer: &ImePreeditTransfer) {
    if transfer.text.is_empty() { return; }
    let Some(host) = browser.host() else { return; };
    let text = CefString::from(transfer.text.as_str());
    let utf16_len = transfer.text.encode_utf16().count() as u32;
    let selection = Range { from: utf16_len, to: utf16_len };
    host.ime_set_composition(Some(&text), None, None, Some(&selection));
    println!(
        "CEF IME composition set: {:?} selection={}..{}",
        transfer.text, selection.from, selection.to
    );
}

#[derive(Clone)]
struct ClickTask { browser: Browser, transfer: ClickTransfer }
wrap_task! {
    struct ClickTaskBuilder { task: ClickTask, }
    impl Task { fn execute(&self) { send_click(&self.task.browser, self.task.transfer); } }
}
impl ClickTaskBuilder {
    fn build(browser: Browser, transfer: ClickTransfer) -> Task { Self::new(ClickTask { browser, transfer }) }
}

#[derive(Clone)]
struct ImePreeditTask { browser: Browser, transfer: ImePreeditTransfer }
wrap_task! {
    struct ImePreeditTaskBuilder { task: ImePreeditTask, }
    impl Task { fn execute(&self) { send_ime_preedit(&self.task.browser, &self.task.transfer); } }
}
impl ImePreeditTaskBuilder {
    fn build(browser: Browser, transfer: ImePreeditTransfer) -> Task { Self::new(ImePreeditTask { browser, transfer }) }
}

#[derive(Clone)]
struct CloseBrowserTask { browser: Browser }
wrap_task! {
    struct CloseBrowserTaskBuilder { task: CloseBrowserTask, }
    impl Task { fn execute(&self) { if let Some(host) = self.task.browser.host() { host.close_browser(true.into()); } } }
}
impl CloseBrowserTaskBuilder {
    fn build(browser: Browser) -> Task { Self::new(CloseBrowserTask { browser }) }
}

impl Drop for ImeCompositionRuntime {
    fn drop(&mut self) {
        let browser = self.browser.lock().ok().and_then(|slot| slot.as_ref().cloned());
        if let Some(browser) = browser {
            #[cfg(target_os = "windows")]
            {
                let mut task = CloseBrowserTaskBuilder::build(browser);
                if post_task(ThreadId::UI, Some(&mut task)) != 1 {
                    eprintln!("WV-11-04-02 IME composition probe: failed to post browser close task");
                }
            }
            #[cfg(not(target_os = "windows"))]
            if let Some(host) = browser.host() { host.close_browser(true.into()); }
        }
        let started = Instant::now();
        while !self.closed.load(Ordering::Acquire) && started.elapsed() < CLOSE_TIMEOUT {
            #[cfg(not(target_os = "windows"))]
            do_message_loop_work();
            sleep(MESSAGE_PUMP_INTERVAL);
        }
        if !self.closed.load(Ordering::Acquire) {
            eprintln!("WV-11-04-02 IME composition probe: browser close timeout");
        }
        shutdown();
    }
}

struct ImeCompositionEframeApp {
    dock_state: DockState<ImeCompositionTab>,
    runtime: ImeCompositionRuntime,
    texture: Option<egui::TextureHandle>,
    applied_generation: u64,
    browser_active: bool,
    current_click: Option<ClickTransfer>,
    browser_rect: Option<egui::Rect>,
    last_ime_status: String,
}

impl ImeCompositionEframeApp {
    fn new(runtime: ImeCompositionRuntime) -> Self {
        Self {
            dock_state: DockState::new(vec![ImeCompositionTab::Browser]),
            runtime,
            texture: None,
            applied_generation: 0,
            browser_active: false,
            current_click: None,
            browser_rect: None,
            last_ime_status: "waiting".to_string(),
        }
    }

    fn update_texture(&mut self, ctx: &egui::Context) {
        let snapshot = {
            let Ok(state) = self.runtime.state.lock() else { return; };
            if state.generation == self.applied_generation || state.width <= 0 || state.height <= 0 || state.rgba.is_empty() {
                None
            } else {
                Some((state.generation, state.width as usize, state.height as usize, state.rgba.clone()))
            }
        };
        let Some((generation, width, height, rgba)) = snapshot else { return; };
        let image = egui::ColorImage::from_rgba_unmultiplied([width, height], &rgba);
        match self.texture.as_mut() {
            Some(texture) if texture.size() == [width, height] => texture.set(image, egui::TextureOptions::LINEAR),
            _ => {
                self.texture = Some(ctx.load_texture(
                    "wv11_04_02_ime_composition_probe",
                    image,
                    egui::TextureOptions::LINEAR,
                ));
            }
        }
        self.applied_generation = generation;
    }

    fn collect_ime_events(&mut self, ctx: &egui::Context) {
        if !self.browser_active { return; }
        let events = ctx.input(|input| input.events.clone());
        for event in events {
            let egui::Event::Ime(ime_event) = event else { continue; };
            match ime_event {
                egui::ImeEvent::Preedit(text) if !text.is_empty() => {
                    println!("egui IME preedit observed: {:?}", text);
                    if self.runtime.request_ime_preedit(ImePreeditTransfer { text: text.clone() }) {
                        self.last_ime_status = format!("Preedit: {text}");
                    }
                }
                egui::ImeEvent::Commit(text) => {
                    println!("egui IME commit observed but not forwarded in composition probe: {:?}", text);
                    self.last_ime_status = format!("Commit observed: {text}");
                }
                _ => {}
            }
        }
    }
}

impl eframe::App for ImeCompositionEframeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.runtime.pump();
        self.update_texture(ctx);

        if self.browser_active {
            ctx.send_viewport_cmd(egui::ViewportCommand::IMEAllowed(true));
        }

        let (generation, callback_count, selected, bounds) = self.runtime.state.lock().map(|state| {
            (
                state.generation,
                state.composition.callback_count,
                state.composition.selected_range.clone(),
                state.composition.character_bounds.len(),
            )
        }).unwrap_or((0, 0, None, 0));

        egui::TopBottomPanel::top("wv11_04_02_ime_composition_status").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("WV-11-04-02 IME-IT-SPEC-001/002");
                ui.separator();
                ui.label(format!("Paint: {generation}"));
                ui.separator();
                ui.label(format!("IME range callbacks: {callback_count}"));
                ui.separator();
                ui.label(format!("Selected: {:?}", selected));
                ui.separator();
                ui.label(format!("Bounds: {bounds}"));
                ui.separator();
                ui.label(format!("Last IME: {}", self.last_ime_status));
            });
        });

        self.current_click = None;
        self.browser_rect = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut viewer = ImeCompositionViewer {
                texture: self.texture.as_ref(),
                click_transfer: &mut self.current_click,
                browser_active: &mut self.browser_active,
                browser_rect: &mut self.browser_rect,
            };
            DockArea::new(&mut self.dock_state).show_inside(ui, &mut viewer);
        });

        if let Some(click) = self.current_click {
            self.runtime.request_click(click);
        }
        self.collect_ime_events(ctx);
        ctx.request_repaint_after(MESSAGE_PUMP_INTERVAL);
    }
}

fn main() -> eframe::Result<()> {
    let cli_args: Vec<String> = std::env::args().collect();
    if is_cef_subprocess(&cli_args) {
        std::process::exit(run_subprocess());
    }

    println!("WV-11-04-02 CEF IME composition probe start");
    println!("Click Browser input, enable OS IME, and type without confirming.");

    let runtime = match ImeCompositionRuntime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("WV-11-04-02 CEF IME composition probe failed: {error}");
            std::process::exit(1);
        }
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("WV-11-04-02 Browser Surface IME Composition Probe")
            .with_inner_size([1000.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "WV-11-04-02 Browser Surface IME Composition Probe",
        native_options,
        Box::new(move |_cc| Ok(Box::new(ImeCompositionEframeApp::new(runtime)))),
    )
}
