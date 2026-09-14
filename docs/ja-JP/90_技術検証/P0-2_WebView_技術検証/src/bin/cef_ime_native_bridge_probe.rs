// WV-11-04-02 Windows native IME -> CEF bridge Probe。
//
// 役割:
// - eframe/winit の Win32 HWND で WM_IME_* を観測する。
// - HIMC から取得した GCS_COMPSTR を CEF OSR の ime_set_composition へ転送する。
// - egui::ImeEvent::Preedit を経由せず、Windows native IME 情報から Browser 側の
//   compositionstart / compositionupdate が成立するかを切り分ける。
// - GCS_RESULTSTR も観測し、後続 Commit 検証へ接続できることを確認する。
//
// 注意点:
// - 本ファイルは技術検証用であり、正式 Surface API ではない。
// - GCS_RESULTSTR は本 Probe では記録のみ行い、CEF へ Commit しない。
// - Candidate Window の位置同期は本 Probe の判定対象外。

use cef::*;
use eframe::egui;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::collections::VecDeque;
use std::ptr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::sleep;
use std::time::{Duration, Instant};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::Ime::{
    ImmGetCompositionStringW, ImmGetContext, ImmGetOpenStatus, ImmReleaseContext,
    IME_COMPOSITION_STRING, GCS_COMPSTR, GCS_CURSORPOS, GCS_RESULTSTR,
};
use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{
    WM_IME_COMPOSITION, WM_IME_ENDCOMPOSITION, WM_IME_SETCONTEXT, WM_IME_STARTCOMPOSITION,
};

const VIEW_WIDTH: i32 = 800;
const VIEW_HEIGHT: i32 = 600;
const MESSAGE_PUMP_INTERVAL: Duration = Duration::from_millis(10);
const BROWSER_CREATE_TIMEOUT: Duration = Duration::from_secs(3);
const CLOSE_TIMEOUT: Duration = Duration::from_secs(3);
const SUBCLASS_ID: usize = 0x5749_4D45;
const TEST_URL: &str = "data:text/html,<html><body style='margin:0;background:rgb(26,36,50);color:white;font-family:sans-serif;height:100vh;display:flex;align-items:center;justify-content:center'><div style='text-align:center;width:92%'><h1 style='font-size:36px'>WV-11-04-02 NATIVE IME BRIDGE</h1><div style='font-size:20px;margin:16px'>Click input, enable Japanese IME, and type without confirming.</div><input id='probe' type='text' value='' style='width:82%;height:88px;font-size:38px;padding:12px;border:6px solid white;background:rgb(55,78,105);color:white;box-sizing:border-box'><div id='start' style='font-size:24px;margin-top:18px'>COMPOSITION START: waiting</div><div id='update' style='font-size:24px;margin-top:10px'>COMPOSITION TEXT: none</div><div id='value' style='font-size:24px;margin-top:10px'>VALUE: empty</div></div><script>let p=document.getElementById('probe');let s=document.getElementById('start');let u=document.getElementById('update');let v=document.getElementById('value');p.addEventListener('compositionstart',()=>s.textContent='COMPOSITION START: fired');p.addEventListener('compositionupdate',e=>u.textContent='COMPOSITION TEXT: '+(e.data||'<empty>'));p.addEventListener('input',()=>v.textContent='VALUE: '+(p.value||'<empty>'));</script></body></html>";

#[derive(Debug, Default)]
struct ProbeState {
    width: i32,
    height: i32,
    rgba: Vec<u8>,
    generation: u64,
    ime_range_callbacks: u64,
    selected_range: Option<Range>,
    character_bounds: Vec<Rect>,
}

#[derive(Clone)]
struct ProbeRenderHandler {
    state: Arc<Mutex<ProbeState>>,
}

wrap_render_handler! {
    struct ProbeRenderHandlerBuilder {
        handler: ProbeRenderHandler,
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
            if let Ok(mut state) = self.handler.state.lock() {
                state.width = width;
                state.height = height;
                state.rgba = rgba;
                state.generation = state.generation.saturating_add(1);
            }
        }

        fn on_ime_composition_range_changed(
            &self,
            _browser: Option<&mut Browser>,
            selected_range: Option<&Range>,
            character_bounds: Option<&[Rect]>,
        ) {
            if let Ok(mut state) = self.handler.state.lock() {
                state.selected_range = selected_range.cloned();
                state.character_bounds = character_bounds.unwrap_or_default().to_vec();
                state.ime_range_callbacks = state.ime_range_callbacks.saturating_add(1);
                println!(
                    "CEF IME range changed: selected={:?} bounds={} callback_count={}",
                    state.selected_range,
                    state.character_bounds.len(),
                    state.ime_range_callbacks
                );
            }
        }
    }
}

