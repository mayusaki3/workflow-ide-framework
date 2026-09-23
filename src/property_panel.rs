use eframe::egui;

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    Text(String),
    Bool(bool),
    Integer(i64),
    Float(f64),
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
            PropertyValue::Enum { value, .. } => { ui.label(value.as_str()); }
        }
        return false;
    }

    match value {
        PropertyValue::Text(v) => ui.text_edit_singleline(v).changed(),
        PropertyValue::Bool(v) => ui.checkbox(v, "").changed(),
        PropertyValue::Integer(v) => ui.add(egui::DragValue::new(v)).changed(),
        PropertyValue::Float(v) => ui.add(egui::DragValue::new(v)).changed(),
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
