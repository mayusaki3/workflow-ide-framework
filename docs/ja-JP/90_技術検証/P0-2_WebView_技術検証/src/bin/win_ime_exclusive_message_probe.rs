//! WV-11-04-02 Windows IME 排他メッセージ処理 Probe。
//!
//! 役割:
//! - eframe/winit の HWND で WM_IME_* を Window subclass により観測する。
//! - Browser Surface 相当の入力領域が active な間だけ、WM_IME_STARTCOMPOSITION /
//!   WM_IME_COMPOSITION / WM_IME_ENDCOMPOSITION を DefSubclassProc へ流さず消費する。
//! - PowerShell では発生しない「Space 1回目で候補ウィンドウが消える」現象が、
//!   winit/egui 側との二重処理に起因するかを切り分ける。
//!
//! 注意点:
//! - 本 Probe は CEF へ転送しない。排他処理そのものの影響だけを確認する。
//! - WM_IME_SETCONTEXT は既定 IME UI の維持に必要なため DefSubclassProc へ流す。
//! - 正式 Surface API ではない。

use eframe::egui;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::ptr;
use std::sync::{Arc, Mutex};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::Ime::{
    ImmGetCompositionStringW, ImmGetContext, ImmGetOpenStatus, ImmReleaseContext,
    IME_COMPOSITION_STRING, GCS_COMPSTR, GCS_CURSORPOS, GCS_RESULTSTR,
};
use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{
    WM_IME_COMPOSITION, WM_IME_ENDCOMPOSITION, WM_IME_SETCONTEXT, WM_IME_STARTCOMPOSITION,
};

const SUBCLASS_ID: usize = 0x5749_4D46;

#[derive(Debug, Default)]
struct NativeImeState {
    active: bool,
    setcontext_count: u64,
    start_count: u64,
    composition_count: u64,
    end_count: u64,
    ime_open: bool,
    last_compstr: String,
    last_resultstr: String,
    last_cursor_pos: i32,
}

static mut NATIVE_IME_STATE_PTR: *const Mutex<NativeImeState> = ptr::null();

/// HIMC から指定された UTF-16 文字列を取得する。
///
/// # 引数
/// - `hwnd`: 対象 HWND。
/// - `index`: GCS_COMPSTR または GCS_RESULTSTR。
///
/// # 戻り値
/// - 取得した文字列。取得できない場合は空文字列。
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

/// HIMC から Composition cursor 位置を取得する。
unsafe fn read_cursor_pos(hwnd: HWND) -> i32 {
    let himc = ImmGetContext(hwnd);
    if himc.0.is_null() {
        return -1;
    }
    let cursor = ImmGetCompositionStringW(himc, GCS_CURSORPOS, None, 0);
    let _ = ImmReleaseContext(hwnd, himc);
    cursor
}

/// HIMC の open 状態を取得する。
unsafe fn read_ime_open(hwnd: HWND) -> bool {
    let himc = ImmGetContext(hwnd);
    if himc.0.is_null() {
        return false;
    }
    let open = ImmGetOpenStatus(himc).as_bool();
    let _ = ImmReleaseContext(hwnd, himc);
    open
}

