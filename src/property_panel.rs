use eframe::egui;

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    Text(String),
    Bool(bool),
    Integer(i64),
    Float(f64),
    Color([u8; 4]),
    Enum { value: String, options: Vec<String> },
}

#[derive(Debug, Clone)]
pub struct PropertyItem {
    pub id: String,
    pub label: String,
    pub value: PropertyValue,
    pub read_only: bool,
}

impl PropertyItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>, value: PropertyValue) -> Self {
        Self { id: id.into(), label: label.into(), value, read_only: false }
    }
    pub fn read_only(mut self, value: bool) -> Self { self.read_only = value; self }
}

#[derive(Debug, Clone, Default)]
pub struct PropertyGroup {
    pub label: String,
    pub items: Vec<PropertyItem>,
}

#[derive(Debug, Clone, Default)]
pub struct PropertyModel {
    pub object_id: Option<String>,
    pub display_name: Option<String>,
    pub groups: Vec<PropertyGroup>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyAction {
    ValueChanged { property_id: String, value: PropertyValue },
}

#[derive(Debug, Default)]
pub struct PropertyResponse { pub actions: Vec<PropertyAction> }

pub fn show(ui: &mut egui::Ui, model: &mut PropertyModel) -> PropertyResponse {
    let mut response = PropertyResponse::default();

    if let Some(name) = &model.display_name {
        ui.heading(name);
    }
    if let Some(id) = &model.object_id {
        ui.label(egui::RichText::new(id).weak().monospace());
    }
    if model.display_name.is_some() || model.object_id.is_some() { ui.separator(); }

    for group in &mut model.groups {
        let mut body = |ui: &mut egui::Ui| {
            egui::Grid::new(("wfide_property_grid", &group.label))
                .num_columns(2)
                .spacing([12.0, 6.0])
                .show(ui, |ui| {
                    for item in &mut group.items {
                        ui.label(&item.label);
                        let changed = show_value(ui, &item.id, &mut item.value, item.read_only);
                        if changed {
                            response.actions.push(PropertyAction::ValueChanged {
                                property_id: item.id.clone(),
                                value: item.value.clone(),
                            });
                        }
                        ui.end_row();
                    }
                });
        };
        if group.label.is_empty() {
            body(ui);
        } else {
            egui::CollapsingHeader::new(&group.label).default_open(true).show(ui, body);
        }
    }
    response
}

fn show_value(ui: &mut egui::Ui, id: &str, value: &mut PropertyValue, read_only: bool) -> bool {
    if read_only {
        match value {
            PropertyValue::Text(v) => { ui.label(v.as_str()); }
            PropertyValue::Bool(v) => { ui.label(if *v { "true" } else { "false" }); }
            PropertyValue::Integer(v) => { ui.label(v.to_string()); }
            PropertyValue::Float(v) => { ui.label(v.to_string()); }
            PropertyValue::Color(v) => { ui.label(format!("#{:02X}{:02X}{:02X}{:02X}", v[0], v[1], v[2], v[3])); }
            PropertyValue::Enum { value, .. } => { ui.label(value.as_str()); }
        }
        return false;
    }

    match value {
        PropertyValue::Text(v) => ui.text_edit_singleline(v).changed(),
        PropertyValue::Bool(v) => ui.checkbox(v, "").changed(),
        PropertyValue::Integer(v) => ui.add(egui::DragValue::new(v)).changed(),
        PropertyValue::Float(v) => ui.add(egui::DragValue::new(v)).changed(),
        PropertyValue::Color(v) => {
            let mut color = egui::Color32::from_rgba_unmultiplied(v[0], v[1], v[2], v[3]);
            let changed = ui.color_edit_button_srgba(&mut color).changed();
            if changed { *v = color.to_array(); }
            changed
        }
        PropertyValue::Enum { value, options } => {
            let before = value.clone();
            egui::ComboBox::from_id_salt(("wfide_property_enum", id))
                .selected_text(value.as_str())
                .show_ui(ui, |ui| {
                    for option in options.iter() {
                        ui.selectable_value(value, option.clone(), option);
                    }
                });
            *value != before
        }
    }
}


/// Build a generic property view for a Flow Editor node.
/// Domain-specific properties remain the Consumer's responsibility.
pub fn from_flow_node(node: &crate::flow_editor::FlowNode) -> PropertyModel {
    PropertyModel {
        object_id: Some(node.id.clone()),
        display_name: Some(format!("{} Properties", node.label)),
        groups: vec![PropertyGroup {
            label: "Node".into(),
            items: vec![
                PropertyItem::new("label", "Label", PropertyValue::Text(node.label.clone())),
                PropertyItem::new("position.x", "X", PropertyValue::Float(node.position.x as f64)),
                PropertyItem::new("position.y", "Y", PropertyValue::Float(node.position.y as f64)),
            ],
        }],
    }
}

/// Build a generic property view for a Flow Editor edge.
pub fn from_flow_edge(edge: &crate::flow_editor::FlowEdge) -> PropertyModel {
    PropertyModel {
        object_id: Some(edge.id.clone()),
        display_name: Some(format!("{} Properties", edge.id)),
        groups: vec![PropertyGroup {
            label: "Edge".into(),
            items: vec![
                PropertyItem::new("from", "From", PropertyValue::Text(format!("{}.{}", edge.from_node, edge.from_port))).read_only(true),
                PropertyItem::new("to", "To", PropertyValue::Text(format!("{}.{}", edge.to_node, edge.to_port))).read_only(true),
            ],
        }],
    }
}

/// Apply the generic editable node properties back to a Flow Editor node.
pub fn apply_to_flow_node(node: &mut crate::flow_editor::FlowNode, action: &PropertyAction) {
    let PropertyAction::ValueChanged { property_id, value } = action;
    match (property_id.as_str(), value) {
        ("label", PropertyValue::Text(value)) => node.label = value.clone(),
        ("position.x", PropertyValue::Float(value)) => node.position.x = *value as f32,
        ("position.y", PropertyValue::Float(value)) => node.position.y = *value as f32,
        _ => {}
    }
}
