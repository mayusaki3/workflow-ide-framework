//! WV-11-04-02 Windows native IME message Probe。
//!
//! 役割:
//! - eframe/winit の Win32 HWND に対して window subclass を設定する。
//! - `WM_IME_SETCONTEXT` / `WM_IME_STARTCOMPOSITION` / `WM_IME_COMPOSITION` /
//!   `WM_IME_ENDCOMPOSITION` を観測する。
//! - `WM_IME_COMPOSITION` 受信時に HIMC から `GCS_COMPSTR` と `GCS_CURSORPOS` を取得する。
//! - CEF 公式 Windows OSR 実装と同じ native IME message 経路が利用可能かを切り分ける。
//!
//! 注意点:
//! - 本ファイルは技術検証用であり、正式 Surface API ではない。
//! - CEF への転送は行わない。native IME message と composition 取得のみを確認する。
//! - subclass callback は元の WindowProc 処理へ必ず委譲する。

use eframe::egui;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::sync::{Arc, Mutex};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::Ime::{
    ImmGetCompositionStringW, ImmGetContext, ImmGetOpenStatus, ImmReleaseContext, GCS_COMPSTR,
    GCS_CURSORPOS,
};
use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{
    WM_IME_COMPOSITION, WM_IME_ENDCOMPOSITION, WM_IME_SETCONTEXT, WM_IME_STARTCOMPOSITION,
};

const SUBCLASS_ID: usize = 0x5749_4D45; // "WIME"

/// native IME message の観測状態。
///
/// @hldocs.ref doc-20260912-104801Z-WV16#sec_b6m1r9p3x7da
#[derive(Debug, Default)]
struct NativeImeMessageState {
    hwnd: isize,
    setcontext_count: u64,
    start_count: u64,
    composition_count: u64,
    end_count: u64,
    has_context: bool,
    ime_open: bool,
    last_lparam: isize,
    last_compstr: String,
    last_cursor_pos: i32,
}

static mut STATE_PTR: *const Mutex<NativeImeMessageState> = std::ptr::null();

/// UTF-16 composition string を HIMC から取得する。
///
/// # 引数
/// - `hwnd`: native window handle。
///
/// # 戻り値
/// - `(composition_text, cursor_pos, has_context, ime_open)`。
unsafe fn read_native_composition(hwnd: HWND) -> (String, i32, bool, bool) {
    let himc = ImmGetContext(hwnd);
    if himc.0.is_null() {
        return (String::new(), -1, false, false);
    }

    let ime_open = ImmGetOpenStatus(himc).as_bool();
    let byte_len = ImmGetCompositionStringW(himc, GCS_COMPSTR, None, 0);
    let cursor_pos = ImmGetCompositionStringW(himc, GCS_CURSORPOS, None, 0);

    let text = if byte_len > 0 {
        let units = (byte_len as usize) / std::mem::size_of::<u16>();
        let mut buffer = vec![0u16; units];
        let copied = ImmGetCompositionStringW(
            himc,
            GCS_COMPSTR,
            Some(buffer.as_mut_ptr().cast()),
            byte_len as u32,
        );
        if copied > 0 {
            let copied_units = (copied as usize) / std::mem::size_of::<u16>();
            String::from_utf16_lossy(&buffer[..copied_units.min(buffer.len())])
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let _ = ImmReleaseContext(hwnd, himc);
    (text, cursor_pos, true, ime_open)
}

/// eframe window に設定する subclass callback。
///
/// 元の WindowProc を置換せず、IME message を観測した後で `DefSubclassProc` へ委譲する。
unsafe extern "system" fn ime_subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _subclass_id: usize,
    _ref_data: usize,
) -> LRESULT {
    if !STATE_PTR.is_null() {
        let state = &*STATE_PTR;
        if let Ok(mut state) = state.lock() {
            state.hwnd = hwnd.0 as isize;
            state.last_lparam = lparam.0;

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
                    let (text, cursor, has_context, ime_open) = read_native_composition(hwnd);
                    state.last_compstr = text;
                    state.last_cursor_pos = cursor;
                    state.has_context = has_context;
                    state.ime_open = ime_open;
                    println!(
                        "WM_IME_STARTCOMPOSITION count={} context={} open={} comp={:?} cursor={}",
                        state.start_count, has_context, ime_open, state.last_compstr, cursor
                    );
                }
                WM_IME_COMPOSITION => {
                    state.composition_count = state.composition_count.saturating_add(1);
                    let (text, cursor, has_context, ime_open) = read_native_composition(hwnd);
                    state.last_compstr = text;
                    state.last_cursor_pos = cursor;
                    state.has_context = has_context;
                    state.ime_open = ime_open;
                    println!(
                        "WM_IME_COMPOSITION count={} lparam=0x{:X} context={} open={} comp={:?} cursor={}",
                        state.composition_count,
                        lparam.0,
                        has_context,
                        ime_open,
                        state.last_compstr,
                        cursor
                    );
                }
                WM_IME_ENDCOMPOSITION => {
                    state.end_count = state.end_count.saturating_add(1);
                    let (text, cursor, has_context, ime_open) = read_native_composition(hwnd);
                    state.last_compstr = text;
                    state.last_cursor_pos = cursor;
                    state.has_context = has_context;
                    state.ime_open = ime_open;
                    println!(
                        "WM_IME_ENDCOMPOSITION count={} context={} open={} comp={:?} cursor={}",
                        state.end_count, has_context, ime_open, state.last_compstr, cursor
                    );
                }
                _ => {}
            }
        }
    }

    DefSubclassProc(hwnd, msg, wparam, lparam)
}

