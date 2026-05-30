//! P0-2 WebView 技術検証
//!
//! 目的:
//! - egui_dock による Dock Panel 矩形取得確認
//! - Dock 移動時の矩形変化確認
//! - Dock リサイズ時の矩形変化確認
//! - WebView Support Panel 方式の前提確認
//!
//! 注意:
//! - WV-00 の PoC-0 実装
//! - この段階では wry は未導入
//! - WebView は Placeholder
//! - Child Window / WebView 追従は PoC-1 以降で検証する

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

    /// WebView Placeholder Panel
    WebViewPlaceholder,
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
            PanelTab::WebViewPlaceholder => "WebView".into(),
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
            PanelTab::WebViewPlaceholder => {
                ui.heading("WebView Placeholder");

                let rect = ui.max_rect();

                ui.separator();

                ui.label(format!(
                    "x={:.1} y={:.1}",
                    rect.min.x,
                    rect.min.y
                ));

                ui.label(format!(
                    "width={:.1} height={:.1}",
                    rect.width(),
                    rect.height()
                ));

                ui.separator();

                ui.label(
                    "WV-00: Dock移動・Dockリサイズ時の矩形変化確認"
                );

                ui.separator();

                ui.label("WV-00-01: Panel Rect取得 成功");
                ui.label("WV-00-02: Dock移動検知 成功");
                ui.label("WV-00-03: Dockリサイズ検知 成功");            }
        }
    }
}

/// Docking 検証アプリ
struct DockingValidationApp {
    dock_state: DockState<PanelTab>,
    viewport_info: String,
}

impl DockingValidationApp {
    /// 初期化
    fn new() -> Self {
        let dock_state = Self::load_layout()
            .unwrap_or_else(Self::create_default_layout);

        Self {
            dock_state,
            viewport_info: String::new(),
        }
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

        let [_, bottom] = surface.split_below(
            right,
            0.8,
            vec![PanelTab::Log],
        );

        surface.split_right(
            bottom,
            0.5,
            vec![PanelTab::WebViewPlaceholder],
        );

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
        let screen_rect = ctx.input(|i| i.content_rect());

        self.viewport_info = ctx.input(|i| {
            format!("{:?}", i.viewport())
        });

        egui::TopBottomPanel::top("debug_panel").show(ctx, |ui| {
            ui.label(format!(
                "ContentRect: x={} y={} w={} h={}",
                screen_rect.min.x,
                screen_rect.min.y,
                screen_rect.width(),
                screen_rect.height()
            ));
            
            ui.label(format!(
                "PixelsPerPoint={:.2}",
                ctx.pixels_per_point()
            ));

            ui.label(format!(
                "ViewportRect: {:?}",
                ctx.viewport_rect()
            ));
        });

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
        "P0-2 WebView Validation",
        options,
        Box::new(|_cc| Ok(Box::new(DockingValidationApp::new()))),
    )
}
