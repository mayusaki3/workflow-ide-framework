//! Windows native IME message probe 実装前の API 形状確認用 Probe。
//!
//! 役割:
//! - `eframe::Frame` から Win32 HWND を取得できることを型検査する。
//! - `windows 0.61` の IMM32 API (`ImmGetContext` / `ImmGetCompositionStringW`) の
//!   実シグネチャを確認する。
//! - WM_IME_* を直接観測する runtime Probe へ推測した API 形状を持ち込まない。
//!
//! 注意点:
//! - 本ファイルは `cargo check --bin win_ime_message_api_compile_probe` 用。
//! - 実行時の WindowProc 差し替えは行わない。
//! - 正式 Surface API ではない。

#[cfg(target_os = "windows")]
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;
#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::Ime::{
    ImmGetCompositionStringW, ImmGetContext, ImmReleaseContext, GCS_COMPSTR, GCS_CURSORPOS,
};

/// eframe Frame の raw window handle から Win32 HWND を取り出す形を型検査する。
///
/// # 引数
/// - `frame`: eframe の native frame。
///
/// # 戻り値
/// - Windows HWND。取得できない場合は `None`。
///
/// @hldocs.ref doc-20260912-104801Z-WV16#sec_b6m1r9p3x7da
#[cfg(target_os = "windows")]
#[allow(dead_code)]
fn frame_hwnd(frame: &eframe::Frame) -> Option<HWND> {
    let handle = frame.window_handle().ok()?;
    match handle.as_raw() {
        RawWindowHandle::Win32(win32) => Some(HWND(win32.hwnd.get() as *mut _)),
        _ => None,
    }
}

/// HWND に関連付いた HIMC から Composition 文字列長と cursor 位置を読む形を型検査する。
///
/// # 引数
/// - `hwnd`: 対象 native window。
///
/// # 戻り値
/// - `(composition_bytes, cursor_pos)`。HIMC が無い場合は `None`。
///
/// @hldocs.ref doc-20260912-104801Z-WV16#sec_b6m1r9p3x7da
#[cfg(target_os = "windows")]
#[allow(dead_code)]
fn inspect_ime_context(hwnd: HWND) -> Option<(i32, i32)> {
    unsafe {
        let himc = ImmGetContext(hwnd);
        if himc.0.is_null() {
            return None;
        }

        let composition_bytes = ImmGetCompositionStringW(himc, GCS_COMPSTR, None, 0);
        let cursor_pos = ImmGetCompositionStringW(himc, GCS_CURSORPOS, None, 0);
        let _ = ImmReleaseContext(hwnd, himc);

        Some((composition_bytes, cursor_pos))
    }
}

fn main() {
    println!("Windows IME message API compile probe");
}
