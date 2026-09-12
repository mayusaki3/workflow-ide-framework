//! WV-11-04-01 CEF Browser Surface Copy Probe。
//!
//! 役割:
//! - EDIT-IT-SPEC-003 の Copy 操作を検証する。
//! - Browser Surface 内の input 要素を Pointer Button でフォーカスし、選択済み文字列に対する
//!   Ctrl+C を CEF OSR Browser へ転送する。
//! - Copy 後の OS Clipboard 内容は外部から確認する。
//!
//! 注意点:
//! - 本ファイルは技術検証用であり、正式 Surface API ではない。
//! - Windows Virtual-Key Code と CEF EVENTFLAG_CONTROL_DOWN の使用は本 Probe の検証実装である。
//! - Windows では CEF へ渡す native_key_code に Win32 の lParam 相当値を設定する。
//! - OS 差異をアプリ利用側へ公開する正式設計は本検証では定義しない。
//! - Windows では CEF の multi-threaded message loop を使用する。
//! - egui では Ctrl+C が Copy Event として消費される場合があるため、Copy Event と Key Event の双方を監視する。

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
const CEF_EVENTFLAG_CONTROL_DOWN: u32 = 1 << 2;
const TEST_TEXT: &str = "COPY-ME-12345";
const TEST_URL: &str = "data:text/html,<html><body style='margin:0;background:rgb(26,36,50);color:white;font-family:sans-serif;height:100vh;display:flex;align-items:center;justify-content:center'><div style='text-align:center;width:92%'><h1 style='font-size:38px'>WV-11-04-01 COPY</h1><div style='font-size:21px;margin:16px'>Click the input. The full text will be selected. Then press Ctrl+C.</div><input id='probe' type='text' value='COPY-ME-12345' style='width:82%;height:88px;font-size:38px;padding:12px;border:6px solid white;background:rgb(55,78,105);color:white;box-sizing:border-box'><div id='status' style='font-size:28px;margin-top:22px'>SELECTION: waiting</div><div id='selected' style='font-size:24px;margin-top:12px'>TEXT: none</div><div id='copy' style='font-size:24px;margin-top:12px'>COPY EVENT: waiting</div></div><script>let p=document.getElementById('probe');let update=function(){let s=p.selectionStart||0;let e=p.selectionEnd||0;document.getElementById('status').textContent='SELECTION: '+s+' - '+e;document.getElementById('selected').textContent='TEXT: '+(p.value.substring(s,e)||'none');};p.addEventListener('click',function(){p.select();update();});p.addEventListener('select',update);p.addEventListener('copy',function(){document.getElementById('copy').textContent='COPY EVENT: fired';});</script></body></html>";

/// 最新 CEF Paint を保持する共有状態。
///
/// @hldocs.ref doc-20260912-104800Z-WV15#sec_m2k6a8d1v4qs
#[derive(Debug, Default)]
struct CopyState {
    width: i32,
    height: i32,
    rgba: Vec<u8>,
    generation: u64,
}

#[derive(Clone)]
struct CopyRenderHandler {
    state: Arc<Mutex<CopyState>>,
}

wrap_render_handler! {
    struct CopyRenderHandlerBuilder {
        handler: CopyRenderHandler,
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
    }
}

impl CopyRenderHandlerBuilder {
    fn build(state: Arc<Mutex<CopyState>>) -> RenderHandler {
        Self::new(CopyRenderHandler { state })
    }
}

#[derive(Clone)]
struct CopyLifeSpanHandler {
    browser: Arc<Mutex<Option<Browser>>>,
    browser_created: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
}

