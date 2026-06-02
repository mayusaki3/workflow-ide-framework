//! Dock Panel タブ定義とタブ描画処理。
//!
//! 役割:
//! - egui_dock に表示する検証用Panelタブを定義する。
//! - WebView Placeholder の矩形を取得し、アプリ側へ通知する。
//!
//! 注意:
//! - P0-2 WebView 技術検証用のPoCコード。
//! - Native Surface の扱いは WV-03以降で変更される可能性がある。

use eframe::egui;
use egui_dock::TabViewer;
use serde::{Deserialize, Serialize};

/// P0-2 WebView 技術検証で使用する Dock Panel タブ。
#[derive(Clone, Serialize, Deserialize)]
pub enum PanelTab {
    Status,
    Viewport,
    Log,
    WebViewPlaceholder,
}

/// egui_dock のタブ描画担当。
///
/// `webview_rect` には WebView Placeholder Panel の矩形を記録する。
/// `active_panel_rects` には現在描画されたPanelコンテンツ矩形を記録する。
///
/// Windows では、`webview_rect` を Child Window Overlay の配置に使用し、
/// `active_panel_rects` をタブドラッグ候補判定の近似に使用する。
pub struct ValidationTabViewer<'a> {
    pub webview_rect: &'a mut Option<egui::Rect>,
    pub active_panel_rects: &'a mut Vec<egui::Rect>,
}

impl<'a> ValidationTabViewer<'a> {
    /// 現在描画中のPanelコンテンツ矩形を記録する。
    fn register_active_panel_rect(&mut self, ui: &egui::Ui) -> egui::Rect {
        let rect = ui.max_rect();
        self.active_panel_rects.push(rect);
        rect
    }
}

impl<'a> TabViewer for ValidationTabViewer<'a> {
    type Tab = PanelTab;

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            PanelTab::Status => "Status".into(),
            PanelTab::Viewport => "Viewport".into(),
            PanelTab::Log => "Log".into(),
            PanelTab::WebViewPlaceholder => "WebView".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        let rect = self.register_active_panel_rect(ui);

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
            PanelTab::WebViewPlaceholder => {
                ui.heading("WebView Placeholder");

                *self.webview_rect = Some(rect);

                ui.separator();
                ui.label(format!("x={:.1} y={:.1}", rect.min.x, rect.min.y));
                ui.label(format!("width={:.1} height={:.1}", rect.width(), rect.height()));
                ui.separator();
                ui.label("WV-00: Dock移動・Dockリサイズ時の矩形変化確認");
                ui.separator();
                ui.label("WV-00-01: Panel Rect取得 成功");
                ui.label("WV-00-02: Dock移動検知 成功");
                ui.label("WV-00-03: Dockリサイズ検知 成功");
                ui.label("PoC-1f: Child Window Dock追従確認");
            }
        }
    }
}
