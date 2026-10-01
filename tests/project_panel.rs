use workflow_ide_framework::{
    project_panel::{ProjectPanelModel, ProjectPanelNode, ProvidedResource, ResourceProviderKind},
    project_resource::{ResourceReference, ResourceScope},
    resource_registry::ResourceRegistry,
};

#[test]
fn logical_tree_combines_framework_application_and_project_resources() {
    let mut registry = ResourceRegistry::default();
    registry.register("project-image", ResourceReference::new(ResourceScope::Project, "images/avatar.png").unwrap());

    let model = ProjectPanelModel {
        roots: vec![
            ProjectPanelNode::logical("root", "Project")
                .child(ProjectPanelNode::resources(
                    "resources",
                    "Resources",
                    [ResourceProviderKind::Framework, ResourceProviderKind::Application, ResourceProviderKind::Project],
                )),
        ],
        framework_resources: vec![ProvidedResource::new(
            "fw-theme", "Sakura", "themes/sakura.toml", ResourceProviderKind::Framework,
        )],
        application_resources: vec![ProvidedResource::new(
            "app-icon", "Robot Icon", "icons/robot.png", ResourceProviderKind::Application,
        )],
    };

    let tree = model.tree(&registry);
    let resources = &tree.roots[0].children[0].children;
    assert_eq!(resources.len(), 3);
    assert!(resources.iter().any(|node| node.label == "Sakura"));
    assert!(resources.iter().any(|node| node.label == "Robot Icon"));
    assert!(resources.iter().any(|node| node.label == "avatar.png"));
}

#[test]
fn resource_node_can_filter_provider_kinds() {
    let mut registry = ResourceRegistry::default();
    registry.register("external", ResourceReference::new(ResourceScope::External, std::env::temp_dir().join("external.png")).unwrap());
    registry.register("project", ResourceReference::new(ResourceScope::Project, "project.png").unwrap());

    let model = ProjectPanelModel {
        roots: vec![ProjectPanelNode::resources("project-only", "Project Resources", [ResourceProviderKind::Project])],
        ..Default::default()
    };

    let tree = model.tree(&registry);
    assert_eq!(tree.roots[0].children.len(), 1);
    assert_eq!(tree.roots[0].children[0].label, "project.png");
}
