//! P0-1 egui 技術検証
//!
//! 目的:
//! - eframe による Window 表示確認
//! - egui_dock による Docking 確認
//! - Multi Panel 確認
//!
//! 注意:
//! - Runtime / WebView は未導入
//! - GPU Viewport は Placeholder

use eframe::egui;
use egui_dock::{DockArea, DockState, NodeIndex, TabViewer};

/// Panel 種別
#[derive(Clone)]
enum PanelTab {
    /// Status Panel
    Status,

    /// Viewport Placeholder Panel
    Viewport,

    /// Log Panel
    Log,
}

/// Dock Tab Viewer
struct ValidationTabViewer;

impl TabViewer for ValidationTabViewer {
    type Tab = PanelTab;

    /// タイトル取得
    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            PanelTab::Status => "Status".into(),
            PanelTab::Viewport => "Viewport".into(),
            PanelTab::Log => "Log".into(),
        }
    }

    /// Panel 描画
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            PanelTab::Status => {
                ui.heading("Workflow IDE Framework");
                ui.separator();
                ui.label("Docking Validation");
            }
            PanelTab::Viewport => {
                ui.heading("GPU Viewport Placeholder");
                ui.label("GPU Viewport 予定領域");
            }
            PanelTab::Log => {
                ui.heading("Log Panel");
                ui.label("Runtime / Event Log 予定領域");
            }
        }
    }
}

/// Docking 検証アプリ
struct DockingValidationApp {
    dock_state: DockState<PanelTab>,
}

impl DockingValidationApp {
    /// 初期化
    fn new() -> Self {
        let mut dock_state = DockState::new(vec![PanelTab::Status]);

        let surface = dock_state.main_surface_mut();

        let [_, right] = surface.split_right(
            NodeIndex::root(),
            0.7,
            vec![PanelTab::Viewport],
        );

        surface.split_below(right, 0.8, vec![PanelTab::Log]);

        Self { dock_state }
    }
}

impl eframe::App for DockingValidationApp {
    /// UI 更新
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            DockArea::new(&mut self.dock_state)
                .show_inside(ui, &mut ValidationTabViewer);
        });
    }
}

/// エントリーポイント
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "P0-1 Docking Validation",
        options,
        Box::new(|_cc| Ok(Box::new(DockingValidationApp::new()))),
    )
}
