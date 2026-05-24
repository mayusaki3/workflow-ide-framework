//! P0-1 egui 技術検証
//!
//! 目的:
//! - eframe による Window 表示確認
//! - egui_dock による Docking UI 確認
//! - Multi Panel 表示確認
//! - GPU Viewport 用 Panel 配置確認
//!
//! 注意:
//! - 本コードは技術検証用であり、正式実装ではない
//! - Runtime / Event / State / WebView は未統合

use eframe::egui;
use egui_dock::{DockArea, DockState, NodeIndex, TabViewer};

/// Dock Tab 種別
#[derive(Debug, Clone)]
enum PanelTab {
    /// 状態表示 Panel
    Status,

    /// GPU Viewport 想定 Panel
    Viewport,

    /// ログ表示 Panel
    Log,
}

/// Dock 表示実装
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
                ui.heading("P0-1 egui Validation");
                ui.label("Window / Docking / Multi Panel 確認");
            }
            PanelTab::Viewport => {
                ui.heading("GPU Viewport Placeholder");
                ui.separator();
                ui.label("将来的に GPU Viewport を配置予定");
            }
            PanelTab::Log => {
                ui.heading("Log Panel");
                ui.label("Event / Runtime log 表示予定");
            }
        }
    }
}

/// 技術検証アプリ
struct ValidationApp {
    dock_state: DockState<PanelTab>,
}

impl ValidationApp {
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

impl eframe::App for ValidationApp {
    /// UI 更新
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Workflow IDE Framework");
            });
        });

        DockArea::new(&mut self.dock_state)
            .show(ctx, &mut ValidationTabViewer);
    }
}

/// エントリーポイント
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "P0-1 egui 技術検証",
        options,
        Box::new(|_cc| Box::new(ValidationApp::new())),
    )
}
