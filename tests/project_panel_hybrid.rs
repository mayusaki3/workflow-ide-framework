use workflow_ide_framework::{
    project::ProjectContext,
    project_panel_api::{
        AppPanelAction, AppPanelActionResult, AppPanelNode, AppPanelNodeKind,
        ApplicationProjectPanelProvider, ProjectPanelInvalidation,
    },
};

struct RobotProvider;

impl ApplicationProjectPanelProvider for RobotProvider {
    fn project_panel_roots(&self, _project: &ProjectContext) -> Vec<AppPanelNode> {
        vec![AppPanelNode {
            id: "category.robots".into(),
            label: "Robots".into(),
            kind: AppPanelNodeKind::Category {
                type_id: "meridian.robots".into(),
            },
            children: vec![AppPanelNode {
                id: "robot.01".into(),
                label: "robot01".into(),
                kind: AppPanelNodeKind::LogicalResource {
                    resource_id: "persistent-robot-id".into(),
                    type_id: "meridian.robot".into(),
                },
                children: vec![
                    AppPanelNode {
                        id: "robot.01.urdf".into(),
                        label: "robot.urdf".into(),
                        kind: AppPanelNodeKind::RegisteredResource {
                            registry_resource_id: "urdf-registry-id".into(),
                        },
                        children: vec![],
                    },
                    AppPanelNode {
                        id: "robot.01.mjcf".into(),
                        label: "model.xml".into(),
                        kind: AppPanelNodeKind::RegisteredResource {
                            registry_resource_id: "mjcf-registry-id".into(),
                        },
                        children: vec![],
                    },
                ],
            }],
        }]
    }

    fn handle_project_panel_action(
        &mut self,
        _project: &ProjectContext,
        action: AppPanelAction,
    ) -> AppPanelActionResult {
        match action {
            AppPanelAction::Activated { .. } => AppPanelActionResult::Handled,
            _ => AppPanelActionResult::NotHandled,
        }
    }
}

#[test]
fn pp_hyb_001_002_recursive_logical_and_file_nodes() {
    let project = ProjectContext::new("example");
    let roots = RobotProvider.project_panel_roots(&project);
    assert_eq!(roots.len(), 1);
    let robot = &roots[0].children[0];
    assert_eq!(robot.children.len(), 2);
    assert!(matches!(
        robot.kind,
        AppPanelNodeKind::LogicalResource { ref resource_id, .. }
            if resource_id == "persistent-robot-id"
    ));
    assert!(matches!(
        robot.children[0].kind,
        AppPanelNodeKind::RegisteredResource { ref registry_resource_id }
            if registry_resource_id == "urdf-registry-id"
    ));
}

#[test]
fn pp_hyb_004_same_registry_file_multiple_display_nodes() {
    let mut roots = RobotProvider.project_panel_roots(&ProjectContext::new("example"));
    let duplicate = AppPanelNode {
        id: "robot.02.urdf".into(),
        label: "same.urdf".into(),
        kind: roots[0].children[0].children[0].kind.clone(),
        children: vec![],
    };
    let original = &roots[0].children[0].children[0];
    assert_ne!(original.id, duplicate.id);
    assert_eq!(original.kind, duplicate.kind);
    roots[0].children[0].children.push(duplicate);
    assert_eq!(roots[0].children[0].children.len(), 3);
}

#[test]
fn pp_hyb_011_notifications_are_coalesced_and_reset() {
    let mut state = ProjectPanelInvalidation::default();
    assert!(!state.take_pending());
    state.invalidate_application_project_panel();
    state.invalidate_application_project_panel();
    assert!(state.is_pending());
    assert!(state.take_pending());
    assert!(!state.take_pending());
    state.invalidate_application_project_panel();
    state.clear_on_project_change();
    assert!(!state.take_pending());
}

