use workflow_ide_framework::{
    Application, LayoutConfig, LayoutSplit, PanelDefinition, PanelKind, SplitDirection,
    flow_editor::{FlowEdge, FlowModel, FlowNode, FlowPort, PortDirection},
    property_panel::PropertyModel,
};

fn main() -> eframe::Result<()> {
    // Temporary Step 5.8 input probe: show DEBUG input-routing events in the
    // console without changing the framework-wide default log level.
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(true)
        .try_init();

    let base = FlowNode::new("base", "base_link", [70.0, 100.0])
        .port(FlowPort::new("child", "child", PortDirection::Output));
    let joint = FlowNode::new("joint", "shoulder_joint", [330.0, 100.0])
        .port(FlowPort::new("parent", "parent", PortDirection::Input))
        .port(FlowPort::new("child", "child", PortDirection::Output));
    let arm = FlowNode::new("arm", "arm_link", [590.0, 100.0])
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

    Application::new("step5-9-flow-properties", "Step 5.9 Flow + Properties")
        .panel(PanelDefinition::new("flow", "Robot Connections", PanelKind::StandardUi))
        .panel(PanelDefinition::new("properties", "Properties", PanelKind::StandardUi))
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
