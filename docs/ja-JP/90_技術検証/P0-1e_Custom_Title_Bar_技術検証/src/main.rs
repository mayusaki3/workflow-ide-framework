//! P0-1e Custom Title Bar 技術検証
//!
//! 目的:
//! - native title 非依存確認
//! - custom title bar 実現性確認
//! - 日本語 title 表示確認

use eframe::egui;

#[derive(Default)]
struct ValidationApp;

impl eframe::App for ValidationApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("custom_title_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("P0-1e カスタムタイトルバー検証");
                ui.separator();
                ui.label("日本語タイトル表示");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Custom Title Bar 技術検証");
            ui.separator();
            ui.label("native title 非依存確認");
            ui.label("Windows / Linux / macOS 共通UI確認");
            ui.label("日本語表示確認");
        });
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
