//! P0-1c Linux Native Window Title 技術検証
//!
//! 目的:
//! - Linux native title 日本語確認
//! - Wayland/X11 差異確認
//! - locale 確認

use std::fs;
use std::path::Path;

use eframe::egui;

/// title mode
#[derive(Debug)]
enum TitleMode {
    /// ASCII only
    Ascii,

    /// 日本語 only
    Japanese,

    /// mixed
    Mixed,
}

impl TitleMode {
    /// title 取得
    fn window_title(&self) -> &'static str {
        match self {
            TitleMode::Ascii => "P0-1c ASCII TITLE",
            TitleMode::Japanese => "日本語タイトル検証",
            TitleMode::Mixed => "P0-1c Linux 日本語タイトル検証",
        }
    }
}

/// 検証アプリ
#[derive(Default)]
struct ValidationApp;

impl eframe::App for ValidationApp {
    /// UI 更新
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("Linux Native Window Title 技術検証");

        ui.separator();

        ui.label("日本語 Panel 表示確認");
        ui.label("ASCII title / 日本語 title / mixed title");

        ui.separator();

        ui.label(format!(
            "LANG: {}",
            std::env::var("LANG").unwrap_or_else(|_| "<undefined>".to_owned())
        ));

        ui.label(format!(
            "LC_ALL: {}",
            std::env::var("LC_ALL").unwrap_or_else(|_| "<undefined>".to_owned())
        ));

        ui.label(format!(
            "XDG_SESSION_TYPE: {}",
            std::env::var("XDG_SESSION_TYPE")
                .unwrap_or_else(|_| "<undefined>".to_owned())
        ));
    }
}

/// EmbeddedFont optional load
fn setup_optional_embedded_font(ctx: &egui::Context) {
    let font_path = Path::new("assets/fonts/default/NotoSansCJKjp-Regular.otf");

    if !font_path.exists() {
        println!("EmbeddedFont not found: fallback to OS default font");
        return;
    }

    println!("Loading EmbeddedFont: {}", font_path.display());

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

    println!("EmbeddedFont loaded successfully");
}

/// エントリーポイント
fn main() -> eframe::Result<()> {
    println!("LANG={:?}", std::env::var("LANG"));
    println!("LC_ALL={:?}", std::env::var("LC_ALL"));
    println!("XDG_SESSION_TYPE={:?}", std::env::var("XDG_SESSION_TYPE"));

    // TODO:
    // 将来 command line argument 化予定
    let title_mode = TitleMode::Mixed;

    println!("TitleMode={title_mode:?}");

    let options = eframe::NativeOptions::default();

    eframe::run_native(
        title_mode.window_title(),
        options,
        Box::new(|cc| {
            setup_optional_embedded_font(&cc.egui_ctx);
            Ok(Box::new(ValidationApp))
        }),
    )
}
