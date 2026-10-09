use workflow_ide_framework::{
    project::ProjectContext,
    project_adapter::ApplicationProjectAdapter,
    project_controller::{ProjectCommandResult, ProjectController},
    project_resource::NewProjectStoragePolicy,
    project_resource::{ProjectDataCompatibility, ProjectDataConsistency},
    project_save::ApplicationSaveResult,
};

#[derive(Default)]
struct App;
impl ApplicationProjectAdapter for App {
    type Error = &'static str;
    fn initialize_project(&mut self, _context: &ProjectContext) -> Result<(), Self::Error> {
        Ok(())
    }
    fn inspect_project_data(
        &mut self,
        _context: &ProjectContext,
        _version: Option<&str>,
    ) -> Result<ProjectDataCompatibility, Self::Error> {
        Ok(ProjectDataCompatibility::Compatible)
    }
    fn check_project_consistency(
        &mut self,
        _context: &ProjectContext,
    ) -> Result<ProjectDataConsistency, Self::Error> {
        Ok(ProjectDataConsistency::Consistent)
    }
    fn save_project_data(
        &mut self,
        _context: &ProjectContext,
        _save_id: &str,
    ) -> Result<ApplicationSaveResult, Self::Error> {
        Ok(ApplicationSaveResult {
            data_version: Some("1".into()),
        })
    }
    fn save_project_data_as(
        &mut self,
        _source: Option<&ProjectContext>,
        _destination: &ProjectContext,
        _save_id: &str,
    ) -> Result<ApplicationSaveResult, Self::Error> {
        Ok(ApplicationSaveResult {
            data_version: Some("1".into()),
        })
    }
}

#[test]
fn deferred_new_routes_save_to_save_as_and_close_requires_confirmation() {
    let mut controller = ProjectController::new("org.test", "Test");
    let mut app = App;
    assert_eq!(
        controller.new_project(
            "Untitled",
            "en",
            None,
            NewProjectStoragePolicy::Deferred,
            None,
            &mut app
        ),
        ProjectCommandResult::Completed
    );
    assert!(controller.is_open());
    assert_eq!(
        controller.save(&mut app, "save-1", "2026-09-29T00:00:00Z"),
        ProjectCommandResult::NeedsSaveLocation
    );
    assert_eq!(
        controller.close(false),
        ProjectCommandResult::NeedsDirtyConfirmation
    );
    assert!(controller.is_open());
    assert_eq!(controller.close(true), ProjectCommandResult::Completed);
    assert!(!controller.is_open());
}

#[test]
fn required_new_without_root_requests_location() {
    let mut controller = ProjectController::new("org.test", "Test");
    let mut app = App;
    assert_eq!(
        controller.new_project(
            "Untitled",
            "en",
            None,
            NewProjectStoragePolicy::Required,
            None,
            &mut app
        ),
        ProjectCommandResult::NeedsSaveLocation
    );
    assert!(!controller.is_open());
}

#[test]
fn new_project_keeps_name_description_and_language_metadata() {
    let mut controller = ProjectController::new("org.test", "Test");
    let mut app = App;
    assert_eq!(
        controller.new_project(
            "Demo",
            "ja-JP",
            Some("説明".into()),
            NewProjectStoragePolicy::Deferred,
            None,
            &mut app
        ),
        ProjectCommandResult::Completed
    );
    let file = controller.project_file.as_ref().unwrap();
    assert_eq!(file.project.name, "Demo");
    assert_eq!(file.project.description.as_deref(), Some("説明"));
    assert_eq!(file.project.language, "ja-JP");
}

#[test]
fn project_properties_update_framework_metadata_and_mark_dirty() {
    let mut controller = ProjectController::new("app.test", "Test App");
    let mut adapter = App;
    assert_eq!(
        controller.new_project(
            "Before",
            "en-US",
            None,
            NewProjectStoragePolicy::Deferred,
            None,
            &mut adapter
        ),
        ProjectCommandResult::Completed
    );
    controller.session.as_mut().unwrap().dirty = Default::default();

    assert_eq!(
        controller.update_project_metadata("After", Some(" Project description ".into())),
        ProjectCommandResult::Completed
    );
    assert_eq!(controller.project_name(), Some("After"));
    assert_eq!(
        controller.project_description(),
        Some("Project description")
    );
    assert!(controller.session.as_ref().unwrap().dirty.metadata);
}

