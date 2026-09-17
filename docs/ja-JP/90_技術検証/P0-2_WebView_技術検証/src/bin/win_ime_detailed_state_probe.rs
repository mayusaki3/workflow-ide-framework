//! WV-11-04-02 Windows IME detailed state Probe。
//!
//! 役割:
//! - eframe/winit の Win32 HWND で WM_IME_* を観測する。
//! - GCS_COMPSTR / GCS_COMPATTR / GCS_COMPCLAUSE / GCS_CURSORPOS / GCS_RESULTSTR を同時取得する。
//! - CEF 公式 Windows OSR 実装へ渡すために必要な native IME 状態を確認する。
//! - Space 変換時に composition 属性・clause・cursor がどう変化するかを記録する。
//!
//! 注意点:
//! - 本ファイルは技術検証用であり、正式 Surface API ではない。
//! - IME message は観測のみ行い、DefSubclassProc へ通常どおり転送する。
//! - CEF への転送は行わない。まず native IME 情報の実体を確定する。

use eframe::egui;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::ptr;
use std::sync::{Arc, Mutex};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::Ime::{
    ImmGetCompositionStringW, ImmGetContext, ImmGetOpenStatus, ImmReleaseContext,
    IME_COMPOSITION_STRING, GCS_COMPATTR, GCS_COMPCLAUSE, GCS_COMPSTR, GCS_CURSORPOS,
    GCS_RESULTSTR,
};
use windows::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{
    WM_IME_COMPOSITION, WM_IME_ENDCOMPOSITION, WM_IME_SETCONTEXT, WM_IME_STARTCOMPOSITION,
};

const SUBCLASS_ID: usize = 0x5749_4D46;

#[derive(Debug, Default)]
struct NativeImeState {
    hwnd: isize,
    setcontext_count: u64,
    start_count: u64,
    composition_count: u64,
    end_count: u64,
    ime_open: bool,
    lparam: usize,
    compstr: String,
    resultstr: String,
    compattr: Vec<u8>,
    compclause: Vec<u32>,
    cursor_pos: i32,
}

static mut NATIVE_IME_STATE_PTR: *const Mutex<NativeImeState> = ptr::null();

/// HIMC から UTF-16 composition 文字列を読み出す。
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
    let result = if byte_len > 0 {
        let unit_count = byte_len as usize / std::mem::size_of::<u16>();
        let mut buffer = vec![0u16; unit_count];
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
    result
}

