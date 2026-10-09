use std::{collections::BTreeMap, fs, path::PathBuf};
use workflow_ide_framework::{
    framework_settings::{
        FrameworkSettings, StoredDockNode, StoredFloatingGeometry, StoredHiddenPanel,
        StoredPanelIdentity, StoredResourceEntry, StoredResourceScope, StoredSplitAxis,
        StoredWorkspaceContainer, StoredWorkspaceLayout,
    },
    project::{
        ApplicationMetadata, PROJECT_FORMAT_VERSION, ProjectContext, ProjectFile, ProjectMetadata,
    },
    project_lifecycle::ProjectSession,
    project_save::{ApplicationProjectSaver, ApplicationSaveResult, save_project},
};

fn temp_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("wfide-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

fn project_file() -> ProjectFile {
    ProjectFile {
        project: ProjectMetadata {
            format_version: PROJECT_FORMAT_VERSION,
            name: "保存テスト".into(),
            description: None,
            language: "ja-JP".into(),
            save_id: "old".into(),
            saved_at: "old".into(),
        },
        application: ApplicationMetadata {
            id: "org.example.test".into(),
            name: "Test".into(),
            description: None,
            language: "en-US".into(),
            data_version: None,
        },
    }
}

struct Saver {
    fail: bool,
    calls: usize,
}
impl ApplicationProjectSaver for Saver {
    type Error = &'static str;
    fn save_project_data(
        &mut self,
        context: &ProjectContext,
        save_id: &str,
    ) -> Result<ApplicationSaveResult, Self::Error> {
        self.calls += 1;
        if self.fail {
            return Err("application save failed");
        }
        fs::create_dir_all(context.application_directory()).map_err(|_| "mkdir")?;
        fs::write(context.application_directory().join("data.txt"), save_id)
            .map_err(|_| "write")?;
        Ok(ApplicationSaveResult {
            data_version: Some("7".into()),
        })
    }
}

#[test]
fn framework_settings_round_trip_resource_registry() {
    let settings = FrameworkSettings {
        format_version: 1,
        save_id: Some("save-registry".into()),
        resources: vec![StoredResourceEntry {
            resource_id: "r1".into(),
            scope: StoredResourceScope::Project,
            path: PathBuf::from("textures/robot.png"),
        }],
        ..FrameworkSettings::default()
    };
    let text = settings.to_toml().unwrap();
    let parsed = FrameworkSettings::from_toml(&text).unwrap();
    assert_eq!(parsed, settings);
    let resource = parsed.resources[0].to_resource().unwrap();
    assert_eq!(resource.reference.path, PathBuf::from("textures/robot.png"));
}

#[test]
fn save_writes_project_file_last_and_clears_dirty_after_success() {
    let root = temp_root("save-ok");
    let context = ProjectContext::new(&root);
    let mut file = project_file();
    let settings = FrameworkSettings::default();
    let mut session = ProjectSession::default();
    session.mark_metadata_dirty();
    session.mark_framework_dirty();
    session.mark_application_dirty();
    let mut saver = Saver {
        fail: false,
        calls: 0,
    };

    save_project(
        &mut session,
        &context,
        &mut file,
        &settings,
        &mut saver,
        "save-42",
        "2026-09-27T13:18:00+09:00",
    )
    .unwrap();

    assert_eq!(saver.calls, 1);
    assert!(!session.dirty.is_dirty());
    assert_eq!(session.current_save_id.as_deref(), Some("save-42"));
    assert!(context.framework_settings_path().is_file());
    assert!(context.project_file_path().is_file());
    assert_eq!(
        fs::read_to_string(context.application_directory().join("data.txt")).unwrap(),
        "save-42"
    );

    let stored = ProjectFile::from_toml(
        &fs::read_to_string(context.project_file_path()).unwrap(),
        "org.example.test",
    )
    .unwrap();
    assert_eq!(stored.project.save_id, "save-42");
    let stored_framework = FrameworkSettings::from_toml(
        &fs::read_to_string(context.framework_settings_path()).unwrap(),
    )
    .unwrap();
    assert_eq!(stored_framework.save_id.as_deref(), Some("save-42"));
    assert_eq!(stored.application.data_version.as_deref(), Some("7"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn application_failure_does_not_write_framework_or_project_and_keeps_dirty() {
    let root = temp_root("save-app-fail");
    let context = ProjectContext::new(&root);
    let mut file = project_file();
    let mut session = ProjectSession::default();
    session.mark_application_dirty();
    let mut saver = Saver {
        fail: true,
        calls: 0,
    };

    assert!(
        save_project(
            &mut session,
            &context,
            &mut file,
            &FrameworkSettings::default(),
            &mut saver,
            "failed-save",
            "now"
        )
        .is_err()
    );
    assert!(session.dirty.is_dirty());
    assert!(!context.framework_settings_path().exists());
    assert!(!context.project_file_path().exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn failed_project_file_write_does_not_mutate_in_memory_project_metadata() {
    let root = temp_root("save-project-fail");
    let context = ProjectContext::new(&root);
    fs::create_dir_all(context.project_file_path()).unwrap();
    let mut file = project_file();
    let original = file.clone();
    let mut session = ProjectSession::default();
    session.mark_metadata_dirty();
    let mut saver = Saver {
        fail: false,
        calls: 0,
    };
    assert!(
        save_project(
            &mut session,
            &context,
            &mut file,
            &FrameworkSettings::default(),
            &mut saver,
            "new-save",
            "new-time"
        )
        .is_err()
    );
    assert_eq!(file, original);
    assert!(session.dirty.is_dirty());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn workspace_layout_round_trips_split_tabs_floating_and_hidden_panels() {
    let panel = |id: &str| StoredPanelIdentity {
        definition_id: "editor".into(),
        instance_id: id.into(),
    };
    let layout = StoredWorkspaceLayout {
        containers: vec![
            StoredWorkspaceContainer {
                id: "default".into(),
                floating: false,
                geometry: None,
                tree: Some(StoredDockNode::Split {
                    axis: StoredSplitAxis::Horizontal,
                    fraction: 0.35,
                    first: Box::new(StoredDockNode::Tabs {
                        panels: vec![panel("one"), panel("two")],
                        active: 1,
                    }),
                    second: Box::new(StoredDockNode::Tabs {
                        panels: vec![panel("three")],
                        active: 0,
                    }),
                }),
            },
            StoredWorkspaceContainer {
                id: "floating-1".into(),
                floating: true,
                geometry: Some(StoredFloatingGeometry {
                    position: [120.0, 80.0],
                    size: [640.0, 480.0],
                }),
                tree: Some(StoredDockNode::Tabs {
                    panels: vec![panel("four")],
                    active: 0,
                }),
            },
        ],
        hidden_panels: vec![StoredHiddenPanel {
            panel: panel("five"),
            normal_container: Some("default".into()),
            floating_geometry: None,
        }],
    };
    let mut settings = FrameworkSettings::default();
    settings.workspace_layouts = BTreeMap::from([("workspace-main".into(), layout)]);
    let encoded = settings.to_toml().expect("serialize workspace layout");
    let decoded = FrameworkSettings::from_toml(&encoded).expect("parse workspace layout");
    assert_eq!(decoded, settings);
}

#[test]
fn legacy_framework_settings_without_workspace_layouts_defaults_to_empty() {
    let decoded = FrameworkSettings::from_toml("format_version = 1\nresources = []\n")
        .expect("parse legacy framework settings");
    assert!(decoded.workspace_layouts.is_empty());
}

#[test]
fn workspace_layouts_survive_real_project_files_and_reopen() {
    use workflow_ide_framework::{
        project_controller::ProjectController,
        project_open::{open_project, ApplicationProjectInspector, FrameworkSettingsOpen},
        project_resource::{ProjectDataCompatibility, ProjectDataConsistency},
        workspace::{PanelInstanceId, WorkspaceRegistry, DEFAULT_CONTAINER_ID, DEFAULT_WORKSPACE_ID},
        workspace_dock::{project_workspace, snapshot_workspace_layout},
    };
    struct CompatibleInspector;
    impl ApplicationProjectInspector for CompatibleInspector {
        type Error = &'static str;
        fn inspect_project_data(
            &mut self, _: &ProjectContext, _: Option<&str>,
        ) -> Result<ProjectDataCompatibility, Self::Error> {
            Ok(ProjectDataCompatibility::Compatible)
        }
        fn check_project_consistency(
            &mut self, _: &ProjectContext,
        ) -> Result<ProjectDataConsistency, Self::Error> {
            Ok(ProjectDataConsistency::Consistent)
        }
    }

    let root = temp_root("workspace-disk-roundtrip");
    let context = ProjectContext::new(&root);
    let mut source = WorkspaceRegistry::new();
    source.register("second", "Second").unwrap();
    for (workspace, ids) in [
        (DEFAULT_WORKSPACE_ID, vec!["a", "b"]),
        ("second", vec!["b", "c"]),
    ] {
        for id in ids {
            let panel = PanelInstanceId::new("editor", id);
            source.register_panel(panel.clone());
            source.place_panel(workspace, &panel, DEFAULT_CONTAINER_ID).unwrap();
        }
    }
    let projections: BTreeMap<_, _> = source.iter()
        .map(|ws| (ws.id.clone(), project_workspace(ws).unwrap()))
        .collect();
    let mut controller = ProjectController::new("org.example.test", "Test");
    controller.capture_workspace_layouts(&source, &projections).unwrap();
    let expected = controller.framework_settings.workspace_layouts.clone();

    let mut session = ProjectSession::default();
    session.mark_framework_dirty();
    let mut file = project_file();
    let mut saver = Saver { fail: false, calls: 0 };
    save_project(
        &mut session, &context, &mut file, &controller.framework_settings,
        &mut saver, "workspace-save-1", "2026-10-09T16:00:00+09:00",
    ).unwrap();

    let reopened = open_project(&context, "org.example.test", &mut CompatibleInspector).unwrap();
    let FrameworkSettingsOpen::Loaded(disk_settings) = reopened.framework_settings else {
        panic!("expected saved framework settings");
    };
    assert_eq!(disk_settings.save_id.as_deref(), Some("workspace-save-1"));
    assert_eq!(disk_settings.workspace_layouts, expected);

    let mut restored_controller = ProjectController::new("org.example.test", "Test");
    restored_controller.framework_settings = disk_settings;
    let mut fresh = WorkspaceRegistry::new();
    fresh.register("second", "Second").unwrap();
    for id in ["a", "b", "c"] {
        fresh.register_panel(PanelInstanceId::new("editor", id));
    }
    let restored = restored_controller.restore_project_workspace_layouts(&mut fresh).unwrap();
    for (id, layout) in expected {
        assert_eq!(
            snapshot_workspace_layout(fresh.get(&id).unwrap(), &restored[&id]).unwrap(),
            layout
        );
    }
    fs::remove_dir_all(root).unwrap();
}
