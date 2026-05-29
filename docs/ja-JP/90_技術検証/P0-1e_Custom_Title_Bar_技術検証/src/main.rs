//! P0-1e Custom Title Bar 技術検証

use eframe::egui;

#[derive(Default)]
struct ValidationApp;

impl eframe::App for ValidationApp {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _frame: &mut eframe::Frame,
    ) {
        ui.heading("P0-1e カスタムタイトルバー検証");

        ui.separator();

        ui.label("日本語タイトル表示");
        ui.label("native title 非依存確認");
        ui.label("Windows / Linux / macOS 共通UI確認");
    }
}

fn main() -> eframe::Result<()> {
    let viewport = egui::ViewportBuilder::default()
        .with_decorations(false)
        .with_title("P0-1e Custom Title Bar Validation");

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "P0-1e",
        options,
        Box::new(|_cc| Ok(Box::new(ValidationApp::default()))),
    )
}