/// HIMC から byte 配列を読み出す。
///
/// GCS_COMPATTR の観測に使用する。
unsafe fn read_ime_bytes(hwnd: HWND, index: IME_COMPOSITION_STRING) -> Vec<u8> {
    let himc = ImmGetContext(hwnd);
    if himc.0.is_null() {
        return Vec::new();
    }

    let byte_len = ImmGetCompositionStringW(himc, index, None, 0);
    let result = if byte_len > 0 {
        let mut buffer = vec![0u8; byte_len as usize];
        let copied = ImmGetCompositionStringW(
            himc,
            index,
            Some(buffer.as_mut_ptr().cast()),
            byte_len as u32,
        );
        if copied > 0 {
            buffer.truncate(copied as usize);
            buffer
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let _ = ImmReleaseContext(hwnd, himc);
    result
}

/// HIMC から DWORD 配列を読み出す。
///
/// GCS_COMPCLAUSE の観測に使用する。
unsafe fn read_ime_u32s(hwnd: HWND, index: IME_COMPOSITION_STRING) -> Vec<u32> {
    let himc = ImmGetContext(hwnd);
    if himc.0.is_null() {
        return Vec::new();
    }

    let byte_len = ImmGetCompositionStringW(himc, index, None, 0);
    let result = if byte_len > 0 {
        let count = byte_len as usize / std::mem::size_of::<u32>();
        let mut buffer = vec![0u32; count];
        let copied = ImmGetCompositionStringW(
            himc,
            index,
            Some(buffer.as_mut_ptr().cast()),
            byte_len as u32,
        );
        if copied > 0 {
            let copied_count = copied as usize / std::mem::size_of::<u32>();
            buffer.truncate(copied_count.min(buffer.len()));
            buffer
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let _ = ImmReleaseContext(hwnd, himc);
    result
}

/// HIMC から composition cursor 位置を読み出す。
unsafe fn read_cursor_pos(hwnd: HWND) -> i32 {
    let himc = ImmGetContext(hwnd);
    if himc.0.is_null() {
        return -1;
    }
    let cursor = ImmGetCompositionStringW(himc, GCS_CURSORPOS, None, 0);
    let _ = ImmReleaseContext(hwnd, himc);
    cursor
}

/// 現在の IME open 状態を読み出す。
unsafe fn read_ime_open(hwnd: HWND) -> bool {
    let himc = ImmGetContext(hwnd);
    if himc.0.is_null() {
        return false;
    }
    let open = ImmGetOpenStatus(himc).as_bool();
    let _ = ImmReleaseContext(hwnd, himc);
    open
}

/// native IME message と composition 詳細状態を観測する Window subclass procedure。
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
                    println!(
                        "WM_IME_STARTCOMPOSITION count={} open={}",
                        state.start_count, state.ime_open
                    );
                }
                WM_IME_COMPOSITION => {
                    state.composition_count = state.composition_count.saturating_add(1);
                    state.ime_open = read_ime_open(hwnd);
                    state.lparam = lparam.0 as usize;

                    let flags = IME_COMPOSITION_STRING(lparam.0 as u32);
                    if flags.contains(GCS_COMPSTR) {
                        state.compstr = read_ime_string(hwnd, GCS_COMPSTR);
                    }
                    if flags.contains(GCS_RESULTSTR) {
                        state.resultstr = read_ime_string(hwnd, GCS_RESULTSTR);
                    }
                    if flags.contains(GCS_COMPATTR) || flags.contains(GCS_COMPSTR) {
                        state.compattr = read_ime_bytes(hwnd, GCS_COMPATTR);
                    }
                    if flags.contains(GCS_COMPCLAUSE) || flags.contains(GCS_COMPSTR) {
                        state.compclause = read_ime_u32s(hwnd, GCS_COMPCLAUSE);
                    }
                    state.cursor_pos = read_cursor_pos(hwnd);

                    println!(
                        "WM_IME_COMPOSITION count={} lparam=0x{:X} comp={:?} result={:?} cursor={} attr={:?} clause={:?}",
                        state.composition_count,
                        lparam.0,
                        state.compstr,
                        state.resultstr,
                        state.cursor_pos,
                        state.compattr,
                        state.compclause
                    );
                }
                WM_IME_ENDCOMPOSITION => {
                    state.end_count = state.end_count.saturating_add(1);
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

struct ProbeApp {
    native_ime: Arc<Mutex<NativeImeState>>,
    installed_hwnd: Option<HWND>,
    input_text: String,
}

impl ProbeApp {
    fn new() -> Self {
        Self {
            native_ime: Arc::new(Mutex::new(NativeImeState::default())),
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
            NATIVE_IME_STATE_PTR = Arc::as_ptr(&self.native_ime);
            if SetWindowSubclass(hwnd, Some(native_ime_subclass_proc), SUBCLASS_ID, 0).as_bool() {
                self.installed_hwnd = Some(hwnd);
                println!("Detailed IME subclass installed: hwnd=0x{:X}", hwnd.0 as usize);
            } else {
                eprintln!("Detailed IME subclass installation failed");
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

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("WV-11-04-02 Windows IME Detailed State Probe");
            ui.label("Focus input, enable Japanese IME, type にほん, then press Space repeatedly.");
            ui.separator();

            let response = ui.text_edit_singleline(&mut self.input_text);
            if response.has_focus() {
                ctx.send_viewport_cmd(egui::ViewportCommand::IMEAllowed(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::IMERect(response.rect));
            }

            ui.separator();

            if let Ok(state) = self.native_ime.lock() {
                ui.monospace(format!("HWND              : 0x{:X}", state.hwnd));
                ui.monospace(format!("WM_IME_SETCONTEXT : {}", state.setcontext_count));
                ui.monospace(format!("WM_IME_START      : {}", state.start_count));
                ui.monospace(format!("WM_IME_COMPOSITION: {}", state.composition_count));
                ui.monospace(format!("WM_IME_END        : {}", state.end_count));
                ui.monospace(format!("IME open          : {}", state.ime_open));
                ui.monospace(format!("Last lParam       : 0x{:X}", state.lparam));
                ui.monospace(format!("GCS_COMPSTR       : {:?}", state.compstr));
                ui.monospace(format!("GCS_RESULTSTR     : {:?}", state.resultstr));
                ui.monospace(format!("GCS_CURSORPOS     : {}", state.cursor_pos));
                ui.monospace(format!("GCS_COMPATTR      : {:?}", state.compattr));
                ui.monospace(format!("GCS_COMPCLAUSE    : {:?}", state.compclause));
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    println!("WV-11-04-02 Windows IME detailed state probe start");
    println!("Focus input, enable Japanese IME, type にほん, then press Space repeatedly.");

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("WV-11-04-02 Windows IME Detailed State Probe")
            .with_inner_size([900.0, 560.0]),
        ..Default::default()
    };

    eframe::run_native(
        "WV-11-04-02 Windows IME Detailed State Probe",
        native_options,
        Box::new(|_cc| Ok(Box::new(ProbeApp::new()))),
    )
}
