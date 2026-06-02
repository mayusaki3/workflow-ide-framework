//! Platform固有処理の公開口。
//!
//! 役割:
//! - Windows固有の Child Window / WebView 処理を呼び出す。
//! - 非Windows環境では同じ関数シグネチャのスタブを提供する。
//!
//! 注意:
//! - P0-2 WebView 技術検証用のPoCコード。
//! - Linux / macOS の正式処理は WV-03 / WV-04 の結果に応じて追加する。

#[cfg(target_os = "windows")]
mod windows_webview;

#[cfg(target_os = "windows")]
pub use windows_webview::{
    ensure_webview_initialized,
    set_root_hwnd,
    sync_child_window,
};

#[cfg(not(target_os = "windows"))]
use eframe::egui;

/// 非Windows環境では WebView表示フラグ更新は行わない。
#[cfg(not(target_os = "windows"))]
pub fn mark_webview_visible() {}

/// 非Windows環境では WebView表示フラグ初期化は行わない。
#[cfg(not(target_os = "windows"))]
pub fn reset_webview_visible() {}

/// 非Windows環境では WebView / Child Window 初期化は行わない。
#[cfg(not(target_os = "windows"))]
pub fn ensure_webview_initialized(_initial_rect: Option<egui::Rect>, _scale: f32) {}

#[cfg(not(target_os = "windows"))]
pub fn set_root_hwnd(_hwnd: isize) {}

/// 非Windows環境では Child Window 追従処理は行わない。
#[cfg(not(target_os = "windows"))]
pub fn sync_child_window(
    _ctx: &egui::Context,
    _webview_rect: Option<egui::Rect>,
    _should_show_native_surface: bool,
) {
}
