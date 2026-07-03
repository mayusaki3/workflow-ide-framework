//! Platform固有処理の公開口。
//!
//! 役割:
//! - OS固有の Child Window / WebView 処理を呼び出す。
//! - Windows / Linux では OS別の実装を使用する。
//! - WV-11 Browser Surface 検証用 CEF モジュールを公開する。
//! - その他の環境では同じ関数シグネチャのスタブを提供する。
//!
//! 注意:
//! - P0-2 WebView 技術検証用のPoCコード。
//! - Linux / macOS の正式処理は WV-03 / WV-04 の結果に応じて追加する。
//! - CEF モジュールは WV-11 技術検証用であり、正式 API 仕様ではない。

pub mod cef;

#[cfg(target_os = "windows")]
mod windows_webview;

#[cfg(target_os = "linux")]
mod linux_webview;

#[cfg(target_os = "windows")]
pub use windows_webview::{
    ensure_webview_initialized,
    initialize_root_window,
    set_root_hwnd,
    sync_child_window,
};

#[cfg(target_os = "linux")]
pub use linux_webview::{
    ensure_webview_initialized,
    initialize_root_window,
    sync_child_window,
};

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
use eframe::egui;

/// 非Windows/Linux環境では Root Window 初期化は行わない。
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn initialize_root_window(_cc: &eframe::CreationContext<'_>) {}

/// 非Windows/Linux環境では WebView / Child Window 初期化は行わない。
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn ensure_webview_initialized(_initial_rect: Option<egui::Rect>, _scale: f32) {}

/// 非Windows/Linux環境では Child Window 追従処理は行わない。
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn sync_child_window(
    _ctx: &egui::Context,
    _webview_rect: Option<egui::Rect>,
    _should_show_native_surface: bool,
) {
}