wrap_life_span_handler! {
    struct CopyLifeSpanHandlerBuilder {
        handler: CopyLifeSpanHandler,
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

impl CopyLifeSpanHandlerBuilder {
    fn build(
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
    ) -> LifeSpanHandler {
        Self::new(CopyLifeSpanHandler {
            browser,
            browser_created,
            closed,
        })
    }
}

wrap_client! {
    struct CopyClientBuilder {
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

impl CopyClientBuilder {
    fn build(
        state: Arc<Mutex<CopyState>>,
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
    ) -> Client {
        Self::new(
            CopyRenderHandlerBuilder::build(state),
            CopyLifeSpanHandlerBuilder::build(browser, browser_created, closed),
        )
    }
}

#[derive(Clone)]
struct CopyBrowserProcessHandler {
    state: Arc<Mutex<CopyState>>,
    browser: Arc<Mutex<Option<Browser>>>,
    browser_created: Arc<AtomicBool>,
    browser_create_failed: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
}

wrap_browser_process_handler! {
    struct CopyBrowserProcessHandlerBuilder {
        handler: CopyBrowserProcessHandler,
    }

    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            let mut client = CopyClientBuilder::build(
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

impl CopyBrowserProcessHandlerBuilder {
    fn build(handler: CopyBrowserProcessHandler) -> BrowserProcessHandler {
        Self::new(handler)
    }
}

#[derive(Clone)]
struct CopyCefApp {
    browser_process_handler: BrowserProcessHandler,
}

wrap_app! {
    struct CopyCefAppBuilder {
        handler: CopyCefApp,
    }

    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(self.handler.browser_process_handler.clone())
        }
    }
}

impl CopyCefAppBuilder {
    fn build(handler: CopyBrowserProcessHandler) -> App {
        Self::new(CopyCefApp {
            browser_process_handler: CopyBrowserProcessHandlerBuilder::build(handler),
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
    if exit_code >= 0 {
        exit_code
    } else {
        eprintln!("WV-11-04-01 copy subprocess failed: cef_execute_process returned {exit_code}");
        1
    }
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
struct ClickTransfer {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy, Debug)]
struct CopyKeyTransfer {
    pressed: bool,
}

#[derive(Clone)]
enum CopyTab {
    Browser,
}

struct CopyViewer<'a> {
    texture: Option<&'a egui::TextureHandle>,
    click_transfer: &'a mut Option<ClickTransfer>,
    browser_active: &'a mut bool,
}

impl<'a> TabViewer for CopyViewer<'a> {
    type Tab = CopyTab;

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn title(&mut self, _tab: &mut Self::Tab) -> egui::WidgetText {
        "Browser Surface Copy".into()
    }

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

struct CopyRuntime {
    state: Arc<Mutex<CopyState>>,
    browser: Arc<Mutex<Option<Browser>>>,
    closed: Arc<AtomicBool>,
}

impl CopyRuntime {
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

        let state = Arc::new(Mutex::new(CopyState::default()));
        let browser = Arc::new(Mutex::new(None));
        let browser_created = Arc::new(AtomicBool::new(false));
        let browser_create_failed = Arc::new(AtomicBool::new(false));
        let closed = Arc::new(AtomicBool::new(false));

        let handler = CopyBrowserProcessHandler {
            state: state.clone(),
            browser: browser.clone(),
            browser_created: browser_created.clone(),
            browser_create_failed: browser_create_failed.clone(),
            closed: closed.clone(),
        };
        let mut app = CopyCefAppBuilder::build(handler);

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
        let Some(browser) = self.browser.lock().ok().and_then(|slot| slot.as_ref().cloned()) else {
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

    fn request_copy_key(&self, transfer: CopyKeyTransfer) -> bool {
        let Some(browser) = self.browser.lock().ok().and_then(|slot| slot.as_ref().cloned()) else {
            return false;
        };
        #[cfg(target_os = "windows")]
        {
            let mut task = CopyKeyTaskBuilder::build(browser, transfer);
            return post_task(ThreadId::UI, Some(&mut task)) == 1;
        }
        #[cfg(not(target_os = "windows"))]
        {
            send_copy_key(&browser, transfer);
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
    println!("CEF copy focus click sent: x={}, y={}", transfer.x, transfer.y);
}

/// Windows の C key に対応する native_key_code を返す。
///
/// # 引数
/// - `pressed`: key-down の場合 `true`、key-up の場合 `false`。
///
/// # 戻り値
/// - Win32 の WM_KEYDOWN / WM_KEYUP で C key を受けた場合の lParam 相当値。
#[cfg(target_os = "windows")]
fn native_key_code_c(pressed: bool) -> i32 {
    const SCAN_CODE_C: i32 = 0x2E;
    let mut value = 1 | (SCAN_CODE_C << 16);
    if !pressed {
        value |= 1 << 30;
        value |= 1u32.wrapping_shl(31) as i32;
    }
    value
}

#[cfg(not(target_os = "windows"))]
fn native_key_code_c(_pressed: bool) -> i32 {
    0x43
}

/// CEF BrowserHost へ Ctrl+C を送信する。
///
/// @hldocs.ref doc-20260912-104800Z-WV15#sec_m2k6a8d1v4qs
fn send_copy_key(browser: &Browser, transfer: CopyKeyTransfer) {
    let Some(host) = browser.host() else {
        return;
    };

    let event = KeyEvent {
        type_: if transfer.pressed {
            KeyEventType::RAWKEYDOWN
        } else {
            KeyEventType::KEYUP
        },
        windows_key_code: 0x43,
        native_key_code: native_key_code_c(transfer.pressed),
        modifiers: CEF_EVENTFLAG_CONTROL_DOWN,
        ..Default::default()
    };
    host.send_key_event(Some(&event));
    println!(
        "CEF copy key sent: {} key=C vk=67 native={} ctrl=true",
        if transfer.pressed { "DOWN" } else { "UP" },
        event.native_key_code
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
struct CopyKeyTask {
    browser: Browser,
    transfer: CopyKeyTransfer,
}

wrap_task! {
    struct CopyKeyTaskBuilder {
        task: CopyKeyTask,
    }

    impl Task {
        fn execute(&self) {
            send_copy_key(&self.task.browser, self.task.transfer);
        }
    }
}

impl CopyKeyTaskBuilder {
    fn build(browser: Browser, transfer: CopyKeyTransfer) -> Task {
        Self::new(CopyKeyTask { browser, transfer })
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

impl Drop for CopyRuntime {
    fn drop(&mut self) {
        let browser = self.browser.lock().ok().and_then(|slot| slot.as_ref().cloned());
        if let Some(browser) = browser {
            #[cfg(target_os = "windows")]
            {
                let mut task = CloseBrowserTaskBuilder::build(browser);
                if post_task(ThreadId::UI, Some(&mut task)) != 1 {
                    eprintln!("WV-11-04-01 copy probe: failed to post browser close task");
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
            eprintln!("WV-11-04-01 copy probe: browser close timeout");
        }
        shutdown();
    }
}

struct CopyEframeApp {
    dock_state: DockState<CopyTab>,
    runtime: CopyRuntime,
    texture: Option<egui::TextureHandle>,
    applied_generation: u64,
    browser_active: bool,
    current_click: Option<ClickTransfer>,
    last_key_status: String,
}

impl CopyEframeApp {
    fn new(runtime: CopyRuntime) -> Self {
        Self {
            dock_state: DockState::new(vec![CopyTab::Browser]),
            runtime,
            texture: None,
            applied_generation: 0,
            browser_active: false,
            current_click: None,
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
                    "wv11_04_01_copy_probe",
                    image,
                    egui::TextureOptions::LINEAR,
                ));
            }
        }
        self.applied_generation = generation;
    }

    fn collect_copy_key(&mut self, ctx: &egui::Context) {
        if !self.browser_active {
            return;
        }

        let events = ctx.input(|input| input.events.clone());
        let mut copy_event_seen = false;
        for event in events {
            match event {
                egui::Event::Copy => {
                    copy_event_seen = true;
                    if self.runtime.request_copy_key(CopyKeyTransfer { pressed: true }) {
                        self.last_key_status = "DOWN Ctrl+C (Copy Event)".to_string();
                    }
                    if self.runtime.request_copy_key(CopyKeyTransfer { pressed: false }) {
                        self.last_key_status = "UP Ctrl+C (Copy Event)".to_string();
                    }
                }
                egui::Event::Key {
                    key: egui::Key::C,
                    pressed,
                    modifiers,
                    ..
                } if modifiers.ctrl && !copy_event_seen => {
                    if self.runtime.request_copy_key(CopyKeyTransfer { pressed }) {
                        self.last_key_status = format!(
                            "{} Ctrl+C (Key Event)",
                            if pressed { "DOWN" } else { "UP" }
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

impl eframe::App for CopyEframeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.runtime.pump();
        self.update_texture(ctx);
        let generation = self
            .runtime
            .state
            .lock()
            .map(|state| state.generation)
            .unwrap_or(0);

        egui::TopBottomPanel::top("wv11_04_01_copy_status").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("WV-11-04-01 EDIT-IT-SPEC-003");
                ui.separator();
                ui.label(format!("Paint generation: {generation}"));
                ui.separator();
                ui.label(format!(
                    "Browser input: {}",
                    if self.browser_active { "ACTIVE" } else { "click text first" }
                ));
                ui.separator();
                ui.label(format!("Last key: {}", self.last_key_status));
                ui.separator();
                ui.label(format!("Expected clipboard: {TEST_TEXT}"));
            });
        });

        self.current_click = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut viewer = CopyViewer {
                texture: self.texture.as_ref(),
                click_transfer: &mut self.current_click,
                browser_active: &mut self.browser_active,
            };
            DockArea::new(&mut self.dock_state).show_inside(ui, &mut viewer);
        });

        if let Some(click) = self.current_click {
            self.runtime.request_click(click);
        }
        self.collect_copy_key(ctx);

        ctx.request_repaint_after(MESSAGE_PUMP_INTERVAL);
    }
}

fn main() -> eframe::Result<()> {
    let cli_args: Vec<String> = std::env::args().collect();
    if is_cef_subprocess(&cli_args) {
        std::process::exit(run_subprocess());
    }

    println!("WV-11-04-01 CEF copy probe start");
    println!("Expected OS Clipboard after Ctrl+C: {TEST_TEXT}");
    let runtime = match CopyRuntime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("WV-11-04-01 CEF copy probe failed: {error}");
            std::process::exit(1);
        }
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("WV-11-04-01 Browser Surface Copy Probe")
            .with_inner_size([1000.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "WV-11-04-01 Browser Surface Copy Probe",
        native_options,
        Box::new(move |_cc| Ok(Box::new(CopyEframeApp::new(runtime)))),
    )
}
