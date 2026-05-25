//! P0-1 egui 技術検証
//!
//! 目的:
//! - eframe による Window 表示確認
//! - egui_dock による Docking 確認
//! - Multi Panel 確認
//! - Layout Persistence 確認
//!
//! 注意:
//! - Runtime / WebView は未導入
//! - GPU Viewport は Placeholder

use std::fs;
use std::path::Path;

use eframe::egui;
use egui_dock::{DockArea, DockState, NodeIndex, TabViewer};
use serde::{Deserialize, Serialize};

/// Layout 保存先
const LAYOUT_FILE_PATH: &str = "dock_layout.json";

/// Panel 種別
#[derive(Clone, Serialize, Deserialize)]
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
        let dock_state = Self::load_layout()
            .unwrap_or_else(Self::create_default_layout);

        Self { dock_state }
    }

    /// 初期 Layout 作成
    fn create_default_layout() -> DockState<PanelTab> {
        let mut dock_state = DockState::new(vec![PanelTab::Status]);

        let surface = dock_state.main_surface_mut();

        let [_, right] = surface.split_right(
            NodeIndex::root(),
            0.7,
            vec![PanelTab::Viewport],
        );

        surface.split_below(right, 0.8, vec![PanelTab::Log]);

        dock_state
    }

    /// Layout 保存
    fn save_layout(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.dock_state) {
            let _ = fs::write(LAYOUT_FILE_PATH, json);
        }
    }

    /// Layout 読み込み
    fn load_layout() -> Option<DockState<PanelTab>> {
        if !Path::new(LAYOUT_FILE_PATH).exists() {
            return None;
        }

        let json = fs::read_to_string(LAYOUT_FILE_PATH).ok()?;

        serde_json::from_str(&json).ok()
    }
}

impl Drop for DockingValidationApp {
    /// 終了時保存
    fn drop(&mut self) {
        self.save_layout();
    }
}

impl eframe::App for DockingValidationApp {
    /// UI 更新
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_panel").show(ctx, |ui| {
            if ui.button("Save Layout").clicked() {
                self.save_layout();
            }
        });

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