#[test]
fn pp_hyb_013_014_actions_and_not_handled_result() {
    let mut provider = RobotProvider;
    let project = ProjectContext::new("example");
    assert_eq!(
        provider.handle_project_panel_action(
            &project,
            AppPanelAction::Selected {
                node_ids: vec!["robot.01".into(), "robot.02".into()],
            },
        ),
        AppPanelActionResult::NotHandled
    );
    assert_eq!(
        provider.handle_project_panel_action(
            &project,
            AppPanelAction::Activated {
                node_id: "robot.01".into(),
            },
        ),
        AppPanelActionResult::Handled
    );
}

#[test]
fn pp_hyb_015_unknown_registry_resource_is_rejected() {
    use workflow_ide_framework::{
        project_panel_api::{AppPanelTreeError, build_application_project_tree},
        resource_registry::ResourceRegistry,
    };
    let roots = RobotProvider.project_panel_roots(&ProjectContext::new("example"));
    let result = build_application_project_tree(&roots, &ResourceRegistry::default());
    assert_eq!(
        result.unwrap_err(),
        AppPanelTreeError::UnknownRegisteredResource("urdf-registry-id".into())
    );
}

#[test]
fn pp_hyb_002_registry_resources_are_resolved() {
    use workflow_ide_framework::{
        project_panel_api::build_application_project_tree,
        project_resource::{ResourceReference, ResourceScope},
        resource_registry::ResourceRegistry,
    };
    let mut registry = ResourceRegistry::default();
    registry.register(
        "urdf-registry-id",
        ResourceReference::new(ResourceScope::Project, "robot.urdf").unwrap(),
    );
    registry.register(
        "mjcf-registry-id",
        ResourceReference::new(ResourceScope::Project, "model.xml").unwrap(),
    );
    let roots = RobotProvider.project_panel_roots(&ProjectContext::new("example"));
    let tree = build_application_project_tree(&roots, &registry).unwrap();
    assert_eq!(tree.roots[0].children[0].children.len(), 2);
    assert_eq!(
        tree.roots[0].children[0].node_type.as_deref(),
        Some("project.logical")
    );
}

#[test]
fn pp_hyb_005_duplicate_node_id_is_rejected() {
    use workflow_ide_framework::{
        project_panel_api::{AppPanelTreeError, build_application_project_tree},
        resource_registry::ResourceRegistry,
    };
    let nodes = vec![
        AppPanelNode {
            id: "same".into(),
            label: "Robot".into(),
            kind: AppPanelNodeKind::Category {
                type_id: "robot".into(),
            },
            children: vec![],
        },
        AppPanelNode {
            id: "same".into(),
            label: "SysID".into(),
            kind: AppPanelNodeKind::Category {
                type_id: "sysid".into(),
            },
            children: vec![],
        },
    ];
    assert_eq!(
        build_application_project_tree(&nodes, &ResourceRegistry::default()).unwrap_err(),
        AppPanelTreeError::DuplicateNodeId("same".into())
    );
}

#[test]
fn pp_hyb_013_tree_events_map_to_application_actions() {
    use workflow_ide_framework::{
        project_panel_api::app_action_from_tree_action, tree_viewer::TreeAction,
    };
    assert_eq!(
        app_action_from_tree_action(&TreeAction::SelectionChanged {
            node_ids: vec!["robot01".into(), "robot02".into()],
        }),
        Some(AppPanelAction::Selected {
            node_ids: vec!["robot01".into(), "robot02".into()],
        })
    );
    assert_eq!(
        app_action_from_tree_action(&TreeAction::Activated {
            node_id: "robot01".into(),
        }),
        Some(AppPanelAction::Activated {
            node_id: "robot01".into(),
        })
    );
    assert_eq!(
        app_action_from_tree_action(&TreeAction::ContextRequested {
            node_ids: vec!["robot01".into()],
        }),
        None
    );
}

