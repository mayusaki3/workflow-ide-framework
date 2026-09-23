use workflow_ide_framework::{
    Application, LayoutConfig, LayoutSplit, PanelDefinition, PanelKind, SplitDirection,
    property_panel::{PropertyGroup, PropertyItem, PropertyModel, PropertyValue},
};

fn main() -> eframe::Result<()> {
    let model = PropertyModel {
        object_id: Some("arm_link".into()),
        display_name: Some("arm_link Properties".into()),
        groups: vec![
            PropertyGroup {
                label: "General".into(),
                items: vec![
                    PropertyItem::new("name", "Name", PropertyValue::Text("アームリンク".into())),
                    PropertyItem::new("enabled", "Enabled", PropertyValue::Bool(true)),
                    PropertyItem::new("index", "Index", PropertyValue::Integer(2)).read_only(true),
                ],
            },
            PropertyGroup {
                label: "Joint".into(),
                items: vec![
                    PropertyItem::new("angle", "Angle", PropertyValue::Float(0.0)),
                    PropertyItem::new("mode", "Mode", PropertyValue::Enum {
                        value: "Position".into(),
                        options: vec!["Position".into(), "Velocity".into(), "Torque".into()],
                    }),
                ],
            },
        ],
    };

    Application::new("step5-9-property-panel", "Step 5.9 Property Panel")
        .font_path("assets/fonts/default/NotoSansCJK-Regular.ttc")
        .panel(PanelDefinition::new("viewport", "Selection Source", PanelKind::StandardUi))
        .panel(PanelDefinition::new("properties", "Properties", PanelKind::StandardUi))
        .layout(LayoutConfig {
            root_panel_ids: vec!["viewport".into()],
            splits: vec![LayoutSplit {
                anchor_panel_id: "viewport".into(),
                direction: SplitDirection::Right,
                fraction: 0.68,
                panel_ids: vec!["properties".into()],
            }],
            selected_panel_id: Some("viewport".into()),
        })
        .property_panel("properties", model)
        .run()
}
