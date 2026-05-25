//! P0-1 egui 技術検証
//!
//! 目的:
//! - eframe による最小 Window 表示確認
//! - egui Event Loop 確認
//!
//! 注意:
//! - 最小 Window 構成
//! - Docking は未導入
//! - GPU Viewport は未導入
//! - Runtime / WebView は未導入

use eframe::egui;

/// 最小検証アプリ
struct MinimalApp;

impl eframe::App for MinimalApp {
    /// UI 更新
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Workflow IDE Framework");
            ui.separator();
            ui.label("P0-1 Minimal Window Validation");
            ui.label("eframe / egui 起動確認");
        });
    }
}

/// エントリーポイント
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "P0-1 Minimal Window",
        options,
        Box::new(|_cc| Ok(Box::new(MinimalApp))),
    )
}
