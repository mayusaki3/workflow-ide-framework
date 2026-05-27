//! P0-1c Linux Native Window Title 技術検証
//!
//! 目的:
//! - Linux native title 日本語確認
//! - Wayland/X11 差異確認
//! - locale 確認

use eframe::egui;

/// 検証アプリ
#[derive(Default)]
struct ValidationApp;

impl eframe::App for ValidationApp {
    /// UI 更新
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("Linux Native Window Title 技術検証");

        ui.separator();

        ui.label("日本語 Panel 表示確認");
        ui.label("ASCII title / 日本語 title / mixed title");

        ui.separator();

        ui.label(format!(
            "LANG: {}",
            std::env::var("LANG").unwrap_or_else(|_| "<undefined>".to_owned())
        ));

        ui.label(format!(
            "LC_ALL: {}",
            std::env::var("LC_ALL").unwrap_or_else(|_| "<undefined>".to_owned())
        ));

        ui.label(format!(
            "XDG_SESSION_TYPE: {}",
            std::env::var("XDG_SESSION_TYPE")
                .unwrap_or_else(|_| "<undefined>".to_owned())
        ));
    }
}

/// エントリーポイント
fn main() -> eframe::Result<()> {
    println!("LANG={:?}", std::env::var("LANG"));
    println!("LC_ALL={:?}", std::env::var("LC_ALL"));
    println!("XDG_SESSION_TYPE={:?}", std::env::var("XDG_SESSION_TYPE"));

    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "P0-1c Linux 日本語タイトル検証",
        options,
        Box::new(|_cc| Ok(Box::new(ValidationApp))),
    )
}