/// IME message を観測し、active 中の Composition 系 message を排他的に消費する。
///
/// WM_IME_SETCONTEXT は default IME UI を維持するため通常処理へ流す。
unsafe extern "system" fn native_ime_subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _subclass_id: usize,
    _ref_data: usize,
) -> LRESULT {
    if !NATIVE_IME_STATE_PTR.is_null() {
        let shared = &*NATIVE_IME_STATE_PTR;
        if let Ok(mut state) = shared.lock() {
            match msg {
                WM_IME_SETCONTEXT => {
                    state.setcontext_count = state.setcontext_count.saturating_add(1);
                    println!(
                        "WM_IME_SETCONTEXT count={} active={} wparam=0x{:X} lparam=0x{:X}",
                        state.setcontext_count, state.active, wparam.0, lparam.0
                    );
                }
                WM_IME_STARTCOMPOSITION => {
                    state.start_count = state.start_count.saturating_add(1);
                    state.ime_open = read_ime_open(hwnd);
                    println!(
                        "WM_IME_STARTCOMPOSITION count={} active={} open={} consumed={}",
                        state.start_count, state.active, state.ime_open, state.active
                    );
                    if state.active {
                        return LRESULT(0);
                    }
                }
                WM_IME_COMPOSITION => {
                    state.composition_count = state.composition_count.saturating_add(1);
                    state.ime_open = read_ime_open(hwnd);
                    let flags = IME_COMPOSITION_STRING(lparam.0 as u32);
                    if flags.contains(GCS_COMPSTR) {
                        state.last_compstr = read_ime_string(hwnd, GCS_COMPSTR);
                        state.last_cursor_pos = read_cursor_pos(hwnd);
                    }
                    if flags.contains(GCS_RESULTSTR) {
                        state.last_resultstr = read_ime_string(hwnd, GCS_RESULTSTR);
                    }
                    println!(
                        "WM_IME_COMPOSITION count={} active={} comp={:?} result={:?} cursor={} lparam=0x{:X} consumed={}",
                        state.composition_count,
                        state.active,
                        state.last_compstr,
                        state.last_resultstr,
                        state.last_cursor_pos,
                        lparam.0,
                        state.active
                    );
                    if state.active {
                        return LRESULT(0);
                    }
                }
                WM_IME_ENDCOMPOSITION => {
                    state.end_count = state.end_count.saturating_add(1);
                    println!(
                        "WM_IME_ENDCOMPOSITION count={} active={} consumed={}",
                        state.end_count, state.active, state.active
                    );
                    if state.active {
                        return LRESULT(0);
                    }
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

struct ProbeApp {
    state: Arc<Mutex<NativeImeState>>,
    installed_hwnd: Option<HWND>,
    input_text: String,
}

impl ProbeApp {
    fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(NativeImeState::default())),
            installed_hwnd: None,
            input_text: String::new(),
        }
    }

    fn ensure_subclass(&mut self, frame: &eframe::Frame) {
        if self.installed_hwnd.is_some() {
            return;
        }
        let Some(hwnd) = frame_hwnd(frame) else { return; };
        unsafe {
            NATIVE_IME_STATE_PTR = Arc::as_ptr(&self.state);
            if SetWindowSubclass(hwnd, Some(native_ime_subclass_proc), SUBCLASS_ID, 0).as_bool() {
                self.installed_hwnd = Some(hwnd);
                println!("Exclusive IME subclass installed: hwnd=0x{:X}", hwnd.0 as usize);
            } else {
                eprintln!("Exclusive IME subclass installation failed");
            }
        }
    }
}

impl Drop for ProbeApp {
    fn drop(&mut self) {
        if let Some(hwnd) = self.installed_hwnd.take() {
            unsafe {
                let _ = RemoveWindowSubclass(hwnd, Some(native_ime_subclass_proc), SUBCLASS_ID);
                NATIVE_IME_STATE_PTR = ptr::null();
            }
        }
    }
}

impl eframe::App for ProbeApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.ensure_subclass(frame);

        let snapshot = self.state.lock().ok().map(|state| {
            (
                state.active,
                state.setcontext_count,
                state.start_count,
                state.composition_count,
                state.end_count,
                state.ime_open,
                state.last_compstr.clone(),
                state.last_resultstr.clone(),
                state.last_cursor_pos,
            )
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("WV-11-04-02 Windows IME Exclusive Message Probe");
            ui.label("Click the input below, enable Japanese IME, type, then press Space repeatedly.");
            ui.label("Composition messages are consumed while this input is focused.");
            ui.separator();

            if let Some((active, setcontext, start, composition, end, open, comp, result, cursor)) = snapshot {
                ui.monospace(format!("Active             : {active}"));
                ui.monospace(format!("WM_IME_SETCONTEXT  : {setcontext}"));
                ui.monospace(format!("WM_IME_START       : {start}"));
                ui.monospace(format!("WM_IME_COMPOSITION : {composition}"));
                ui.monospace(format!("WM_IME_END         : {end}"));
                ui.monospace(format!("IME open           : {open}"));
                ui.monospace(format!("GCS_COMPSTR        : {comp:?}"));
                ui.monospace(format!("GCS_RESULTSTR      : {result:?}"));
                ui.monospace(format!("GCS_CURSORPOS      : {cursor}"));
            }

            ui.separator();
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.input_text)
                    .desired_width(520.0)
                    .hint_text("Japanese IME test target"),
            );
            let active = response.has_focus();
            if let Ok(mut state) = self.state.lock() {
                state.active = active;
            }
            if active {
                ctx.send_viewport_cmd(egui::ViewportCommand::IMEAllowed(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::IMERect(response.rect));
            }
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(16));
    }
}

fn main() -> eframe::Result<()> {
    println!("WV-11-04-02 Windows IME exclusive message probe start");
    println!("Focus input, enable Japanese IME, type, and press Space repeatedly.");

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("WV-11-04-02 Windows IME Exclusive Message Probe")
            .with_inner_size([900.0, 620.0]),
        ..Default::default()
    };

    eframe::run_native(
        "WV-11-04-02 Windows IME Exclusive Message Probe",
        native_options,
        Box::new(|_cc| Ok(Box::new(ProbeApp::new()))),
    )
}
