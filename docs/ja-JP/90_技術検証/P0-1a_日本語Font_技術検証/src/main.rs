//! P0-1a 日本語Font 技術検証
//!
//! 目的:
//! - egui 日本語表示確認
//! - Noto CJK font load 確認
//! - Docking UI 日本語確認

use eframe::egui;
use egui::{FontData, FontDefinitions, FontFamily};
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
            }
            PanelTab::ログ => {
                ui.heading("ログ");
                ui.label("イベントログ表示予定");
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
        let mut fonts = FontDefinitions::default();

        fonts.font_data.insert(
            "noto_sans_jp".to_owned(),
            FontData::from_owned(
                std::fs::read(
                    "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
                )
                .expect("failed to load Noto font"),
            )
            .into(),
        );

        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "noto_sans_jp".to_owned());

        cc.egui_ctx.set_fonts(fonts);

        let dock_state = DockState::new(vec![
            PanelTab::状態,
            PanelTab::ログ,
        ]);

        Self { dock_state }
    }
}

impl eframe::App for ValidationApp {
    /// UI 更新
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        DockArea::new(&mut self.dock_state)
            .show(ctx, &mut ValidationTabViewer);
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
