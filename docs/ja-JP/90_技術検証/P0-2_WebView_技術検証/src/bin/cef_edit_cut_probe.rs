//! WV-11-04-01 CEF Browser Surface Cut Probe。
//!
//! 役割:
//! - EDIT-IT-SPEC-005 の Cut 操作を検証する。
//! - Browser Surface 内の input 要素を Pointer Button でフォーカスし、選択済み文字列に対する
//!   Ctrl+X を CEF OSR Browser へ転送する。
//! - Cut 後に Browser 側から選択文字列が削除され、OS Clipboard へ反映されることを確認する。
//!
//! 注意点:
//! - 本ファイルは技術検証用であり、正式 Surface API ではない。
//! - Windows Virtual-Key Code と CEF EVENTFLAG_CONTROL_DOWN の使用は本 Probe の検証実装である。
//! - Windows では CEF へ渡す native_key_code に Win32 の lParam 相当値を設定する。
//! - OS 差異をアプリ利用側へ公開する正式設計は本検証では定義しない。
//! - Windows では CEF の multi-threaded message loop を使用する。
//! - egui では Ctrl+X が Cut Event と Key Event の両方として現れる場合があるため、
//!   Cut Event を優先し、次の X key release Event を明示的に一度だけ破棄する。

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
const TEST_TEXT: &str = "CUT-ME-12345";
const TEST_URL: &str = "data:text/html,<html><body style='margin:0;background:rgb(26,36,50);color:white;font-family:sans-serif;height:100vh;display:flex;align-items:center;justify-content:center'><div style='text-align:center;width:92%'><h1 style='font-size:38px'>WV-11-04-01 CUT</h1><div style='font-size:21px;margin:16px'>Click the input. The full text will be selected. Then press Ctrl+X.</div><input id='probe' type='text' value='CUT-ME-12345' style='width:82%;height:88px;font-size:38px;padding:12px;border:6px solid white;background:rgb(55,78,105);color:white;box-sizing:border-box'><div id='status' style='font-size:28px;margin-top:22px'>SELECTION: waiting</div><div id='value' style='font-size:24px;margin-top:12px'>VALUE: CUT-ME-12345</div><div id='cut' style='font-size:24px;margin-top:12px'>CUT EVENT: waiting</div></div><script>let p=document.getElementById('probe');let update=function(){let s=p.selectionStart||0;let e=p.selectionEnd||0;document.getElementById('status').textContent='SELECTION: '+s+' - '+e;document.getElementById('value').textContent='VALUE: '+(p.value||'<empty>');};p.addEventListener('click',function(){p.select();update();});p.addEventListener('select',update);p.addEventListener('input',update);p.addEventListener('cut',function(){document.getElementById('cut').textContent='CUT EVENT: fired';setTimeout(update,0);});</script></body></html>";

/// 最新 CEF Paint を保持する共有状態。
///
/// @hldocs.ref doc-20260912-104800Z-WV15#sec_v1r5m8k2d7pa
#[derive(Debug, Default)]
struct CutState {
    width: i32,
    height: i32,
    rgba: Vec<u8>,
    generation: u64,
}

#[derive(Clone)]
struct CutRenderHandler {
    state: Arc<Mutex<CutState>>,
}

