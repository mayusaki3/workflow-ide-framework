use workflow_ide_framework::{
    Application, LayoutConfig, PanelDefinition, PanelKind,
    tree_viewer::{TreeModel, TreeNode},
};

fn main() -> eframe::Result<()> {
    let project = TreeNode::new("project", "Sample Project")
        .node_type("project")
        .child(
            TreeNode::new("src", "src")
                .node_type("folder")
                .child(TreeNode::new("main", "main.rs").node_type("file"))
                .child(TreeNode::new("robot", "robot.rs").node_type("file")),
        )
        .child(
            TreeNode::new("assets", "assets")
                .node_type("folder")
                .child(TreeNode::new("urdf", "robot.urdf").node_type("file"))
                .child(TreeNode::new("mesh", "robot.glb").node_type("file")),
        )
        .child(TreeNode::new("cargo", "Cargo.toml").node_type("file"));

    Application::new("step5-7-tree-viewer", "Step 5.7 Tree Viewer")
        .panel(PanelDefinition::new(
            "project",
            "Project",
            PanelKind::StandardUi,
        ))
        .layout(LayoutConfig::new(["project"]))
        .tree_viewer_panel(
            "project",
            TreeModel {
                roots: vec![project],
                selected_id: None,
            },
        )
        .run()
}