impl ProbeRenderHandlerBuilder {
    fn build(state: Arc<Mutex<ProbeState>>) -> RenderHandler {
        Self::new(ProbeRenderHandler { state })
    }
}

#[derive(Clone)]
struct ProbeLifeSpanHandler {
    browser: Arc<Mutex<Option<Browser>>>,
    browser_created: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
}

wrap_life_span_handler! {
    struct ProbeLifeSpanHandlerBuilder {
        handler: ProbeLifeSpanHandler,
    }

    impl LifeSpanHandler {
        fn on_after_created(&self, browser: Option<&mut Browser>) {
            let Some(browser) = browser.cloned() else { return; };
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

impl ProbeLifeSpanHandlerBuilder {
    fn build(
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
    ) -> LifeSpanHandler {
        Self::new(ProbeLifeSpanHandler {
            browser,
            browser_created,
            closed,
        })
    }
}

wrap_client! {
    struct ProbeClientBuilder {
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

impl ProbeClientBuilder {
    fn build(
        state: Arc<Mutex<ProbeState>>,
        browser: Arc<Mutex<Option<Browser>>>,
        browser_created: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
    ) -> Client {
        Self::new(
            ProbeRenderHandlerBuilder::build(state),
            ProbeLifeSpanHandlerBuilder::build(browser, browser_created, closed),
        )
    }
}

#[derive(Clone)]
struct ProbeBrowserProcessHandler {
    state: Arc<Mutex<ProbeState>>,
    browser: Arc<Mutex<Option<Browser>>>,
    browser_created: Arc<AtomicBool>,
    browser_create_failed: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
}

wrap_browser_process_handler! {
    struct ProbeBrowserProcessHandlerBuilder {
        handler: ProbeBrowserProcessHandler,
    }

    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            let mut client = ProbeClientBuilder::build(
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

impl ProbeBrowserProcessHandlerBuilder {
    fn build(handler: ProbeBrowserProcessHandler) -> BrowserProcessHandler {
        Self::new(handler)
    }
}

#[derive(Clone)]
struct ProbeCefApp {
    browser_process_handler: BrowserProcessHandler,
}

wrap_app! {
    struct ProbeCefAppBuilder {
        handler: ProbeCefApp,
    }

    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(self.handler.browser_process_handler.clone())
        }
    }
}

impl ProbeCefAppBuilder {
    fn build(handler: ProbeBrowserProcessHandler) -> App {
        Self::new(ProbeCefApp {
            browser_process_handler: ProbeBrowserProcessHandlerBuilder::build(handler),
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

#[derive(Clone, Copy, Debug)]
struct ClickTransfer {
    x: i32,
    y: i32,
}

#[derive(Clone, Debug)]
struct NativeCompositionTransfer {
    text: String,
    cursor_pos: i32,
}

struct ProbeRuntime {
    state: Arc<Mutex<ProbeState>>,
    browser: Arc<Mutex<Option<Browser>>>,
    closed: Arc<AtomicBool>,
}

impl ProbeRuntime {
    fn new() -> Result<Self, String> {
        let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
        let args = args::Args::new();
        let process_result = execute_process(Some(args.as_main_args()), None, ptr::null_mut());
        if process_result >= 0 {
            return Err(format!("Unexpected subprocess result in browser process: {process_result}"));
        }

        let settings = Settings {
            windowless_rendering_enabled: 1,
            no_sandbox: 1,
            multi_threaded_message_loop: 1,
            ..Default::default()
        };

        let state = Arc::new(Mutex::new(ProbeState::default()));
        let browser = Arc::new(Mutex::new(None));
        let browser_created = Arc::new(AtomicBool::new(false));
        let browser_create_failed = Arc::new(AtomicBool::new(false));
        let closed = Arc::new(AtomicBool::new(false));

        let handler = ProbeBrowserProcessHandler {
            state: state.clone(),
            browser: browser.clone(),
            browser_created: browser_created.clone(),
            browser_create_failed: browser_create_failed.clone(),
            closed: closed.clone(),
        };
        let mut app = ProbeCefAppBuilder::build(handler);
        let initialized = initialize(
            Some(args.as_main_args()),
            Some(&settings),
            Some(&mut app),
            ptr::null_mut(),
        );
        if initialized != 1 {
            return Err(format!("cef_initialize returned {initialized}"));
        }

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

        Ok(Self { state, browser, closed })
    }

    fn request_click(&self, transfer: ClickTransfer) -> bool {
        let Some(browser) = self.browser.lock().ok().and_then(|slot| slot.as_ref().cloned()) else {
            return false;
        };
        let mut task = ClickTaskBuilder::build(browser, transfer);
        post_task(ThreadId::UI, Some(&mut task)) == 1
    }

    fn request_composition(&self, transfer: NativeCompositionTransfer) -> bool {
        let Some(browser) = self.browser.lock().ok().and_then(|slot| slot.as_ref().cloned()) else {
            return false;
        };
        let mut task = CompositionTaskBuilder::build(browser, transfer);
        post_task(ThreadId::UI, Some(&mut task)) == 1
    }
}

fn send_click(browser: &Browser, transfer: ClickTransfer) {
    let Some(host) = browser.host() else { return; };
    host.set_focus(true.into());
    let event = MouseEvent {
        x: transfer.x,
        y: transfer.y,
        ..Default::default()
    };
    host.send_mouse_move_event(Some(&event), 0);
    host.send_mouse_click_event(Some(&event), MouseButtonType::LEFT, 0, 1);
    host.send_mouse_click_event(Some(&event), MouseButtonType::LEFT, 1, 1);
    println!("CEF Browser focus click sent: x={} y={}", transfer.x, transfer.y);
}

/// Windows native GCS_COMPSTR を CEF OSR Composition へ転送する。
///
/// # 引数
/// - `browser`: 転送先 Browser。
/// - `transfer`: native IME から取得した未確定文字列と cursor 位置。
///
/// # 戻り値
/// - なし。
///
/// @hldocs.ref doc-20260912-104801Z-WV16#sec_b6m1r9p3x7da
fn send_native_composition(browser: &Browser, transfer: &NativeCompositionTransfer) {
    if transfer.text.is_empty() {
        return;
    }
    let Some(host) = browser.host() else { return; };
    let text = CefString::from(transfer.text.as_str());
    let utf16_len = transfer.text.encode_utf16().count() as u32;
    let selection = Range { from: 0, to: utf16_len };
    let underline = CompositionUnderline {
        range: Range { from: 0, to: utf16_len },
        color: 0xFF000000,
        background_color: 0x00000000,
        thick: 0,
        ..Default::default()
    };
    let underlines = [underline];
    host.ime_set_composition(
        Some(&text),
        Some(&underlines),
        None,
        Some(&selection),
    );
    println!(
        "CEF native IME composition set: {:?} cursor={} selection={}..{}",
        transfer.text, transfer.cursor_pos, selection.from, selection.to
    );
}

#[derive(Clone)]
struct ClickTask {
    browser: Browser,
    transfer: ClickTransfer,
}
wrap_task! {
    struct ClickTaskBuilder { task: ClickTask, }
    impl Task { fn execute(&self) { send_click(&self.task.browser, self.task.transfer); } }
}
impl ClickTaskBuilder {
    fn build(browser: Browser, transfer: ClickTransfer) -> Task {
        Self::new(ClickTask { browser, transfer })
    }
}

#[derive(Clone)]
struct CompositionTask {
    browser: Browser,
    transfer: NativeCompositionTransfer,
}
wrap_task! {
    struct CompositionTaskBuilder { task: CompositionTask, }
    impl Task { fn execute(&self) { send_native_composition(&self.task.browser, &self.task.transfer); } }
}
impl CompositionTaskBuilder {
    fn build(browser: Browser, transfer: NativeCompositionTransfer) -> Task {
        Self::new(CompositionTask { browser, transfer })
    }
}

#[derive(Clone)]
struct CloseBrowserTask {
    browser: Browser,
}
wrap_task! {
    struct CloseBrowserTaskBuilder { task: CloseBrowserTask, }
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

impl Drop for ProbeRuntime {
    fn drop(&mut self) {
        let browser = self.browser.lock().ok().and_then(|slot| slot.as_ref().cloned());
        if let Some(browser) = browser {
            let mut task = CloseBrowserTaskBuilder::build(browser);
            if post_task(ThreadId::UI, Some(&mut task)) != 1 {
                eprintln!("native IME bridge probe: failed to post browser close task");
            }
        }
        let started = Instant::now();
        while !self.closed.load(Ordering::Acquire) && started.elapsed() < CLOSE_TIMEOUT {
            sleep(MESSAGE_PUMP_INTERVAL);
        }
        if !self.closed.load(Ordering::Acquire) {
            eprintln!("native IME bridge probe: browser close timeout");
        }
        shutdown();
    }
}

#[derive(Debug, Clone)]
enum NativeImeEvent {
    Start,
    Composition(NativeCompositionTransfer),
    Result(String),
    End,
}

#[derive(Debug, Default)]
struct NativeImeState {
    hwnd: isize,
    setcontext_count: u64,
    start_count: u64,
    composition_count: u64,
    end_count: u64,
    ime_open: bool,
    last_compstr: String,
    last_resultstr: String,
    last_cursor_pos: i32,
    events: VecDeque<NativeImeEvent>,
}

static mut NATIVE_IME_STATE_PTR: *const Mutex<NativeImeState> = std::ptr::null();

/// HIMC から指定された UTF-16 文字列を読み出す。
///
/// # 引数
/// - `hwnd`: 対象 HWND。
/// - `index`: GCS_COMPSTR または GCS_RESULTSTR。
///
/// # 戻り値
/// - 読み出した文字列。取得できない場合は空文字列。
unsafe fn read_ime_string(hwnd: HWND, index: IME_COMPOSITION_STRING) -> String {
    let himc = ImmGetContext(hwnd);
    if himc.0.is_null() {
        return String::new();
    }
    let byte_len = ImmGetCompositionStringW(himc, index, None, 0);
    let text = if byte_len > 0 {
        let units = byte_len as usize / std::mem::size_of::<u16>();
        let mut buffer = vec![0u16; units];
        let copied = ImmGetCompositionStringW(
            himc,
            index,
            Some(buffer.as_mut_ptr().cast()),
            byte_len as u32,
        );
        if copied > 0 {
            let copied_units = copied as usize / std::mem::size_of::<u16>();
            String::from_utf16_lossy(&buffer[..copied_units.min(buffer.len())])
        } else {
            String::new()
        }
    } else {
        String::new()
    };
    let _ = ImmReleaseContext(hwnd, himc);
    text
}

unsafe fn read_cursor_pos(hwnd: HWND) -> i32 {
    let himc = ImmGetContext(hwnd);
    if himc.0.is_null() {
        return -1;
    }
    let cursor = ImmGetCompositionStringW(himc, GCS_CURSORPOS, None, 0);
    let _ = ImmReleaseContext(hwnd, himc);
    cursor
}

unsafe fn read_ime_open(hwnd: HWND) -> bool {
    let himc = ImmGetContext(hwnd);
    if himc.0.is_null() {
        return false;
    }
    let open = ImmGetOpenStatus(himc).as_bool();
    let _ = ImmReleaseContext(hwnd, himc);
    open
}

/// Window subclass で native IME message を観測し、Bridge queue へ積む。
unsafe extern "system" fn native_ime_subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _subclass_id: usize,
    _ref_data: usize,
) -> LRESULT {
    if !NATIVE_IME_STATE_PTR.is_null() {
        let state = &*NATIVE_IME_STATE_PTR;
        if let Ok(mut state) = state.lock() {
            state.hwnd = hwnd.0 as isize;
            match msg {
                WM_IME_SETCONTEXT => {
                    state.setcontext_count = state.setcontext_count.saturating_add(1);
                    println!(
                        "WM_IME_SETCONTEXT count={} wparam=0x{:X} lparam=0x{:X}",
                        state.setcontext_count, wparam.0, lparam.0
                    );
                }
                WM_IME_STARTCOMPOSITION => {
                    state.start_count = state.start_count.saturating_add(1);
                    state.ime_open = read_ime_open(hwnd);
                    state.events.push_back(NativeImeEvent::Start);
                    println!("WM_IME_STARTCOMPOSITION count={} open={}", state.start_count, state.ime_open);
                }
                WM_IME_COMPOSITION => {
                    state.composition_count = state.composition_count.saturating_add(1);
                    state.ime_open = read_ime_open(hwnd);
                    let flags = IME_COMPOSITION_STRING(lparam.0 as u32);
                    if flags.contains(GCS_COMPSTR) {
                        let text = read_ime_string(hwnd, GCS_COMPSTR);
                        let cursor_pos = read_cursor_pos(hwnd);
                        state.last_compstr = text.clone();
                        state.last_cursor_pos = cursor_pos;
                        if !text.is_empty() {
                            state.events.push_back(NativeImeEvent::Composition(
                                NativeCompositionTransfer { text: text.clone(), cursor_pos },
                            ));
                        }
                        println!(
                            "WM_IME_COMPOSITION count={} comp={:?} cursor={} lparam=0x{:X}",
                            state.composition_count, text, cursor_pos, lparam.0
                        );
                    }
                    if flags.contains(GCS_RESULTSTR) {
                        let result = read_ime_string(hwnd, GCS_RESULTSTR);
                        state.last_resultstr = result.clone();
                        if !result.is_empty() {
                            state.events.push_back(NativeImeEvent::Result(result.clone()));
                        }
                        println!("WM_IME_RESULT result={:?}", result);
                    }
                }
                WM_IME_ENDCOMPOSITION => {
                    state.end_count = state.end_count.saturating_add(1);
                    state.events.push_back(NativeImeEvent::End);
                    println!("WM_IME_ENDCOMPOSITION count={}", state.end_count);
                }
                _ => {}
            }
        }
    }
    DefSubclassProc(hwnd, msg, wparam, lparam)
}

fn frame_hwnd(frame: &eframe::Frame) -> Option<HWND> {
    let handle = frame.window_handle().ok()?;
    match handle.as_raw() {
        RawWindowHandle::Win32(win32) => Some(HWND(win32.hwnd.get() as *mut _)),
        _ => None,
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

struct ProbeEframeApp {
    runtime: ProbeRuntime,
    texture: Option<egui::TextureHandle>,
    applied_generation: u64,
    browser_active: bool,
    current_click: Option<ClickTransfer>,
    native_ime: Arc<Mutex<NativeImeState>>,
    installed_hwnd: Option<HWND>,
    last_bridge_status: String,
}

impl ProbeEframeApp {
    fn new(runtime: ProbeRuntime) -> Self {
        Self {
            runtime,
            texture: None,
            applied_generation: 0,
            browser_active: false,
            current_click: None,
            native_ime: Arc::new(Mutex::new(NativeImeState::default())),
            installed_hwnd: None,
            last_bridge_status: "waiting".to_string(),
        }
    }

    fn ensure_subclass(&mut self, frame: &eframe::Frame) {
        if self.installed_hwnd.is_some() {
            return;
        }
        let Some(hwnd) = frame_hwnd(frame) else { return; };
        unsafe {
            NATIVE_IME_STATE_PTR = Arc::as_ptr(&self.native_ime);
            if SetWindowSubclass(hwnd, Some(native_ime_subclass_proc), SUBCLASS_ID, 0).as_bool() {
                self.installed_hwnd = Some(hwnd);
                println!("Native IME bridge subclass installed: hwnd=0x{:X}", hwnd.0 as usize);
            } else {
                eprintln!("Native IME bridge subclass installation failed");
            }
        }
    }

    fn update_texture(&mut self, ctx: &egui::Context) {
        let snapshot = {
            let Ok(state) = self.runtime.state.lock() else { return; };
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
        let Some((generation, width, height, rgba)) = snapshot else { return; };
        let image = egui::ColorImage::from_rgba_unmultiplied([width, height], &rgba);
        match self.texture.as_mut() {
            Some(texture) if texture.size() == [width, height] => {
                texture.set(image, egui::TextureOptions::LINEAR);
            }
            _ => {
                self.texture = Some(ctx.load_texture(
                    "wv11_04_02_native_ime_bridge",
                    image,
                    egui::TextureOptions::LINEAR,
                ));
            }
        }
        self.applied_generation = generation;
    }

    fn drain_native_ime_events(&mut self) {
        if !self.browser_active {
            return;
        }
        let events = self
            .native_ime
            .lock()
            .map(|mut state| state.events.drain(..).collect::<Vec<_>>())
            .unwrap_or_default();
        for event in events {
            match event {
                NativeImeEvent::Composition(transfer) => {
                    let text = transfer.text.clone();
                    if self.runtime.request_composition(transfer) {
                        self.last_bridge_status = format!("Composition -> CEF: {text}");
                    }
                }
                NativeImeEvent::Result(text) => {
                    println!("Native IME result observed but not committed in this probe: {:?}", text);
                    self.last_bridge_status = format!("Result observed: {text}");
                }
                NativeImeEvent::Start => {
                    self.last_bridge_status = "Native composition start".to_string();
                }
                NativeImeEvent::End => {
                    self.last_bridge_status = "Native composition end".to_string();
                }
            }
        }
    }
}

impl Drop for ProbeEframeApp {
    fn drop(&mut self) {
        if let Some(hwnd) = self.installed_hwnd.take() {
            unsafe {
                let _ = RemoveWindowSubclass(hwnd, Some(native_ime_subclass_proc), SUBCLASS_ID);
                NATIVE_IME_STATE_PTR = ptr::null();
            }
        }
    }
}

impl eframe::App for ProbeEframeApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.ensure_subclass(frame);
        self.update_texture(ctx);

        if self.browser_active {
            ctx.send_viewport_cmd(egui::ViewportCommand::IMEAllowed(true));
        }

        let (generation, callbacks, bounds) = self
            .runtime
            .state
            .lock()
            .map(|state| (state.generation, state.ime_range_callbacks, state.character_bounds.len()))
            .unwrap_or((0, 0, 0));
        let (native_start, native_comp, native_end, compstr, resultstr, cursor) = self
            .native_ime
            .lock()
            .map(|state| {
                (
                    state.start_count,
                    state.composition_count,
                    state.end_count,
                    state.last_compstr.clone(),
                    state.last_resultstr.clone(),
                    state.last_cursor_pos,
                )
            })
            .unwrap_or((0, 0, 0, String::new(), String::new(), -1));

        egui::TopBottomPanel::top("status").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("WV-11-04-02 Native IME -> CEF Bridge");
                ui.separator();
                ui.label(format!("Paint: {generation}"));
                ui.separator();
                ui.label(format!("CEF range callbacks: {callbacks}"));
                ui.separator();
                ui.label(format!("Bounds: {bounds}"));
                ui.separator();
                ui.label(format!("Native start/comp/end: {native_start}/{native_comp}/{native_end}"));
                ui.separator();
                ui.label(format!("Cursor: {cursor}"));
            });
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("Native COMPSTR: {:?}", compstr));
                ui.separator();
                ui.label(format!("RESULTSTR: {:?}", resultstr));
                ui.separator();
                ui.label(format!("Bridge: {}", self.last_bridge_status));
            });
        });

        self.current_click = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            let available = ui.available_size();
            if let Some(texture) = self.texture.as_ref() {
                let response = ui.add(
                    egui::Image::new(texture)
                        .fit_to_exact_size(available)
                        .sense(egui::Sense::click()),
                );
                if response.clicked() {
                    if let Some(pointer) = response.interact_pointer_pos() {
                        if let Some((x, y)) = map_pointer_to_browser(response.rect, pointer) {
                            self.current_click = Some(ClickTransfer { x, y });
                            self.browser_active = true;
                        }
                    }
                }
            } else {
                ui.centered_and_justified(|ui| ui.label("Waiting for CEF OSR Paint..."));
            }
        });

        if let Some(click) = self.current_click {
            let _ = self.runtime.request_click(click);
        }
        self.drain_native_ime_events();
        ctx.request_repaint_after(MESSAGE_PUMP_INTERVAL);
    }
}

fn main() -> eframe::Result<()> {
    let cli_args: Vec<String> = std::env::args().collect();
    if is_cef_subprocess(&cli_args) {
        std::process::exit(run_subprocess());
    }

    println!("WV-11-04-02 Windows native IME -> CEF bridge probe start");
    println!("Click Browser input, enable Japanese IME, and type without confirming.");

    let runtime = match ProbeRuntime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("native IME bridge probe failed: {error}");
            std::process::exit(1);
        }
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("WV-11-04-02 Native IME -> CEF Bridge Probe")
            .with_inner_size([1100.0, 760.0]),
        ..Default::default()
    };

    eframe::run_native(
        "WV-11-04-02 Native IME -> CEF Bridge Probe",
        native_options,
        Box::new(move |_cc| Ok(Box::new(ProbeEframeApp::new(runtime)))),
    )
}