use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerMode { Operate, Edit }

#[derive(Debug, Clone)]
pub enum ControllerElementKind {
    Button { text: String, image_source: Option<String> },
    Joystick { value: [f32; 2], return_to_center: bool },
    Slider { value: f32, min: f32, max: f32 },
    Label { text: String },
    Image { source: String },
    Line { to: [f32; 2], width: f32 },
}

#[derive(Debug, Clone)]
pub struct ControllerElement {
    pub id: String, pub label: String, pub position: [f32; 2], pub size: [f32; 2],
    pub foreground: [u8; 4], pub background: [u8; 4],
    pub kind: ControllerElementKind,
}
impl ControllerElement {
    pub fn new(id: impl Into<String>, label: impl Into<String>, position: [f32; 2], size: [f32; 2], kind: ControllerElementKind) -> Self {
        Self { id: id.into(), label: label.into(), position, size, foreground: [220, 220, 220, 255], background: [60, 60, 60, 255], kind }
    }
}

#[derive(Debug, Clone)]
pub struct ControllerModel {
    pub mode: ControllerMode,
    /// Logical design size. Rendering is automatically fitted into the panel.
    pub canvas_size: [f32; 2],
    pub background: [u8; 4],
    pub elements: Vec<ControllerElement>,
    pub selected_id: Option<String>,
}
impl Default for ControllerModel {
    fn default() -> Self { Self { mode: ControllerMode::Operate, canvas_size: [800.0, 520.0], background: [30, 30, 30, 255], elements: Vec::new(), selected_id: None } }
}

#[derive(Debug, Clone)]
pub enum ControllerAction {
    ButtonPressed { element_id: String },
    JoystickChanged { element_id: String, value: [f32; 2] },
    SliderChanged { element_id: String, value: f32 },
    ElementSelected { element_id: String },
    CanvasSelected,
    ElementMoved { element_id: String, position: [f32; 2] },
    ElementResized { element_id: String, size: [f32; 2] },
    CanvasResized { size: [f32; 2] },
}
#[derive(Debug, Default)]
pub struct ControllerResponse { pub actions: Vec<ControllerAction> }

