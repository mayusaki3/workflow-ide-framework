//! Linux向け WebView / GTK Host Window PoC処理。
//!
//! # 役割
//!
//! - eframe / egui 側から Linux 固有処理を分離する。
//! - WV-10-10 では GTK / WebKitGTK を生成しない構成で応答なし再現有無を確認する。
//!
//! # 注意点
//!
//! - 技術検証用コードである。
//! - Linux / X11 環境での実行を前提とする。
//! - 本検証では GTK Window、GtkFixed、WebKitGTK WebView を生成しない。
//! - 目的は、応答なしが GTK / WebKitGTK 生成に依存するかを切り分けることである。

use eframe::{egui, CreationContext};

/// Linux向け Root Window 初期化処理。
///
/// # 役割
///
/// - WV-10-10 では GTK Window を生成しない。
/// - App層の呼び出し構造を維持しながら、GTK / WebKitGTK 系を完全に外した状態を作る。
///
/// # 注意点
///
/// - `gtk::init()` も呼ばない。
/// - GTK Window / GtkFixed / WebView は生成しない。
///
/// # 引数
///
/// * `_cc` - eframe生成コンテキスト。本検証では使用しない。
///
/// # 戻り値
///
/// なし。
pub fn initialize_root_window(_cc: &CreationContext<'_>) {
    println!("WV-10-10 GTK/WebKit initialization skipped");
}

/// Linux向け WebView 初期化処理。
///
/// # 役割
///
/// - WV-10-10 では WebView を生成しない。
/// - App層の呼び出し構造を維持したまま、GTK / WebKitGTK なしで実行する。
///
/// # 注意点
///
/// - GTK Window / GtkFixed / WebView は生成しない。
/// - 初期矩形は受け取るが使用しない。
///
/// # 引数
///
/// * `_initial_rect` - 初期配置矩形。本検証では使用しない。
/// * `_scale` - egui の pixels_per_point。本検証では使用しない。
///
/// # 戻り値
///
/// なし。
pub fn ensure_webview_initialized(_initial_rect: Option<egui::Rect>, _scale: f32) {
    println!("WV-10-10 WebView creation skipped");
}

/// Linux向け Native Surface 追従処理。
///
/// # 役割
///
/// - WV-10-10 では Native Surface を生成していないため、何も同期しない。
/// - App層の呼び出し構造だけを維持する。
///
/// # 注意点
///
/// - GTKイベント処理も行わない。
/// - Window移動、Windowリサイズ、WebView bounds 同期は行わない。
///
/// # 引数
///
/// * `_ctx` - egui コンテキスト。本検証では使用しない。
/// * `_webview_rect` - WebView配置矩形。本検証では使用しない。
/// * `_should_show_native_surface` - Native Surface 表示判定。本検証では使用しない。
///
/// # 戻り値
///
/// なし。
pub fn sync_child_window(
    _ctx: &egui::Context,
    _webview_rect: Option<egui::Rect>,
    _should_show_native_surface: bool,
) {
}