/// eframe Frame から Win32 HWND を取得する。
fn frame_hwnd(frame: &eframe::Frame) -> Option<HWND> {
    let handle = frame.window_handle().ok()?;
    match handle.as_raw() {
        RawWindowHandle::Win32(win32) => Some(HWND(win32.hwnd.get() as *mut _)),
        _ => None,
    }
}

struct NativeImeMessageApp {
    state: Arc<Mutex<NativeImeMessageState>>,
    installed_hwnd: Option<HWND>,
    input_text: String,
}

impl NativeImeMessageApp {
    fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(NativeImeMessageState::default())),
            installed_hwnd: None,
            input_text: String::new(),
        }
    }

    fn ensure_subclass(&mut self, frame: &eframe::Frame) {
        if self.installed_hwnd.is_some() {
            return;
        }

        let Some(hwnd) = frame_hwnd(frame) else {
            return;
        };

        unsafe {
            STATE_PTR = Arc::as_ptr(&self.state);
            let installed = SetWindowSubclass(hwnd, Some(ime_subclass_proc), SUBCLASS_ID, 0).as_bool();
            if installed {
                self.installed_hwnd = Some(hwnd);
                println!("IME subclass installed: hwnd=0x{:X}", hwnd.0 as usize);
            } else {
                eprintln!("IME subclass installation failed: hwnd=0x{:X}", hwnd.0 as usize);
            }
        }
    }
}

impl Drop for NativeImeMessageApp {
    fn drop(&mut self) {
        if let Some(hwnd) = self.installed_hwnd.take() {
            unsafe {
                let _ = RemoveWindowSubclass(hwnd, Some(ime_subclass_proc), SUBCLASS_ID);
                STATE_PTR = std::ptr::null();
            }
        }
    }
}

impl eframe::App for NativeImeMessageApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.ensure_subclass(frame);

        let snapshot = self.state.lock().ok().map(|state| {
            (
                state.hwnd,
                state.setcontext_count,
                state.start_count,
                state.composition_count,
                state.end_count,
                state.has_context,
                state.ime_open,
                state.last_lparam,
                state.last_compstr.clone(),
                state.last_cursor_pos,
            )
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("WV-11-04-02 Windows Native IME Message Probe");
            ui.label("Click this window, enable Japanese IME, then type without confirming.");
            ui.separator();

            if let Some((
                hwnd,
                setcontext_count,
                start_count,
                composition_count,
                end_count,
                has_context,
                ime_open,
                last_lparam,
                last_compstr,
                last_cursor_pos,
            )) = snapshot
            {
                ui.monospace(format!("HWND                 : 0x{:X}", hwnd));
                ui.monospace(format!("WM_IME_SETCONTEXT    : {setcontext_count}"));
                ui.monospace(format!("WM_IME_START         : {start_count}"));
                ui.monospace(format!("WM_IME_COMPOSITION   : {composition_count}"));
                ui.monospace(format!("WM_IME_END           : {end_count}"));
                ui.monospace(format!("Has context          : {has_context}"));
                ui.monospace(format!("IME open             : {ime_open}"));
                ui.monospace(format!("Last lParam          : 0x{:X}", last_lparam));
                ui.monospace(format!("GCS_COMPSTR          : {:?}", last_compstr));
                ui.monospace(format!("GCS_CURSORPOS        : {last_cursor_pos}"));
            } else {
                ui.label("Waiting for native IME state...");
            }

            ui.separator();
            ui.label("Input target for eframe/winit IME:");
            ui.add(
                egui::TextEdit::singleline(&mut self.input_text)
                    .hint_text("Type Japanese IME here")
                    .desired_width(420.0),
            );
            ui.label("Observe WM_IME_* counts and GCS_COMPSTR in this window.");
        });

        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    println!("WV-11-04-02 Windows native IME message probe start");
    println!("Focus the window, enable Japanese IME, and type without confirming.");

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("WV-11-04-02 Windows Native IME Message Probe")
            .with_inner_size([840.0, 520.0]),
        ..Default::default()
    };

    eframe::run_native(
        "WV-11-04-02 Windows Native IME Message Probe",
        native_options,
        Box::new(|_cc| Ok(Box::new(NativeImeMessageApp::new()))),
    )
}
