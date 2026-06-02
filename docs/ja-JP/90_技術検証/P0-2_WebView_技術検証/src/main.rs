//! P0-2 WebView 技術検証
//!
//! 役割:
//! - 技術検証アプリケーションのエントリーポイント。
//!
//! 注意:
//! - P0-2 WebView 技術検証用のPoCコード。
//! - WV-03以降の検証結果により、モジュール構成は変更される可能性がある。

mod app;
mod layout_storage;
mod panel_tab;
mod platform;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "P0-2 WebView Validation",
        options,
        Box::new(|cc| Ok(Box::new(app::DockingValidationApp::new(cc)))),
    )
}
