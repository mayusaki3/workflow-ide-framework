use workflow_ide_framework::{
    flow_editor::{FlowEdge, FlowModel, FlowNode, FlowPort, PortDirection},
    property_panel::{self, PropertyAction, PropertyValue},
    text_editor::{TextDocument, TextEncoding, TextEditorOptions},
    tree_viewer::{TreeModel, TreeNode},
};

#[test]
fn text_document_preserves_japanese_and_metadata() {
    let doc = TextDocument::new("doc", "日本語.txt", "こんにちは\n世界")
        .language_hint("日本語")
        .encoding(TextEncoding::ShiftJis);
    assert_eq!(doc.display_name, "日本語.txt");
    assert_eq!(doc.text, "こんにちは\n世界");
    assert_eq!(doc.language_hint.as_deref(), Some("日本語"));
    assert_eq!(doc.encoding.label(), "Shift_JIS");
    assert!(!doc.modified);
    assert!(!TextEditorOptions::default().word_wrap);
}

#[test]
fn tree_model_preserves_japanese_labels_and_hierarchy() {
    let root = TreeNode::new("root", "サンプルプロジェクト")
        .node_type("project")
        .child(TreeNode::new("robot", "ロボット.rs").node_type("file"));
    let model = TreeModel { roots: vec![root], selected_id: None };
    assert_eq!(model.roots[0].label, "サンプルプロジェクト");
    assert_eq!(model.roots[0].children[0].label, "ロボット.rs");
    assert_eq!(model.roots[0].children[0].node_type.as_deref(), Some("file"));
}

#[test]
fn flow_model_preserves_japanese_nodes_ports_edges_and_view_state() {
    let base = FlowNode::new("base", "ベース", [10.0, 20.0])
        .port(FlowPort::new("out", "子", PortDirection::Output).data_type("link"));
    let arm = FlowNode::new("arm", "アーム", [30.0, 40.0])
        .port(FlowPort::new("in", "親", PortDirection::Input).data_type("link"));
    let model = FlowModel {
        nodes: vec![base, arm],
        edges: vec![FlowEdge::new("e1", "base", "out", "arm", "in")],
        ..Default::default()
    };
    assert_eq!(model.nodes[0].label, "ベース");
    assert_eq!(model.nodes[1].ports[0].label, "親");
    assert_eq!(model.edges[0].to_node, "arm");
    assert_eq!(model.zoom, 1.0);
}

#[test]
fn property_adapter_round_trips_editable_flow_node_values() {
    let mut node = FlowNode::new("arm", "アーム", [12.0, 34.0]);
    let property = property_panel::from_flow_node(&node);
    assert_eq!(property.object_id.as_deref(), Some("arm"));
    assert_eq!(property.display_name.as_deref(), Some("アーム Properties"));

    property_panel::apply_to_flow_node(
        &mut node,
        &PropertyAction::ValueChanged {
            property_id: "label".into(),
            value: PropertyValue::Text("右アーム".into()),
        },
    );
    property_panel::apply_to_flow_node(
        &mut node,
        &PropertyAction::ValueChanged {
            property_id: "position.x".into(),
            value: PropertyValue::Float(56.0),
        },
    );
    assert_eq!(node.label, "右アーム");
    assert_eq!(node.position.x, 56.0);
    assert_eq!(node.position.y, 34.0);
}

#[test]
fn edge_property_adapter_exposes_read_only_endpoints() {
    let edge = FlowEdge::new("接続1", "ベース", "子", "アーム", "親");
    let property = property_panel::from_flow_edge(&edge);
    let items = &property.groups[0].items;
    assert_eq!(property.object_id.as_deref(), Some("接続1"));
    assert!(items.iter().all(|item| item.read_only));
    assert_eq!(items.len(), 2);
}