pub fn show(ui: &mut egui::Ui, model: &mut ControllerModel) -> ControllerResponse {
    let mut out = ControllerResponse::default();
    ui.horizontal(|ui| {
        ui.label("モード:");
        ui.selectable_value(&mut model.mode, ControllerMode::Operate, "操作");
        ui.selectable_value(&mut model.mode, ControllerMode::Edit, "レイアウト編集");
        if model.mode == ControllerMode::Edit {
            ui.separator();
            ui.label("キャンバス");
            let mut w = model.canvas_size[0];
            let mut h = model.canvas_size[1];
            let wc = ui.add(egui::DragValue::new(&mut w).range(100.0..=10000.0).prefix("W ")).changed();
            let hc = ui.add(egui::DragValue::new(&mut h).range(100.0..=10000.0).prefix("H ")).changed();
            if wc || hc {
                model.canvas_size = [w, h];
                out.actions.push(ControllerAction::CanvasResized { size: model.canvas_size });
            }
        }
    });
    ui.separator();

    let available = ui.available_size();
    let logical = egui::vec2(model.canvas_size[0].max(1.0), model.canvas_size[1].max(1.0));
    let scale = (available.x / logical.x).min(available.y / logical.y).min(1.0).max(0.05);
    let display = logical * scale;
    let (canvas, canvas_response) = ui.allocate_exact_size(display, egui::Sense::click());
    let canvas_bg = egui::Color32::from_rgba_unmultiplied(model.background[0], model.background[1], model.background[2], model.background[3]);
    ui.painter().rect_filled(canvas, 0.0, canvas_bg);
    ui.painter().rect_stroke(canvas, 0.0, ui.visuals().widgets.noninteractive.bg_stroke, egui::StrokeKind::Inside);
    if model.mode == ControllerMode::Edit && canvas_response.clicked() {
        model.selected_id = None;
        out.actions.push(ControllerAction::CanvasSelected);
    }

    for element in &mut model.elements {
        let pos = canvas.min + egui::vec2(element.position[0], element.position[1]) * scale;
        let size = egui::vec2(element.size[0], element.size[1]) * scale;
        let rect = egui::Rect::from_min_size(pos, size);
        let id = ui.make_persistent_id(("wfide_controller", &element.id));
        let response = ui.interact(rect, id, egui::Sense::click_and_drag());
        let fg = egui::Color32::from_rgba_unmultiplied(element.foreground[0], element.foreground[1], element.foreground[2], element.foreground[3]);
        let bg = egui::Color32::from_rgba_unmultiplied(element.background[0], element.background[1], element.background[2], element.background[3]);

        if model.mode == ControllerMode::Edit {
            if response.clicked() || response.drag_started() {
                model.selected_id = Some(element.id.clone());
                out.actions.push(ControllerAction::ElementSelected { element_id: element.id.clone() });
            }
            if response.dragged() {
                let d = ui.input(|i| i.pointer.delta()) / scale;
                element.position[0] += d.x;
                element.position[1] += d.y;
                out.actions.push(ControllerAction::ElementMoved { element_id: element.id.clone(), position: element.position });
            }
        }

        match &mut element.kind {
            ControllerElementKind::Button { text, image_source } => {
                let clicked = if model.mode == ControllerMode::Operate {
                    ui.put(rect, egui::Button::new(text.as_str())).clicked()
                } else {
                    ui.painter().rect_filled(rect, 2.0, bg);
                    ui.painter().rect_stroke(rect, 2.0, ui.visuals().widgets.inactive.bg_stroke, egui::StrokeKind::Inside);
                    ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, text.as_str(), egui::TextStyle::Button.resolve(ui.style()), fg);
                    false
                };
                if let Some(source) = image_source {
                    ui.painter().text(rect.center_top() + egui::vec2(0.0, 3.0 * scale), egui::Align2::CENTER_TOP,
                        format!("画像: {source}"), egui::TextStyle::Small.resolve(ui.style()), ui.visuals().weak_text_color());
                }
                if clicked && model.mode == ControllerMode::Operate {
                    out.actions.push(ControllerAction::ButtonPressed { element_id: element.id.clone() });
                }
            }
            ControllerElementKind::Slider { value, min, max } => {
                let before = *value;
                if model.mode == ControllerMode::Operate {
                    ui.put(rect, egui::Slider::new(value, *min..=*max).text(&element.label));
                } else {
                    ui.add_enabled_ui(false, |ui| {
                        ui.put(rect, egui::Slider::new(value, *min..=*max).text(&element.label));
                    });
                }
                if model.mode == ControllerMode::Operate && *value != before {
                    out.actions.push(ControllerAction::SliderChanged { element_id: element.id.clone(), value: *value });
                }
            }
            ControllerElementKind::Joystick { value, return_to_center } => {
                let painter = ui.painter();
                painter.rect_stroke(rect, 4.0, ui.visuals().widgets.noninteractive.bg_stroke, egui::StrokeKind::Inside);
                let center = rect.center();
                let radius = rect.width().min(rect.height()) * 0.35;
                painter.circle_filled(center, radius, bg);
                painter.circle_stroke(center, radius, egui::Stroke::new(1.0, fg));
                painter.circle_filled(center + egui::vec2(value[0], -value[1]) * radius, 7.0 * scale.max(0.5), fg);
                if model.mode == ControllerMode::Operate && response.dragged() {
                    let p = response.interact_pointer_pos().unwrap_or(center);
                    let mut next = egui::vec2((p.x - center.x) / radius, -(p.y - center.y) / radius);
                    let length = next.length();
                    if length > 1.0 {
                        next /= length;
                    }
                    value[0] = next.x;
                    value[1] = next.y;
                    out.actions.push(ControllerAction::JoystickChanged { element_id: element.id.clone(), value: *value });
                }
                if model.mode == ControllerMode::Operate && *return_to_center && response.drag_stopped() {
                    *value = [0.0, 0.0];
                    out.actions.push(ControllerAction::JoystickChanged { element_id: element.id.clone(), value: *value });
                }
            }
            ControllerElementKind::Label { text } => {
                ui.painter().text(rect.left_center(), egui::Align2::LEFT_CENTER, text, egui::TextStyle::Body.resolve(ui.style()), fg);
            }
            ControllerElementKind::Image { source } => {
                ui.painter().rect_stroke(rect, 2.0, ui.visuals().widgets.noninteractive.bg_stroke, egui::StrokeKind::Inside);
                let clipped = ui.painter().with_clip_rect(rect.shrink(2.0));
                if rect.width() >= 48.0 && rect.height() >= 28.0 {
                    clipped.text(rect.center(), egui::Align2::CENTER_CENTER, format!("画像\n{source}"), egui::TextStyle::Small.resolve(ui.style()), ui.visuals().weak_text_color());
                }
            }
            ControllerElementKind::Line { to, width } => {
                let end = canvas.min + egui::vec2(to[0], to[1]) * scale;
                ui.painter().line_segment([pos, end], egui::Stroke::new(*width * scale, fg));
            }
        }

        if model.mode == ControllerMode::Edit && model.selected_id.as_deref() == Some(element.id.as_str()) {
            ui.painter().rect_stroke(rect, 2.0, ui.visuals().selection.stroke, egui::StrokeKind::Outside);
            let hs = 10.0;
            let handle = egui::Rect::from_center_size(rect.right_bottom(), egui::vec2(hs, hs));
            ui.painter().rect_filled(handle, 1.0, fg);
            let resize = ui.interact(handle, id.with("resize"), egui::Sense::drag());
            if resize.dragged() {
                let d = ui.input(|i| i.pointer.delta()) / scale;
                element.size[0] = (element.size[0] + d.x).max(12.0);
                element.size[1] = (element.size[1] + d.y).max(12.0);
                out.actions.push(ControllerAction::ElementResized { element_id: element.id.clone(), size: element.size });
            }
        }
    }
    out
}