#[test]
fn pp_hyb_014_fallback_only_on_not_handled() {
    use workflow_ide_framework::project_panel_api::dispatch_project_panel_action;
    let mut provider = RobotProvider;
    let project = ProjectContext::new("example");
    let mut called = false;
    let result = dispatch_project_panel_action(
        &mut provider,
        &project,
        AppPanelAction::Selected {
            node_ids: vec!["robot01".into()],
        },
        |_| {
            called = true;
            AppPanelActionResult::Handled
        },
    );
    assert_eq!(result, AppPanelActionResult::Handled);
    assert!(called);

    called = false;
    let result = dispatch_project_panel_action(
        &mut provider,
        &project,
        AppPanelAction::Activated {
            node_id: "robot01".into(),
        },
        |_| {
            called = true;
            AppPanelActionResult::Handled
        },
    );
    assert_eq!(result, AppPanelActionResult::Handled);
    assert!(!called);
}

#[test]
fn pp_hyb_012_project_lifecycle_drops_stale_notifications() {
    use workflow_ide_framework::project_panel_api::ProjectPanelLifecycle;
    let mut lifecycle = ProjectPanelLifecycle::default();
    lifecycle.invalidate_application_project_panel();
    assert!(lifecycle.take_refresh_request().is_none());

    let first = ProjectContext::new("first-project");
    lifecycle.activate_project(&first);
    assert_eq!(lifecycle.take_refresh_request(), Some(first.clone()));
    lifecycle.invalidate_application_project_panel();
    lifecycle.close_project();
    assert!(lifecycle.take_refresh_request().is_none());

    let second = ProjectContext::new("second-project");
    lifecycle.activate_project(&second);
    assert_eq!(lifecycle.take_refresh_request(), Some(second));
    assert!(lifecycle.take_refresh_request().is_none());
}

#[test]
fn pp_hyb_011_refresh_handle_coalesces_cross_clone_requests() {
    use workflow_ide_framework::project_panel_api::ProjectPanelRefreshHandle;
    let handle = ProjectPanelRefreshHandle::default();
    let app_handle = handle.clone();
    app_handle.invalidate_application_project_panel();
    app_handle.invalidate_application_project_panel();
    assert!(handle.take_pending());
    assert!(!app_handle.take_pending());
    handle.invalidate_application_project_panel();
    assert!(app_handle.take_pending());
}

#[test]
fn pp_dir_001_002_project_directory_and_extension_filter() {
    use workflow_ide_framework::{
        project_panel_api::{DirectoryFilter, build_application_project_tree_with_directories},
        resource_registry::ResourceRegistry,
    };
    let root = std::env::temp_dir().join(format!("wfide-pp-dir-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("models")).unwrap();
    std::fs::write(root.join("models/robot.urdf"), "robot").unwrap();
    std::fs::write(root.join("models/readme.txt"), "notes").unwrap();
    let node = AppPanelNode {
        id: "models".into(),
        label: "Models".into(),
        kind: AppPanelNodeKind::Directory {
            relative_path: "models".into(),
            filter: DirectoryFilter {
                extensions: vec!["urdf".into()],
                ..DirectoryFilter::default()
            },
        },
        children: vec![],
    };
    let registry = ResourceRegistry::default();
    let tree = build_application_project_tree_with_directories(
        &[node],
        &registry,
        &ProjectContext::new(&root),
    )
    .unwrap();
    assert_eq!(tree.roots[0].children.len(), 1);
    assert_eq!(tree.roots[0].children[0].label, "robot.urdf");
    assert!(registry.entries().is_empty());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn pp_dir_008_rejects_parent_traversal() {
    use workflow_ide_framework::{
        project_panel_api::{
            AppPanelTreeError, DirectoryFilter, build_application_project_tree_with_directories,
        },
        resource_registry::ResourceRegistry,
    };
    let node = AppPanelNode {
        id: "escape".into(),
        label: "Escape".into(),
        kind: AppPanelNodeKind::Directory {
            relative_path: "../outside".into(),
            filter: DirectoryFilter::default(),
        },
        children: vec![],
    };
    let result = build_application_project_tree_with_directories(
        &[node],
        &ResourceRegistry::default(),
        &ProjectContext::new("."),
    );
    assert!(matches!(
        result,
        Err(AppPanelTreeError::InvalidDirectoryPath(_))
    ));
}
