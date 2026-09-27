use workflow_ide_framework::{
    Application, LayoutConfig, PanelDefinition, PanelKind,
    controller_panel::{ControllerElement, ControllerElementKind, ControllerModel, ControllerMode, LineShape},
    flow_editor::{FlowEdge, FlowModel, FlowNode, FlowPort, PortDirection},
    logging::LogLevel,
    property_panel::PropertyModel,
    text_editor::TextDocument,
    tree_viewer::{TreeModel, TreeNode},
};

fn main() -> eframe::Result<()> {
    let project = TreeNode::new("project", "サンプルプロジェクト / Sample Project")
        .node_type("project")
        .child(
            TreeNode::new("src", "src")
                .node_type("folder")
                .child(TreeNode::new("main", "main.rs").node_type("file")),
        )
        .child(
            TreeNode::new("resources", "resources")
                .node_type("folder")
                .child(TreeNode::new("robot", "robot.glb").node_type("file")),
        );

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

    let controller = ControllerModel {
        mode: ControllerMode::Operate,
        canvas_size: [820.0, 520.0],
        background: [30, 30, 30, 255],
        selected_id: None,
        elements: vec![
            ControllerElement::new(
                "title", "タイトル", [30.0, 25.0], [260.0, 32.0],
                ControllerElementKind::Label { text: "ロボット操作パネル".into() },
            ),
            ControllerElement::new(
                "separator", "区切り線", [30.0, 70.0], [500.0, 2.0],
                ControllerElementKind::Line { to: [530.0, 70.0], width: 2.0, shape: LineShape::Line },
            ),
            ControllerElement::new(
                "forward", "前進", [40.0, 105.0], [110.0, 40.0],
                ControllerElementKind::Button { text: "前進".into(), image_source: None },
            ),
            ControllerElement::new(
                "speed", "速度", [40.0, 170.0], [240.0, 36.0],
                ControllerElementKind::Slider { value: 0.5, min: 0.0, max: 1.0 },
            ),
            ControllerElement::new(
                "stick", "移動", [330.0, 105.0], [150.0, 150.0],
                ControllerElementKind::Joystick { value: [0.0, 0.0], return_to_center: true },
            ),
        ],
    };

    Application::new("org.workflow-ide-framework.sample", "Workflow IDE Framework Sample Application")
        .font_path("assets/fonts/default/NotoSansCJK-Regular.ttc")
        .log_level(LogLevel::Debug)
        .panel(PanelDefinition::new("project", "プロジェクト / Project", PanelKind::StandardUi))
        .panel(PanelDefinition::new("flow", "フロー / Flow", PanelKind::StandardUi))
        .panel(PanelDefinition::new("controller", "コントローラー / Controller", PanelKind::StandardUi))
        .panel(PanelDefinition::new("properties", "プロパティ / Properties", PanelKind::StandardUi))
        .panel(PanelDefinition::new("text-editor", "テキスト / Text Editor", PanelKind::StandardUi))
        .panel(PanelDefinition::new("logs", "ログ / Logs", PanelKind::StandardUi))
        .layout(
            LayoutConfig::new(["flow", "controller", "text-editor"])
                .split_left("flow", 0.20, ["project"])
                .split_right("flow", 0.25, ["properties"])
                .split_below("flow", 0.72, ["logs"])
                .selected("flow"),
        )
        .tree_viewer_panel("project", TreeModel { roots: vec![project], selected_id: None })
        .flow_editor_panel("flow", flow)
        .controller_panel("controller", controller)
        .property_panel("properties", PropertyModel::default())
        .link_flow_properties("flow", "properties")
        .link_controller_properties("controller", "properties")
        .text_editor_panel(
            "text-editor",
            TextDocument::new(
                "sample-document",
                "sample.toml",
                "# Workflow IDE Framework Sample\n\n[application]\nname = \"サンプルアプリ\"\nenabled = true\n",
            ).language_hint("TOML"),
        )
        .log_viewer_panel("logs")
        .language_settings_panel()
        .theme_settings_panel()
        .run()
}