pub fn property_model_for_element(element: &ControllerElement) -> crate::property_panel::PropertyModel {
    use crate::property_panel::{PropertyGroup, PropertyItem, PropertyModel, PropertyValue};
    let mut items = vec![
        PropertyItem::new("label", "ラベル", PropertyValue::Text(element.label.clone())),
        PropertyItem::new("position.x", "X", PropertyValue::Float(element.position[0] as f64)),
        PropertyItem::new("position.y", "Y", PropertyValue::Float(element.position[1] as f64)),
        PropertyItem::new("size.w", "幅", PropertyValue::Float(element.size[0] as f64)),
        PropertyItem::new("size.h", "高さ", PropertyValue::Float(element.size[1] as f64)),
        PropertyItem::new("foreground", "前景色", PropertyValue::Color(element.foreground)),
        PropertyItem::new("background", "背景色", PropertyValue::Color(element.background)),
    ];
    match &element.kind {
        ControllerElementKind::Button { text, image_source } => {
            items.push(PropertyItem::new("button.text", "文字", PropertyValue::Text(text.clone())));
            items.push(PropertyItem::new("button.image", "画像", PropertyValue::FilePath(image_source.clone().unwrap_or_default())));
        }
        ControllerElementKind::Joystick { value, return_to_center } => {
            items.push(PropertyItem::new("joystick.x", "値 X", PropertyValue::Float(value[0] as f64)).read_only(true));
            items.push(PropertyItem::new("joystick.y", "値 Y", PropertyValue::Float(value[1] as f64)).read_only(true));
            items.push(PropertyItem::new("joystick.return_to_center", "離したら中央へ戻る", PropertyValue::Bool(*return_to_center)));
        }
        ControllerElementKind::Slider { value, min, max } => {
            items.push(PropertyItem::new("slider.value", "値", PropertyValue::Float(*value as f64)));
            items.push(PropertyItem::new("slider.min", "最小", PropertyValue::Float(*min as f64)));
            items.push(PropertyItem::new("slider.max", "最大", PropertyValue::Float(*max as f64)));
        }
        ControllerElementKind::Label { text } => items.push(PropertyItem::new("label.text", "文字", PropertyValue::Text(text.clone()))),
        ControllerElementKind::Image { source } => items.push(PropertyItem::new("image.source", "画像", PropertyValue::FilePath(source.clone()))),
        ControllerElementKind::Line { to, width } => {
            items.push(PropertyItem::new("line.to.x", "終点 X", PropertyValue::Float(to[0] as f64)));
            items.push(PropertyItem::new("line.to.y", "終点 Y", PropertyValue::Float(to[1] as f64)));
            items.push(PropertyItem::new("line.width", "線幅", PropertyValue::Float(*width as f64)));
        }
    }
    PropertyModel {
        object_id: Some(element.id.clone()),
        display_name: Some(format!("{} プロパティ", element.label)),
        groups: vec![PropertyGroup { label: "Controller Element".into(), items }],
    }
}

