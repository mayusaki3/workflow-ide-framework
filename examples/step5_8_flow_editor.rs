use workflow_ide_framework::{
    Application, LayoutConfig, PanelDefinition, PanelKind,
    flow_editor::{FlowEdge, FlowModel, FlowNode, FlowPort, PortDirection},
};

fn main() -> eframe::Result<()> {
    let base = FlowNode::new("base", "base_link", [80.0, 120.0])
        .port(FlowPort::new("child", "child", PortDirection::Output).data_type("link"));
    let joint = FlowNode::new("joint", "shoulder_joint", [330.0, 120.0])
        .port(FlowPort::new("parent", "parent", PortDirection::Input).data_type("link"))
        .port(FlowPort::new("child", "child", PortDirection::Output).data_type("link"));
    let arm = FlowNode::new("arm", "arm_link", [580.0, 120.0])
        .port(FlowPort::new("parent", "parent", PortDirection::Input).data_type("link"))
        .port(FlowPort::new("child", "child", PortDirection::Output).data_type("link"));
    let tool = FlowNode::new("tool", "tool_link", [580.0, 300.0])
        .port(FlowPort::new("parent", "parent", PortDirection::Input).data_type("link"));

    let model = FlowModel {
        nodes: vec![base, joint, arm, tool],
        edges: vec![
            FlowEdge::new("e1", "base", "child", "joint", "parent"),
            FlowEdge::new("e2", "joint", "child", "arm", "parent"),
        ],
        selected_node_id: None,
        selected_edge_id: None,
        pending_connection: None,
        pan: eframe::egui::Vec2::ZERO,
        zoom: 1.0,
    };

    Application::new("step5-8-flow-editor", "Step 5.8 Flow Editor")
        .panel(PanelDefinition::new("flow", "Robot Connections", PanelKind::StandardUi))
        .layout(LayoutConfig::new(["flow"]))
        .flow_editor_panel("flow", model)
        .run()
}