#[test]
fn project_workspace_layout_capture_and_restore_round_trip() {
    use std::collections::BTreeMap;
    use workflow_ide_framework::{
        workspace::{PanelInstanceId, WorkspaceRegistry, FloatingGeometry, DEFAULT_WORKSPACE_ID, DEFAULT_CONTAINER_ID},
        workspace_dock::{project_workspace, snapshot_workspace_layout},
    };
    let mut source = WorkspaceRegistry::new();
    source.register("custom", "Custom").unwrap();
    for id in ["a", "b", "c"] {
        let panel = PanelInstanceId::new("editor", id);
        source.register_panel(panel.clone());
        source.place_panel(DEFAULT_WORKSPACE_ID, &panel, DEFAULT_CONTAINER_ID).unwrap();
    }
    source.float_panel(DEFAULT_WORKSPACE_ID, &PanelInstanceId::new("editor", "b"),
        FloatingGeometry { position: [10.0, 20.0], size: [300.0, 200.0] }).unwrap();
    source.hide_panel(DEFAULT_WORKSPACE_ID, &PanelInstanceId::new("editor", "c")).unwrap();
    let mut projections = BTreeMap::new();
    for ws in source.iter() {
        projections.insert(ws.id.clone(), project_workspace(ws).unwrap());
    }
    let mut controller = ProjectController::new("org.test", "Test");
    controller.capture_workspace_layouts(&source, &projections).unwrap();
    assert_eq!(controller.framework_settings.workspace_layouts.len(), 2);
    let saved = controller.framework_settings.workspace_layouts.clone();
    let mut fresh = WorkspaceRegistry::new();
    fresh.register("custom", "Custom").unwrap();
    for id in ["a", "b", "c"] {
        fresh.register_panel(PanelInstanceId::new("editor", id));
    }
    let restored = controller.restore_project_workspace_layouts(&mut fresh).unwrap();
    assert_eq!(restored.len(), 2);
    for (id, projection) in &restored {
        assert_eq!(snapshot_workspace_layout(fresh.get(id).unwrap(), projection).unwrap(), saved[id]);
    }
}

#[test]
fn project_workspace_layout_restore_is_atomic_across_workspaces() {
    use std::collections::BTreeMap;
    use workflow_ide_framework::{
        workspace::{PanelInstanceId, WorkspaceRegistry, DEFAULT_WORKSPACE_ID, DEFAULT_CONTAINER_ID},
        workspace_dock::{project_workspace, snapshot_workspace_layout},
    };
    let mut source = WorkspaceRegistry::new();
    source.register("custom", "Custom").unwrap();
    let panel = PanelInstanceId::new("editor", "a");
    source.register_panel(panel.clone());
    source.place_panel(DEFAULT_WORKSPACE_ID, &panel, DEFAULT_CONTAINER_ID).unwrap();
    let mut projections = BTreeMap::new();
    for ws in source.iter() {
        projections.insert(ws.id.clone(), project_workspace(ws).unwrap());
    }
    let mut controller = ProjectController::new("org.test", "Test");
    controller.capture_workspace_layouts(&source, &projections).unwrap();
    controller.framework_settings.workspace_layouts.get_mut("custom").unwrap()
        .containers[0].id = "invalid.container".into();
    let mut fresh = WorkspaceRegistry::new();
    fresh.register("custom", "Custom").unwrap();
    fresh.register_panel(panel);
    let before: BTreeMap<_, _> = fresh.iter().map(|ws| {
        (ws.id.clone(), snapshot_workspace_layout(ws, &project_workspace(ws).unwrap()).unwrap())
    }).collect();
    assert!(controller.restore_project_workspace_layouts(&mut fresh).is_err());
    for ws in fresh.iter() {
        assert_eq!(snapshot_workspace_layout(ws, &project_workspace(ws).unwrap()).unwrap(), before[&ws.id]);
    }
}

#[test]
fn legacy_project_without_workspace_layouts_keeps_defaults() {
    use workflow_ide_framework::workspace::{WorkspaceRegistry, DEFAULT_WORKSPACE_ID};
    let controller = ProjectController::new("org.test", "Test");
    let mut registry = WorkspaceRegistry::new();
    let restored = controller.restore_project_workspace_layouts(&mut registry).unwrap();
    assert!(restored.is_empty());
    assert!(registry.get(DEFAULT_WORKSPACE_ID).is_some());
}