pub fn apply_property_action(element: &mut ControllerElement, action: &crate::property_panel::PropertyAction) {
    use crate::property_panel::{PropertyAction, PropertyValue};
    let PropertyAction::ValueChanged { property_id, value } = action;
    match (property_id.as_str(), value) {
        ("label", PropertyValue::Text(v)) => element.label = v.clone(),
        ("position.x", PropertyValue::Float(v)) => element.position[0] = *v as f32,
        ("position.y", PropertyValue::Float(v)) => element.position[1] = *v as f32,
        ("size.w", PropertyValue::Float(v)) => element.size[0] = (*v as f32).max(12.0),
        ("size.h", PropertyValue::Float(v)) => element.size[1] = (*v as f32).max(12.0),
        ("foreground", PropertyValue::Color(v)) => element.foreground = *v,
        ("background", PropertyValue::Color(v)) => element.background = *v,
        ("button.text", PropertyValue::Text(v)) => if let ControllerElementKind::Button { text, .. } = &mut element.kind { *text = v.clone(); },
        ("button.image", PropertyValue::FilePath(v)) => if let ControllerElementKind::Button { image_source, .. } = &mut element.kind { *image_source = if v.is_empty() { None } else { Some(v.clone()) }; },
        ("joystick.return_to_center", PropertyValue::Bool(v)) => if let ControllerElementKind::Joystick { return_to_center, .. } = &mut element.kind { *return_to_center = *v; },
        ("slider.value", PropertyValue::Float(v)) => if let ControllerElementKind::Slider { value, .. } = &mut element.kind { *value = *v as f32; },
        ("slider.min", PropertyValue::Float(v)) => if let ControllerElementKind::Slider { min, .. } = &mut element.kind { *min = *v as f32; },
        ("slider.max", PropertyValue::Float(v)) => if let ControllerElementKind::Slider { max, .. } = &mut element.kind { *max = *v as f32; },
        ("label.text", PropertyValue::Text(v)) => if let ControllerElementKind::Label { text } = &mut element.kind { *text = v.clone(); },
        ("image.source", PropertyValue::FilePath(v)) => if let ControllerElementKind::Image { source } = &mut element.kind { *source = v.clone(); },
        ("line.to.x", PropertyValue::Float(v)) => if let ControllerElementKind::Line { to, .. } = &mut element.kind { to[0] = *v as f32; },
        ("line.to.y", PropertyValue::Float(v)) => if let ControllerElementKind::Line { to, .. } = &mut element.kind { to[1] = *v as f32; },
        ("line.width", PropertyValue::Float(v)) => if let ControllerElementKind::Line { width, .. } = &mut element.kind { *width = (*v as f32).max(0.1); },
        _ => {}
    }
}

pub fn property_model_for_canvas(model: &ControllerModel) -> crate::property_panel::PropertyModel {
    use crate::property_panel::{PropertyGroup, PropertyItem, PropertyModel, PropertyValue};
    PropertyModel {
        object_id: Some("__controller_canvas__".into()),
        display_name: Some("コントロールパネル プロパティ".into()),
        groups: vec![PropertyGroup {
            label: "Controller Panel".into(),
            items: vec![
                PropertyItem::new("canvas.w", "論理幅", PropertyValue::Float(model.canvas_size[0] as f64)),
                PropertyItem::new("canvas.h", "論理高さ", PropertyValue::Float(model.canvas_size[1] as f64)),
                PropertyItem::new("canvas.background", "背景色", PropertyValue::Color(model.background)),
            ],
        }],
    }
}

pub fn apply_canvas_property_action(model: &mut ControllerModel, action: &crate::property_panel::PropertyAction) {
    use crate::property_panel::{PropertyAction, PropertyValue};
    let PropertyAction::ValueChanged { property_id, value } = action;
    match (property_id.as_str(), value) {
        ("canvas.w", PropertyValue::Float(v)) => model.canvas_size[0] = (*v as f32).max(100.0),
        ("canvas.h", PropertyValue::Float(v)) => model.canvas_size[1] = (*v as f32).max(100.0),
        ("canvas.background", PropertyValue::Color(v)) => model.background = *v,
        _ => {}
    }
}
