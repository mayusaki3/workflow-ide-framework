//! P0-1a 日本語Font 技術検証
//!
//! 目的:
//! - egui 日本語表示確認
//! - OS font fallback 確認
//! - Docking UI 日本語確認
//!
//! 注意:
//! - 本コードは技術検証用であり、正式実装ではない
//! - eframe 0.34 系の App::ui API を前提とする
//! - 固定 font path はクロスプラットフォーム性を壊すため使用しない

use eframe::egui;
use egui_dock::{DockArea, DockState, TabViewer};

/// Dock Tab
#[derive(Debug, Clone)]
enum PanelTab {
    /// 状態 Panel
    状態,

    /// ログ Panel
    ログ,
}

/// Dock UI
struct ValidationTabViewer;

impl TabViewer for ValidationTabViewer {
    type Tab = PanelTab;

    /// タイトル取得
    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            PanelTab::状態 => "状態".into(),
            PanelTab::ログ => "ログ".into(),
        }
    }

    /// Panel 描画
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            PanelTab::状態 => {
                ui.heading("日本語Font 技術検証");
                ui.label("日本語表示確認");
                ui.label("Docking UI 日本語確認");
                ui.label("OS font fallback 検証");
                ui.separator();
                ui.label("状態 Panel");
            }
            PanelTab::ログ => {
                ui.heading("ログ");
                ui.label("イベントログ表示予定");
                ui.label("日本語ログ表示テスト");
            }
        }
    }
}

/// 検証アプリ
struct ValidationApp {
    dock_state: DockState<PanelTab>,
}

impl ValidationApp {
    /// 初期化
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let dock_state = DockState::new(vec![
            PanelTab::状態,
            PanelTab::ログ,
        ]);

        Self { dock_state }
    }
}

impl eframe::App for ValidationApp {
    /// UI 更新
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        DockArea::new(&mut self.dock_state)
            .show_inside(ui, &mut ValidationTabViewer);
    }
}

/// エントリーポイント
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "P0-1a 日本語Font 技術検証",
        options,
        Box::new(|cc| Ok(Box::new(ValidationApp::new(cc)))),
    )
}
