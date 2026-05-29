//! P0-1e Custom Title Bar 技術検証

use eframe::egui;
use std::fs;
use std::path::Path;

#[derive(Default)]
struct ValidationApp;

fn setup_optional_embedded_font(ctx: &egui::Context) {
    let font_path = Path::new("assets/fonts/default/NotoSansCJKjp-Regular.otf");

    if !font_path.exists() {
        println!("EmbeddedFont not found");
        return;
    }

    let font_data = match fs::read(font_path) {
        Ok(data) => data,
        Err(error) => {
            println!("EmbeddedFont load failed: {error}");
            return;
        }
    };

    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "embedded_noto_jp".to_owned(),
        egui::FontData::from_owned(font_data).into(),
    );

    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "embedded_noto_jp".to_owned());

    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("embedded_noto_jp".to_owned());

    ctx.set_fonts(fonts);
}

impl eframe::App for ValidationApp {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _frame: &mut eframe::Frame,
    ) {
        //
        // Custom Title Bar (描画のみ)
        //
        egui::Frame::default()
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| {

                ui.horizontal(|ui| {

                    //
                    // タイトル
                    //
                    ui.heading("Workflow 日本語IDE");

                    //
                    // 右寄せ
                    //
                    ui.add_space(
                        (ui.available_width() - 120.0).max(0.0)
                    );

                    //
                    // Window Control (まだ動作なし)
                    //
                    let _ = ui.small_button("－");
                    let _ = ui.small_button("□");
                    let _ = ui.small_button("×");
                });
            });

        ui.separator();

        //
        // 検証領域
        //
        ui.heading("P0-1e Custom Title Bar Validation");

        ui.separator();

        ui.label("日本語タイトル表示");
        ui.label("native title 非依存確認");
        ui.label("Windows / Linux / macOS 共通UI確認");
    }
}

fn main() -> eframe::Result<()> {
    let viewport = egui::ViewportBuilder::default()
        .with_decorations(false)
        .with_title("P0-1e Custom Title Bar Validation");

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "P0-1e",
        options,
        Box::new(|cc| {
            setup_optional_embedded_font(&cc.egui_ctx);
            Ok(Box::new(ValidationApp::default()))
        })
    )
}