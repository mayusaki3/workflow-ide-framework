use workflow_ide_framework::{
    Application, LayoutConfig, LayoutSplit, PanelDefinition, PanelKind, SplitDirection,
    flow_editor::{FlowEdge, FlowModel, FlowNode, FlowPort, PortDirection},
    logging::LogLevel,
    property_panel::PropertyModel,
};

fn main() -> eframe::Result<()> {
    let base = FlowNode::new("base", "ベース / base_link", [70.0, 100.0])
        .port(FlowPort::new("child", "child", PortDirection::Output));
    let joint = FlowNode::new("joint", "肩関節 / shoulder_joint", [330.0, 100.0])
        .port(FlowPort::new("parent", "parent", PortDirection::Input))
        .port(FlowPort::new("child", "child", PortDirection::Output));
    let arm = FlowNode::new("arm", "アーム / arm_link", [590.0, 100.0])
        .port(FlowPort::new("parent", "parent", PortDirection::Input));

    let flow = FlowModel {
        nodes: vec![base, joint, arm],
        edges: vec![
            FlowEdge::new("edge-base-joint", "base", "child", "joint", "parent"),
            FlowEdge::new("edge-joint-arm", "joint", "child", "arm", "parent"),
        ],
        selected_node_id: None,
        selected_edge_id: None,
        pending_connection: None,
        pan: eframe::egui::Vec2::ZERO,
        zoom: 1.0,
    };

    Application::new("step5-9-flow-properties", "Step 5.9 Flow + Properties / 日本語検証")
        .font_path("assets/fonts/default/NotoSansCJK-Regular.ttc")
        .log_level(LogLevel::Debug)
        .panel(PanelDefinition::new("flow", "ロボット接続 / Robot Connections", PanelKind::StandardUi))
        .panel(PanelDefinition::new("properties", "プロパティ / Properties", PanelKind::StandardUi))
        .layout(LayoutConfig {
            root_panel_ids: vec!["flow".into()],
            splits: vec![LayoutSplit {
                anchor_panel_id: "flow".into(),
                direction: SplitDirection::Right,
                fraction: 0.72,
                panel_ids: vec!["properties".into()],
            }],
            selected_panel_id: Some("flow".into()),
        })
        .flow_editor_panel("flow", flow)
        .property_panel("properties", PropertyModel::default())
        .link_flow_properties("flow", "properties")
        .run()
}