#[test]
fn independent_workspace_splits_and_active_tabs_survive_switch_and_project_restore() {
    use std::collections::BTreeMap;
    use egui_dock::{Node, NodeIndex, TabIndex};
    use workflow_ide_framework::{
        workspace::{PanelInstanceId, WorkspaceRegistry, DEFAULT_CONTAINER_ID, DEFAULT_WORKSPACE_ID},
        workspace_dock::{
            project_workspace, snapshot_workspace_layout, switch_workspace_projection,
            DockPanelKey,
        },
    };

    let mut registry = WorkspaceRegistry::new();
    registry.register("second", "Second").unwrap();
    for id in ["a", "b", "c"] {
        let panel = PanelInstanceId::new("editor", id);
        registry.register_panel(panel.clone());
        for workspace_id in [DEFAULT_WORKSPACE_ID, "second"] {
            registry.place_panel(workspace_id, &panel, DEFAULT_CONTAINER_ID).unwrap();
        }
    }

    let mut active = DEFAULT_WORKSPACE_ID.to_owned();
    let mut live = project_workspace(registry.get(&active).unwrap()).unwrap();
    let mut cached = BTreeMap::new();
    let key = |id| DockPanelKey::from(&PanelInstanceId::new("editor", id));

    // Workspace A: a | (b,c), with c selected.
    {
        let dock = live.normal.get_mut(DEFAULT_CONTAINER_ID).unwrap().as_mut().unwrap();
        for node in dock.main_surface_mut().iter_mut() {
            if let Node::Leaf(leaf) = node {
                leaf.tabs.retain(|tab| tab.instance_id == "a");
            }
        }
        dock.main_surface_mut().split_right(NodeIndex::root(), 0.4, vec![key("b"), key("c")]);
        dock.main_surface_mut().set_active_tab(NodeIndex(2), TabIndex(1)).unwrap();
    }
    let expected_a = snapshot_workspace_layout(registry.get(&active).unwrap(), &live).unwrap();

    registry.select("second").unwrap();
    switch_workspace_projection(&registry, &mut active, &mut live, &mut cached).unwrap();

    // Workspace B: (a,b) / c, with b selected.
    {
        let dock = live.normal.get_mut(DEFAULT_CONTAINER_ID).unwrap().as_mut().unwrap();
        for node in dock.main_surface_mut().iter_mut() {
            if let Node::Leaf(leaf) = node {
                leaf.tabs.retain(|tab| tab.instance_id != "c");
            }
        }
        dock.main_surface_mut().split_below(NodeIndex::root(), 0.65, vec![key("c")]);
        dock.main_surface_mut().set_active_tab(NodeIndex(1), TabIndex(1)).unwrap();
    }
    let expected_b = snapshot_workspace_layout(registry.get(&active).unwrap(), &live).unwrap();
    assert_ne!(expected_a, expected_b);

    registry.select(DEFAULT_WORKSPACE_ID).unwrap();
    switch_workspace_projection(&registry, &mut active, &mut live, &mut cached).unwrap();
    assert_eq!(snapshot_workspace_layout(registry.get(&active).unwrap(), &live).unwrap(), expected_a);
    registry.select("second").unwrap();
    switch_workspace_projection(&registry, &mut active, &mut live, &mut cached).unwrap();
    assert_eq!(snapshot_workspace_layout(registry.get(&active).unwrap(), &live).unwrap(), expected_b);

    // Capture the active projection and the independent cached projection.
    let mut projections = cached;
    projections.insert(active.clone(), live);
    let mut controller = ProjectController::new("org.test", "Test");
    controller.capture_workspace_layouts(&registry, &projections).unwrap();

    let mut fresh = WorkspaceRegistry::new();
    fresh.register("second", "Second").unwrap();
    for id in ["a", "b", "c"] {
        fresh.register_panel(PanelInstanceId::new("editor", id));
    }
    let restored = controller.restore_project_workspace_layouts(&mut fresh).unwrap();
    assert_eq!(
        snapshot_workspace_layout(fresh.get(DEFAULT_WORKSPACE_ID).unwrap(), &restored[DEFAULT_WORKSPACE_ID]).unwrap(),
        expected_a
    );
    assert_eq!(
        snapshot_workspace_layout(fresh.get("second").unwrap(), &restored["second"]).unwrap(),
        expected_b
    );
}
