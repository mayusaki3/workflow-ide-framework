use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerMode { Operate, Edit }

#[derive(Debug, Clone)]
pub enum ControllerElementKind {
    Button { text: String },
    Joystick { value: [f32; 2] },
    Slider { value: f32, min: f32, max: f32 },
    Label { text: String },
    Image { source: String },
    Line { to: [f32; 2], width: f32 },
}

#[derive(Debug, Clone)]
pub struct ControllerElement {
    pub id: String,
    pub label: String,
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub kind: ControllerElementKind,
}

impl ControllerElement {
    pub fn new(id: impl Into<String>, label: impl Into<String>, position: [f32; 2], size: [f32; 2], kind: ControllerElementKind) -> Self {
        Self { id: id.into(), label: label.into(), position, size, kind }
    }
}

#[derive(Debug, Clone)]
pub struct ControllerModel {
    pub mode: ControllerMode,
    pub canvas_size: [f32; 2],
    pub elements: Vec<ControllerElement>,
    pub selected_id: Option<String>,
}

impl Default for ControllerModel {
    fn default() -> Self {
        Self { mode: ControllerMode::Operate, canvas_size: [800.0, 520.0], elements: Vec::new(), selected_id: None }
    }
}

#[derive(Debug, Clone)]
pub enum ControllerAction {
    ButtonPressed { element_id: String },
    JoystickChanged { element_id: String, value: [f32; 2] },
    SliderChanged { element_id: String, value: f32 },
    ElementSelected { element_id: String },
    ElementMoved { element_id: String, position: [f32; 2] },
}

#[derive(Debug, Default)]
pub struct ControllerResponse { pub actions: Vec<ControllerAction> }

pub fn show(ui: &mut egui::Ui, model: &mut ControllerModel) -> ControllerResponse {
    let mut out = ControllerResponse::default();
    ui.horizontal(|ui| {
        ui.label("モード:");
        ui.selectable_value(&mut model.mode, ControllerMode::Operate, "操作");
        ui.selectable_value(&mut model.mode, ControllerMode::Edit, "レイアウト編集");
    });
    ui.separator();

    egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
        let (canvas, _) = ui.allocate_exact_size(egui::vec2(model.canvas_size[0], model.canvas_size[1]), egui::Sense::hover());
        ui.painter().rect_stroke(canvas, 0.0, ui.visuals().widgets.noninteractive.bg_stroke, egui::StrokeKind::Inside);

        for element in &mut model.elements {
            let pos = canvas.min + egui::vec2(element.position[0], element.position[1]);
            let size = egui::vec2(element.size[0], element.size[1]);
            let rect = egui::Rect::from_min_size(pos, size);
            let id = ui.make_persistent_id(("wfide_controller", &element.id));
            let sense = if model.mode == ControllerMode::Edit { egui::Sense::click_and_drag() } else { egui::Sense::click() };
            let response = ui.interact(rect, id, sense);

            if model.mode == ControllerMode::Edit {
                if response.clicked() {
                    model.selected_id = Some(element.id.clone());
                    out.actions.push(ControllerAction::ElementSelected { element_id: element.id.clone() });
                }
                if response.dragged() {
                    let d = response.drag_delta();
                    element.position[0] += d.x;
                    element.position[1] += d.y;
                    out.actions.push(ControllerAction::ElementMoved { element_id: element.id.clone(), position: element.position });
                }
            }

            match &mut element.kind {
                ControllerElementKind::Button { text } => {
                    if ui.put(rect, egui::Button::new(text.as_str())).clicked() && model.mode == ControllerMode::Operate {
                        out.actions.push(ControllerAction::ButtonPressed { element_id: element.id.clone() });
                    }
                }
                ControllerElementKind::Slider { value, min, max } => {
                    let before = *value;
                    ui.put(rect, egui::Slider::new(value, *min..=*max).text(&element.label));
                    if model.mode == ControllerMode::Operate && *value != before {
                        out.actions.push(ControllerAction::SliderChanged { element_id: element.id.clone(), value: *value });
                    }
                }
                ControllerElementKind::Joystick { value } => {
                    let painter = ui.painter();
                    painter.rect_stroke(rect, 4.0, ui.visuals().widgets.noninteractive.bg_stroke, egui::StrokeKind::Inside);
                    let center = rect.center();
                    let radius = rect.width().min(rect.height()) * 0.35;
                    painter.circle_stroke(center, radius, ui.visuals().widgets.noninteractive.fg_stroke);
                    painter.circle_filled(center + egui::vec2(value[0], -value[1]) * radius, 7.0, ui.visuals().selection.bg_fill);
                    if model.mode == ControllerMode::Operate && response.dragged() {
                        let p = response.interact_pointer_pos().unwrap_or(center);
                        value[0] = ((p.x - center.x) / radius).clamp(-1.0, 1.0);
                        value[1] = (-(p.y - center.y) / radius).clamp(-1.0, 1.0);
                        out.actions.push(ControllerAction::JoystickChanged { element_id: element.id.clone(), value: *value });
                    }
                }
                ControllerElementKind::Label { text } => {
                    ui.painter().text(rect.left_center(), egui::Align2::LEFT_CENTER, text, egui::TextStyle::Body.resolve(ui.style()), ui.visuals().text_color());
                }
                ControllerElementKind::Image { source } => {
                    ui.painter().rect_stroke(rect, 2.0, ui.visuals().widgets.noninteractive.bg_stroke, egui::StrokeKind::Inside);
                    ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, format!("画像\n{source}"), egui::TextStyle::Small.resolve(ui.style()), ui.visuals().weak_text_color());
                }
                ControllerElementKind::Line { to, width } => {
                    let end = canvas.min + egui::vec2(to[0], to[1]);
                    ui.painter().line_segment([pos, end], egui::Stroke::new(*width, ui.visuals().text_color()));
                }
            }
            if model.mode == ControllerMode::Edit && model.selected_id.as_deref() == Some(element.id.as_str()) {
                ui.painter().rect_stroke(rect, 2.0, ui.visuals().selection.stroke, egui::StrokeKind::Outside);
            }
        }
    });
    out
}
