//! WV-11-04 Browser Surface Pointer 座標変換 Probe。
//!
//! 役割:
//! - INPUT-IT-SPEC-001 の Dock 表示座標から Browser OSR 論理座標への変換を検証する。
//! - Dock Panel 内の Pointer 位置と、800x600 の Browser OSR 座標との対応を可視化する。
//!
//! 注意点:
//! - 本ファイルは技術検証用であり、正式 Surface API ではない。
//! - CEF への入力送信は INPUT-IT-SPEC-002 以降で検証する。
//! - 座標は表示矩形内で正規化し、Browser OSR の論理サイズへ変換する。

use eframe::egui;
use egui_dock::{DockArea, DockState, TabViewer};

const BROWSER_WIDTH: f32 = 800.0;
const BROWSER_HEIGHT: f32 = 600.0;

/// Dock 内の表示座標を Browser OSR 論理座標へ変換する。
///
/// # 引数
/// - `rect`: Browser Surface が表示されている Dock 内矩形。
/// - `position`: Dock 内の Pointer 座標。
/// - `browser_size`: Browser OSR の論理サイズ。
///
/// # 戻り値
/// - 表示矩形内なら Browser OSR 座標。
/// - 表示矩形外、または無効サイズなら `None`。
///
/// @hldocs.ref doc-20260912-010000Z-WV14#sec_v4c8n2q7m1px
fn map_pointer_to_browser(
    rect: egui::Rect,
    position: egui::Pos2,
    browser_size: egui::Vec2,
) -> Option<egui::Pos2> {
    if !rect.contains(position)
        || rect.width() <= 0.0
        || rect.height() <= 0.0
        || browser_size.x <= 0.0
        || browser_size.y <= 0.0
    {
        return None;
    }

    let normalized_x = ((position.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
    let normalized_y = ((position.y - rect.top()) / rect.height()).clamp(0.0, 1.0);

    Some(egui::pos2(
        normalized_x * browser_size.x,
        normalized_y * browser_size.y,
    ))
}

/// 座標変換の境界値を検証する。
///
/// # 戻り値
/// - 期待値と一致する場合 `Ok(())`。
/// - 不一致の場合は理由を含む `Err`。
fn verify_mapping() -> Result<(), String> {
    let rect = egui::Rect::from_min_size(egui::pos2(100.0, 50.0), egui::vec2(400.0, 300.0));
    let browser = egui::vec2(BROWSER_WIDTH, BROWSER_HEIGHT);

    let cases = [
        ("top-left", egui::pos2(100.0, 50.0), egui::pos2(0.0, 0.0)),
        ("center", egui::pos2(300.0, 200.0), egui::pos2(400.0, 300.0)),
        ("bottom-right", egui::pos2(500.0, 350.0), egui::pos2(800.0, 600.0)),
    ];

    for (name, input, expected) in cases {
        let Some(actual) = map_pointer_to_browser(rect, input, browser) else {
            return Err(format!("{name}: coordinate was rejected"));
        };
        if (actual.x - expected.x).abs() > 0.01 || (actual.y - expected.y).abs() > 0.01 {
            return Err(format!(
                "{name}: expected ({:.2},{:.2}), actual ({:.2},{:.2})",
                expected.x, expected.y, actual.x, actual.y
            ));
        }
    }

    if map_pointer_to_browser(rect, egui::pos2(99.0, 49.0), browser).is_some() {
        return Err("outside: coordinate outside Browser Surface was accepted".to_string());
    }

    Ok(())
}

#[derive(Clone)]
enum InputCoordTab {
    Browser,
}

struct InputCoordViewer<'a> {
    pointer_status: &'a mut String,
}

impl<'a> TabViewer for InputCoordViewer<'a> {
    type Tab = InputCoordTab;

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn title(&mut self, _tab: &mut Self::Tab) -> egui::WidgetText {
        "Browser Surface Input Coordinates".into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, _tab: &mut Self::Tab) {
        let available = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(available, egui::Sense::hover());

        ui.painter().rect_filled(rect, 0.0, egui::Color32::from_rgb(30, 45, 65));
        ui.painter().rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(3.0, egui::Color32::WHITE),
            egui::StrokeKind::Inside,
        );

        let browser_size = egui::vec2(BROWSER_WIDTH, BROWSER_HEIGHT);
        if let Some(pointer) = response.hover_pos() {
            if let Some(browser) = map_pointer_to_browser(rect, pointer, browser_size) {
                *self.pointer_status = format!(
                    "Dock local: ({:.1}, {:.1}) | Browser: ({:.1}, {:.1})",
                    pointer.x - rect.left(),
                    pointer.y - rect.top(),
                    browser.x,
                    browser.y
                );

                ui.painter().circle_filled(pointer, 6.0, egui::Color32::YELLOW);
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("Browser OSR\n800 x 600\n\n({:.0}, {:.0})", browser.x, browser.y),
                    egui::FontId::proportional(32.0),
                    egui::Color32::WHITE,
                );
            }
        } else {
            *self.pointer_status = "Pointer outside Browser Surface".to_string();
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Browser OSR\n800 x 600",
                egui::FontId::proportional(32.0),
                egui::Color32::WHITE,
            );
        }
    }
}

struct InputCoordApp {
    dock_state: DockState<InputCoordTab>,
    pointer_status: String,
}

impl InputCoordApp {
    fn new() -> Self {
        Self {
            dock_state: DockState::new(vec![InputCoordTab::Browser]),
            pointer_status: "Pointer outside Browser Surface".to_string(),
        }
    }
}

impl eframe::App for InputCoordApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("wv11_04_coord_status").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("WV-11-04 INPUT-IT-SPEC-001");
                ui.separator();
                ui.label("Browser OSR: 800x600");
                ui.separator();
                ui.label(&self.pointer_status);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut viewer = InputCoordViewer {
                pointer_status: &mut self.pointer_status,
            };
            DockArea::new(&mut self.dock_state).show_inside(ui, &mut viewer);
        });

        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    println!("WV-11-04 Pointer coordinate probe start");
    match verify_mapping() {
        Ok(()) => println!("Pointer coordinate mapping self-check OK"),
        Err(error) => {
            eprintln!("Pointer coordinate mapping self-check FAILED: {error}");
            std::process::exit(1);
        }
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("WV-11-04 Pointer Coordinate Probe")
            .with_inner_size([1000.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "WV-11-04 Pointer Coordinate Probe",
        native_options,
        Box::new(move |_cc| Ok(Box::new(InputCoordApp::new()))),
    )
}
