//! WV-12-04 GPU Surface 入力イベント転送 Probe。
//!
//! Dock 内の GPU Surface 領域で egui が受けた入力を、GPU Surface 側の
//! 実験用入力状態へ転送できることを確認する。
//! 正式 Surface API ではない。

use eframe::egui;
use egui_dock::{DockArea, DockState, TabViewer};

#[derive(Debug)]
enum ProbeTab {
    GpuSurface,
    Info,
}

#[derive(Default)]
struct InputState {
    pointer_move: u64,
    pointer_button: u64,
    wheel: u64,
    keyboard: u64,
    focus: u64,
    focused: bool,
    last_event: String,
}

struct ProbeApp {
    dock_state: DockState<ProbeTab>,
    input: InputState,
}

impl ProbeApp {
    fn new() -> Self {
        println!("WV-12-04 GPU Surface input probe start");
        Self {
            dock_state: DockState::new(vec![ProbeTab::GpuSurface, ProbeTab::Info]),
            input: InputState::default(),
        }
    }
}

struct ProbeViewer<'a> {
    input: &'a mut InputState,
}

impl ProbeViewer<'_> {
    fn surface_ui(&mut self, ui: &mut egui::Ui) {
        let available = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(available, egui::Sense::click_and_drag());

        let fill = if self.input.focused {
            egui::Color32::from_rgb(55, 80, 120)
        } else {
            egui::Color32::from_rgb(45, 45, 50)
        };
        ui.painter().rect_filled(rect, 4.0, fill);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "GPU Surface Input Area\nClick here to focus",
            egui::FontId::proportional(24.0),
            egui::Color32::WHITE,
        );

        if response.hovered() {
            if let Some(pos) = ui.ctx().pointer_hover_pos() {
                self.input.pointer_move += 1;
                self.input.last_event = format!("PointerMove x={:.1} y={:.1}", pos.x - rect.left(), pos.y - rect.top());
            }

            let scroll = ui.input(|i| i.raw_scroll_delta);
            if scroll != egui::Vec2::ZERO {
                self.input.wheel += 1;
                self.input.last_event = format!("Wheel x={:.1} y={:.1}", scroll.x, scroll.y);
                println!("GPU Surface input: {}", self.input.last_event);
            }
        }

        if response.clicked() {
            self.input.pointer_button += 1;
            self.input.last_event = "PointerButton click".to_owned();
            println!("GPU Surface input: PointerButton click");
            response.request_focus();
        }

        let now_focused = response.has_focus();
        if now_focused != self.input.focused {
            self.input.focused = now_focused;
            self.input.focus += 1;
            self.input.last_event = format!("Focus {}", now_focused);
            println!("GPU Surface input: Focus {}", now_focused);
        }

        if now_focused {
            let events = ui.input(|i| i.events.clone());
            for event in events {
                match event {
                    egui::Event::Key { key, pressed, repeat, modifiers, .. } => {
                        self.input.keyboard += 1;
                        self.input.last_event = format!(
                            "Key {:?} pressed={} repeat={} modifiers={:?}",
                            key, pressed, repeat, modifiers
                        );
                        println!("GPU Surface input: {}", self.input.last_event);
                    }
                    egui::Event::Text(text) => {
                        self.input.keyboard += 1;
                        self.input.last_event = format!("Text {:?}", text);
                        println!("GPU Surface input: {}", self.input.last_event);
                    }
                    _ => {}
                }
            }
        }
    }

    fn info_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("WV-12-04 GPU Surface Input");
        ui.label(format!("PointerMove: {}", self.input.pointer_move));
        ui.label(format!("PointerButton: {}", self.input.pointer_button));
        ui.label(format!("Wheel: {}", self.input.wheel));
        ui.label(format!("Keyboard/Text: {}", self.input.keyboard));
        ui.label(format!("Focus transitions: {}", self.input.focus));
        ui.label(format!("Focused: {}", self.input.focused));
        ui.separator();
        ui.label(format!("Last event: {}", self.input.last_event));
        ui.separator();
        ui.label("確認:");
        ui.label("1. GPU Surface 上でマウスを動かす -> PointerMove が増える");
        ui.label("2. GPU Surface をクリック -> PointerButton が増え Focused=true になる");
        ui.label("3. GPU Surface 上でホイール -> Wheel が増える");
        ui.label("4. Focus 後にキー入力 -> Keyboard/Text が増える");
        ui.label("5. 別 UI をクリックして戻る -> Focus transitions が増える");
    }
}

impl TabViewer for ProbeViewer<'_> {
    type Tab = ProbeTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            ProbeTab::GpuSurface => "GPU Surface".into(),
            ProbeTab::Info => "Input Status".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            ProbeTab::GpuSurface => self.surface_ui(ui),
            ProbeTab::Info => self.info_ui(ui),
        }
    }
}

impl eframe::App for ProbeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut viewer = ProbeViewer { input: &mut self.input };
            DockArea::new(&mut self.dock_state).show_inside(ui, &mut viewer);
        });
        ctx.request_repaint();
    }
}

impl Drop for ProbeApp {
    fn drop(&mut self) {
        println!(
            "WV-12-04 result: pointer_move={}, pointer_button={}, wheel={}, keyboard={}, focus={}",
            self.input.pointer_move,
            self.input.pointer_button,
            self.input.wheel,
            self.input.keyboard,
            self.input.focus
        );
        println!("WV-12-04 GPU Surface input probe shutdown");
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_title("WV-12-04 GPU Surface Input Probe")
            .with_inner_size([1000.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "WV-12-04 GPU Surface Input Probe",
        options,
        Box::new(|_cc| Ok(Box::new(ProbeApp::new()))),
    )
}