wrap_render_handler! {
    struct CutRenderHandlerBuilder {
        handler: CutRenderHandler,
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

impl CutRenderHandlerBuilder {
    fn build(state: Arc<Mutex<CutState>>) -> RenderHandler {
        Self::new(CutRenderHandler { state })
    }
}

#[derive(Clone)]
struct CutLifeSpanHandler {
    browser: Arc<Mutex<Option<Browser>>>,
    browser_created: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
}

wrap_life_span_handler! {
    struct CutLifeSpanHandlerBuilder {
        handler: CutLifeSpanHandler,
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

impl CutLifeSpanHandlerBuilder {
    fn build(
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
    ) -> LifeSpanHandler {
        Self::new(CutLifeSpanHandler {
            browser,
            browser_created,
            closed,
        })
    }
}

wrap_client! {
    struct CutClientBuilder {
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

impl CutClientBuilder {
    fn build(
        state: Arc<Mutex<CutState>>,
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
    ) -> Client {
        Self::new(
            CutRenderHandlerBuilder::build(state),
            CutLifeSpanHandlerBuilder::build(browser, browser_created, closed),
        )
    }
}

#[derive(Clone)]
struct CutBrowserProcessHandler {
    state: Arc<Mutex<CutState>>,
    browser: Arc<Mutex<Option<Browser>>>,
    browser_created: Arc<AtomicBool>,
    browser_create_failed: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
}

wrap_browser_process_handler! {
    struct CutBrowserProcessHandlerBuilder {
        handler: CutBrowserProcessHandler,
    }

    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            let mut client = CutClientBuilder::build(
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

impl CutBrowserProcessHandlerBuilder {
    fn build(handler: CutBrowserProcessHandler) -> BrowserProcessHandler {
        Self::new(handler)
    }
}

#[derive(Clone)]
struct CutCefApp {
    browser_process_handler: BrowserProcessHandler,
}

wrap_app! {
    struct CutCefAppBuilder {
        handler: CutCefApp,
    }

    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(self.handler.browser_process_handler.clone())
        }
    }
}

impl CutCefAppBuilder {
    fn build(handler: CutBrowserProcessHandler) -> App {
        Self::new(CutCefApp {
            browser_process_handler: CutBrowserProcessHandlerBuilder::build(handler),
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
        eprintln!("WV-11-04-01 cut subprocess failed: cef_execute_process returned {exit_code}");
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

#[derive(Clone, Copy, Debug)]
struct CutKeyTransfer {
    pressed: bool,
}

#[derive(Clone)]
enum CutTab {
    Browser,
}

struct CutViewer<'a> {
    texture: Option<&'a egui::TextureHandle>,
    click_transfer: &'a mut Option<ClickTransfer>,
    browser_active: &'a mut bool,
}

impl<'a> TabViewer for CutViewer<'a> {
    type Tab = CutTab;

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn title(&mut self, _tab: &mut Self::Tab) -> egui::WidgetText {
        "Browser Surface Cut".into()
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

struct CutRuntime {
    state: Arc<Mutex<CutState>>,
    browser: Arc<Mutex<Option<Browser>>>,
    closed: Arc<AtomicBool>,
}

impl CutRuntime {
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

        let state = Arc::new(Mutex::new(CutState::default()));
        let browser = Arc::new(Mutex::new(None));
        let browser_created = Arc::new(AtomicBool::new(false));
        let browser_create_failed = Arc::new(AtomicBool::new(false));
        let closed = Arc::new(AtomicBool::new(false));

        let handler = CutBrowserProcessHandler {
            state: state.clone(),
            browser: browser.clone(),
            browser_created: browser_created.clone(),
            browser_create_failed: browser_create_failed.clone(),
            closed: closed.clone(),
        };
        let mut app = CutCefAppBuilder::build(handler);

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

    fn request_cut_key(&self, transfer: CutKeyTransfer) -> bool {
        let Some(browser) = self.browser.lock().ok().and_then(|slot| slot.as_ref().cloned()) else {
            return false;
        };
        #[cfg(target_os = "windows")]
        {
            let mut task = CutKeyTaskBuilder::build(browser, transfer);
            return post_task(ThreadId::UI, Some(&mut task)) == 1;
        }
        #[cfg(not(target_os = "windows"))]
        {
            send_cut_key(&browser, transfer);
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
    println!("CEF cut focus click sent: x={}, y={}", transfer.x, transfer.y);
}

/// Windows の X key に対応する native_key_code を返す。
///
/// # 引数
/// - `pressed`: key-down の場合 `true`、key-up の場合 `false`。
///
/// # 戻り値
/// - Win32 の WM_KEYDOWN / WM_KEYUP で X key を受けた場合の lParam 相当値。
#[cfg(target_os = "windows")]
fn native_key_code_x(pressed: bool) -> i32 {
    const SCAN_CODE_X: i32 = 0x2D;
    let mut value = 1 | (SCAN_CODE_X << 16);
    if !pressed {
        value |= 1 << 30;
        value |= 1u32.wrapping_shl(31) as i32;
    }
    value
}

#[cfg(not(target_os = "windows"))]
fn native_key_code_x(_pressed: bool) -> i32 {
    0x58
}

/// CEF BrowserHost へ Ctrl+X を送信する。
///
/// @hldocs.ref doc-20260912-104800Z-WV15#sec_v1r5m8k2d7pa
fn send_cut_key(browser: &Browser, transfer: CutKeyTransfer) {
    let Some(host) = browser.host() else {
        return;
    };

    let event = KeyEvent {
        type_: if transfer.pressed {
            KeyEventType::RAWKEYDOWN
        } else {
            KeyEventType::KEYUP
        },
        windows_key_code: 0x58,
        native_key_code: native_key_code_x(transfer.pressed),
        modifiers: CEF_EVENTFLAG_CONTROL_DOWN,
        ..Default::default()
    };
    host.send_key_event(Some(&event));
    println!(
        "CEF cut key sent: {} key=X vk=88 native={} ctrl=true",
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
struct CutKeyTask {
    browser: Browser,
    transfer: CutKeyTransfer,
}

wrap_task! {
    struct CutKeyTaskBuilder {
        task: CutKeyTask,
    }

    impl Task {
        fn execute(&self) {
            send_cut_key(&self.task.browser, self.task.transfer);
        }
    }
}

impl CutKeyTaskBuilder {
    fn build(browser: Browser, transfer: CutKeyTransfer) -> Task {
        Self::new(CutKeyTask { browser, transfer })
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

impl Drop for CutRuntime {
    fn drop(&mut self) {
        let browser = self.browser.lock().ok().and_then(|slot| slot.as_ref().cloned());
        if let Some(browser) = browser {
            #[cfg(target_os = "windows")]
            {
                let mut task = CloseBrowserTaskBuilder::build(browser);
                if post_task(ThreadId::UI, Some(&mut task)) != 1 {
                    eprintln!("WV-11-04-01 cut probe: failed to post browser close task");
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
            eprintln!("WV-11-04-01 cut probe: browser close timeout");
        }
        shutdown();
    }
}

struct CutEframeApp {
    dock_state: DockState<CutTab>,
    runtime: CutRuntime,
    texture: Option<egui::TextureHandle>,
    applied_generation: u64,
    browser_active: bool,
    current_click: Option<ClickTransfer>,
    last_key_status: String,
    discard_next_cut_key_release: bool,
}

impl CutEframeApp {
    fn new(runtime: CutRuntime) -> Self {
        Self {
            dock_state: DockState::new(vec![CutTab::Browser]),
            runtime,
            texture: None,
            applied_generation: 0,
            browser_active: false,
            current_click: None,
            last_key_status: "waiting".to_string(),
            discard_next_cut_key_release: false,
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
                    "wv11_04_01_cut_probe",
                    image,
                    egui::TextureOptions::LINEAR,
                ));
            }
        }
        self.applied_generation = generation;
    }

    /// egui の Cut / Ctrl+X Event を CEF へ転送する。
    ///
    /// `Event::Cut` では Ctrl+X DOWN/UP を一組送信する。
    /// その後 egui が同じ物理操作の X key release を通知する場合があるため、
    /// 次の X key release Event を一度だけ破棄する。
    fn collect_cut_key(&mut self, ctx: &egui::Context) {
        if !self.browser_active {
            return;
        }

        let events = ctx.input(|input| input.events.clone());
        for event in events {
            match event {
                egui::Event::Cut => {
                    if self.runtime.request_cut_key(CutKeyTransfer { pressed: true }) {
                        self.last_key_status = "DOWN Ctrl+X (Cut Event)".to_string();
                    }
                    if self.runtime.request_cut_key(CutKeyTransfer { pressed: false }) {
                        self.last_key_status = "UP Ctrl+X (Cut Event)".to_string();
                    }
                    self.discard_next_cut_key_release = true;
                }
                egui::Event::Key {
                    key: egui::Key::X,
                    pressed,
                    modifiers,
                    ..
                } if modifiers.ctrl => {
                    if !pressed && self.discard_next_cut_key_release {
                        self.discard_next_cut_key_release = false;
                        continue;
                    }
                    if self.runtime.request_cut_key(CutKeyTransfer { pressed }) {
                        self.last_key_status = format!(
                            "{} Ctrl+X (Key Event)",
                            if pressed { "DOWN" } else { "UP" }
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

impl eframe::App for CutEframeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.runtime.pump();
        self.update_texture(ctx);
        let generation = self
            .runtime
            .state
            .lock()
            .map(|state| state.generation)
            .unwrap_or(0);

        egui::TopBottomPanel::top("wv11_04_01_cut_status").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("WV-11-04-01 EDIT-IT-SPEC-005");
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
            let mut viewer = CutViewer {
                texture: self.texture.as_ref(),
                click_transfer: &mut self.current_click,
                browser_active: &mut self.browser_active,
            };
            DockArea::new(&mut self.dock_state).show_inside(ui, &mut viewer);
        });

        if let Some(click) = self.current_click {
            self.runtime.request_click(click);
        }
        self.collect_cut_key(ctx);

        ctx.request_repaint_after(MESSAGE_PUMP_INTERVAL);
    }
}

fn main() -> eframe::Result<()> {
    let cli_args: Vec<String> = std::env::args().collect();
    if is_cef_subprocess(&cli_args) {
        std::process::exit(run_subprocess());
    }

    println!("WV-11-04-01 CEF cut probe start");
    println!("Expected OS Clipboard after Ctrl+X: {TEST_TEXT}");
    let runtime = match CutRuntime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("WV-11-04-01 CEF cut probe failed: {error}");
            std::process::exit(1);
        }
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("WV-11-04-01 Browser Surface Cut Probe")
            .with_inner_size([1000.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "WV-11-04-01 Browser Surface Cut Probe",
        native_options,
        Box::new(move |_cc| Ok(Box::new(CutEframeApp::new(runtime)))),
    )
}
