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

/// エントリーポイント
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    eframe::run_simple_native(
        "P0-1 Minimal Window",
        options,
        Box::new(|ctx, _frame| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Workflow IDE Framework");
                ui.separator();
                ui.label("P0-1 Minimal Window Validation");
                ui.label("eframe / egui 起動確認");
            });
        }),
    )
}
