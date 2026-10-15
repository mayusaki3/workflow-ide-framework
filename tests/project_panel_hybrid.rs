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
