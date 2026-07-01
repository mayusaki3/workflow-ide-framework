//! P0-1b EmbeddedFont 技術検証
//!
//! 目的:
//! - Embedded Font 検証
//! - 日本語表示統一
//! - cross-platform 一致確認
//!
//! 注意:
//! - 本コードは技術検証用
//! - include_bytes! による font 埋め込みを想定
//! - font asset は repository へ含めない

use std::path::Path;

use eframe::egui;
use egui::{FontData, FontDefinitions, FontFamily};
use egui_dock::{DockArea, DockState, TabViewer};

/// Embedded Font path
const FONT_PATH: &str = "assets/fonts/default/NotoSansCJK-Regular.ttc";

/// Dock Tab
#[derive(Debug, Clone)]
enum PanelTab {
    /// 状態
    状態,

    /// Font
    Font,

    /// ログ
    ログ,
}

/// Dock Viewer
struct ValidationTabViewer;

impl TabViewer for ValidationTabViewer {
    type Tab = PanelTab;

    /// タイトル
    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            PanelTab::状態 => "状態".into(),
            PanelTab::Font => "Font".into(),
            PanelTab::ログ => "ログ".into(),
        }
    }

    /// UI 描画
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            PanelTab::状態 => {
                ui.heading("EmbeddedFont 技術検証");
                ui.label("日本語表示確認");
                ui.label("cross-platform UI");
                ui.label("Embedded Font 構造確認");
            }
            PanelTab::Font => {
                ui.heading("Font");
                ui.label("Framework default font");
                ui.label("Workspace font");
                ui.label("Custom font");
                ui.label("Runtime font reload");
                ui.separator();
                ui.label(format!("Font Path: {FONT_PATH}"));
            }
            PanelTab::ログ => {
                ui.heading("ログ");
                ui.label("日本語ログ確認");
                ui.label("Font reload log");
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
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Embedded Font 読み込み
        if Path::new(FONT_PATH).exists() {
            match std::fs::read(FONT_PATH) {
                Ok(font_bytes) => {
                    let mut fonts = FontDefinitions::default();

                    fonts.font_data.insert(
                        "embedded_font".to_owned(),
                        FontData::from_owned(font_bytes).into(),
                    );

                    fonts
                        .families
                        .entry(FontFamily::Proportional)
                        .or_default()
                        .insert(0, "embedded_font".to_owned());

                    cc.egui_ctx.set_fonts(fonts);

                    println!("Embedded Font Loaded");
                }
                Err(error) => {
                    println!("Failed to load font: {error}");
                }
            }
        } else {
            println!("Font asset not found: {FONT_PATH}");
            println!("Run setup_fonts script before execution");
        }

        let dock_state = DockState::new(vec![
            PanelTab::状態,
            PanelTab::Font,
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
        "P0-1b EmbeddedFont 技術検証",
        options,
        Box::new(|cc| Ok(Box::new(ValidationApp::new(cc)))),
    )
}
