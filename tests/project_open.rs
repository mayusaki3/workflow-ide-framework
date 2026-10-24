use std::{fs, path::PathBuf};
use workflow_ide_framework::{
    framework_settings::FrameworkSettings,
    project::{
        ApplicationMetadata, PROJECT_FORMAT_VERSION, ProjectContext, ProjectFile, ProjectMetadata,
    },
    project_open::{
        ApplicationProjectInspector, FrameworkSettingsOpen, PendingResourceOperationOpen,
        ProjectOpenSessionError, open_can_continue, open_dirty_state, open_project,
        open_requires_user_decision, open_session,
    },
    project_resource::{ProjectDataCompatibility, ProjectDataConsistency},
    resource_journal::{ResourceOperationJournal, StoredExecutionRoute, StoredOperationKind},
};

fn root(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("wfide-open-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

fn write_project(context: &ProjectContext, data_version: Option<&str>) {
    let file = ProjectFile {
        project: ProjectMetadata {
            format_version: PROJECT_FORMAT_VERSION,
            name: "Open Test".into(),
            description: None,
            language: "en-US".into(),
            save_id: "s1".into(),
            saved_at: "time".into(),
        },
        application: ApplicationMetadata {
            id: "org.example.open".into(),
            name: "Open App".into(),
            description: None,
            language: "en-US".into(),
            data_version: data_version.map(str::to_owned),
        },
    };
    fs::write(context.project_file_path(), file.to_toml().unwrap()).unwrap();
}

struct Inspector {
    seen_version: Option<String>,
    compatibility: ProjectDataCompatibility,
    consistency: ProjectDataConsistency,
}
impl ApplicationProjectInspector for Inspector {
    type Error = &'static str;
    fn inspect_project_data(
        &mut self,
        _: &ProjectContext,
        version: Option<&str>,
    ) -> Result<ProjectDataCompatibility, Self::Error> {
        self.seen_version = version.map(str::to_owned);
        Ok(self.compatibility.clone())
    }
    fn check_project_consistency(
        &mut self,
        _: &ProjectContext,
    ) -> Result<ProjectDataConsistency, Self::Error> {
        Ok(self.consistency.clone())
    }
}

#[test]
fn open_loads_framework_settings_and_passes_stored_version_to_application() {
    let r = root("ok");
    let context = ProjectContext::new(&r);
    write_project(&context, Some("4"));
    fs::create_dir_all(context.framework_directory()).unwrap();
    fs::write(
        context.framework_settings_path(),
        FrameworkSettings::default().to_toml().unwrap(),
    )
    .unwrap();
    let mut app = Inspector {
        seen_version: None,
        compatibility: ProjectDataCompatibility::Compatible,
        consistency: ProjectDataConsistency::Consistent,
    };
    let result = open_project(&context, "org.example.open", &mut app).unwrap();
    assert!(matches!(
        result.framework_settings,
        FrameworkSettingsOpen::Loaded(_)
    ));
    assert_eq!(app.seen_version.as_deref(), Some("4"));
    assert!(!open_requires_user_decision(&result));
    let _ = fs::remove_dir_all(r);
}

#[test]
fn missing_framework_settings_returns_defaults_decision_instead_of_blocking_open() {
    let r = root("missing");
    let context = ProjectContext::new(&r);
    write_project(&context, None);
    let mut app = Inspector {
        seen_version: None,
        compatibility: ProjectDataCompatibility::Compatible,
        consistency: ProjectDataConsistency::Consistent,
    };
    let result = open_project(&context, "org.example.open", &mut app).unwrap();
    assert_eq!(
        result.framework_settings,
        FrameworkSettingsOpen::MissingUseDefaults
    );
    assert!(open_requires_user_decision(&result));
    let _ = fs::remove_dir_all(r);
}

#[test]
fn corrupt_framework_settings_is_recoverable_result_not_project_open_error() {
    let r = root("corrupt");
    let context = ProjectContext::new(&r);
    write_project(&context, None);
    fs::create_dir_all(context.framework_directory()).unwrap();
    fs::write(context.framework_settings_path(), "not = [valid").unwrap();
    let mut app = Inspector {
        seen_version: None,
        compatibility: ProjectDataCompatibility::Compatible,
        consistency: ProjectDataConsistency::Consistent,
    };
    let result = open_project(&context, "org.example.open", &mut app).unwrap();
    assert!(matches!(
        result.framework_settings,
        FrameworkSettingsOpen::InvalidUseDefaults { .. }
    ));
    let _ = fs::remove_dir_all(r);
}

#[test]
fn incompatible_application_is_returned_for_ui_decision() {
    let r = root("incompatible");
    let context = ProjectContext::new(&r);
    write_project(&context, Some("1"));
    let mut app = Inspector {
        seen_version: None,
        compatibility: ProjectDataCompatibility::Incompatible {
            reason: Some("newer data".into()),
            handled: false,
        },
        consistency: ProjectDataConsistency::Consistent,
    };
    let result = open_project(&context, "org.example.open", &mut app).unwrap();
    assert!(open_requires_user_decision(&result));
    assert!(matches!(
        result.application_compatibility,
        ProjectDataCompatibility::Incompatible { .. }
    ));
    let _ = fs::remove_dir_all(r);
}

#[test]
fn inconsistent_application_state_is_returned_without_automatic_recovery() {
    let r = root("inconsistent");
    let context = ProjectContext::new(&r);
    write_project(&context, None);
    let mut app = Inspector {
        seen_version: None,
        compatibility: ProjectDataCompatibility::Compatible,
        consistency: ProjectDataConsistency::Inconsistent {
            reason: Some("save mismatch".into()),
            can_open: true,
            can_recover: true,
            handled: false,
        },
    };
    let result = open_project(&context, "org.example.open", &mut app).unwrap();
    assert!(matches!(
        result.application_consistency,
        ProjectDataConsistency::Inconsistent {
            can_recover: true,
            ..
        }
    ));
    assert!(open_requires_user_decision(&result));
    let _ = fs::remove_dir_all(r);
}

#[test]
fn framework_save_id_mismatch_requires_user_decision() {
    let r = root("save-id-mismatch");
    let context = ProjectContext::new(&r);
    write_project(&context, None);
    fs::create_dir_all(context.framework_directory()).unwrap();
    let settings = FrameworkSettings {
        save_id: Some("different-save".into()),
        ..Default::default()
    };
    fs::write(
        context.framework_settings_path(),
        settings.to_toml().unwrap(),
    )
    .unwrap();
    let mut app = Inspector {
        seen_version: None,
        compatibility: ProjectDataCompatibility::Compatible,
        consistency: ProjectDataConsistency::Consistent,
    };
    let result = open_project(&context, "org.example.open", &mut app).unwrap();
    assert!(
        matches!(result.framework_settings,FrameworkSettingsOpen::SaveIdMismatch{ref project_save_id,ref framework_save_id}
        if project_save_id=="s1" && framework_save_id=="different-save")
    );
    assert!(open_requires_user_decision(&result));
    let _ = fs::remove_dir_all(r);
}

#[test]
fn legacy_framework_settings_without_save_id_remains_openable() {
    let r = root("legacy-no-save-id");
    let context = ProjectContext::new(&r);
    write_project(&context, None);
    fs::create_dir_all(context.framework_directory()).unwrap();
    fs::write(
        context.framework_settings_path(),
        "format_version = 1\nresources = []\n",
    )
    .unwrap();
    let mut app = Inspector {
        seen_version: None,
        compatibility: ProjectDataCompatibility::Compatible,
        consistency: ProjectDataConsistency::Consistent,
    };
    let result = open_project(&context, "org.example.open", &mut app).unwrap();
    assert!(
        matches!(result.framework_settings,FrameworkSettingsOpen::Loaded(ref settings) if settings.save_id.is_none())
    );
    assert!(!open_requires_user_decision(&result));
    let _ = fs::remove_dir_all(r);
}

#[test]
fn incompatible_application_cannot_continue_open() {
    let r = root("incompatible-block");
    let context = ProjectContext::new(&r);
    write_project(&context, None);
    let mut app = Inspector {
        seen_version: None,
        compatibility: ProjectDataCompatibility::Incompatible {
            reason: None,
            handled: true,
        },
        consistency: ProjectDataConsistency::Consistent,
    };
    let result = open_project(&context, "org.example.open", &mut app).unwrap();
    assert!(!open_can_continue(&result));
    let _ = fs::remove_dir_all(r);
}

#[test]
fn inconsistent_can_open_false_cannot_continue_open() {
    let r = root("consistency-block");
    let context = ProjectContext::new(&r);
    write_project(&context, None);
    let mut app = Inspector {
        seen_version: None,
        compatibility: ProjectDataCompatibility::Compatible,
        consistency: ProjectDataConsistency::Inconsistent {
            reason: None,
            can_open: false,
            can_recover: true,
            handled: true,
        },
    };
    let result = open_project(&context, "org.example.open", &mut app).unwrap();
    assert!(!open_can_continue(&result));
    let _ = fs::remove_dir_all(r);
}

#[test]
fn converted_application_marks_application_dirty() {
    let r = root("converted-dirty");
    let context = ProjectContext::new(&r);
    write_project(&context, Some("1"));
    let mut app = Inspector {
        seen_version: None,
        compatibility: ProjectDataCompatibility::Converted {
            reason: Some("migrated".into()),
            handled: true,
        },
        consistency: ProjectDataConsistency::Consistent,
    };
    let result = open_project(&context, "org.example.open", &mut app).unwrap();
    assert!(open_can_continue(&result));
    let dirty = open_dirty_state(&result);
    assert!(dirty.application);
    assert!(dirty.is_dirty());
    let _ = fs::remove_dir_all(r);
}

#[test]
fn framework_recovery_choice_is_framework_dirty() {
    let r = root("framework-dirty");
    let context = ProjectContext::new(&r);
    write_project(&context, None);
    let mut app = Inspector {
        seen_version: None,
        compatibility: ProjectDataCompatibility::Compatible,
        consistency: ProjectDataConsistency::Consistent,
    };
    let result = open_project(&context, "org.example.open", &mut app).unwrap();
    let dirty = open_dirty_state(&result);
    assert!(dirty.framework);
    assert!(!dirty.application);
    let _ = fs::remove_dir_all(r);
}

#[test]
fn successful_open_session_uses_project_root_save_id_and_clean_state() {
    let r = root("session-ok");
    let context = ProjectContext::new(&r);
    write_project(&context, None);
    fs::create_dir_all(context.framework_directory()).unwrap();
    fs::write(
        context.framework_settings_path(),
        FrameworkSettings::default().to_toml().unwrap(),
    )
    .unwrap();
    let mut app = Inspector {
        seen_version: None,
        compatibility: ProjectDataCompatibility::Compatible,
        consistency: ProjectDataConsistency::Consistent,
    };
    let result = open_project(&context, "org.example.open", &mut app).unwrap();
    let session = open_session(context.clone(), &result).unwrap();
    assert_eq!(session.context(), Some(&context));
    assert_eq!(session.current_save_id.as_deref(), Some("s1"));
    assert!(!session.dirty.is_dirty());
    let _ = fs::remove_dir_all(r);
}

#[test]
fn converted_open_session_preserves_application_dirty() {
    let r = root("session-converted");
    let context = ProjectContext::new(&r);
    write_project(&context, Some("1"));
    fs::create_dir_all(context.framework_directory()).unwrap();
    fs::write(
        context.framework_settings_path(),
        FrameworkSettings::default().to_toml().unwrap(),
    )
    .unwrap();
    let mut app = Inspector {
        seen_version: None,
        compatibility: ProjectDataCompatibility::Converted {
            reason: None,
            handled: true,
        },
        consistency: ProjectDataConsistency::Consistent,
    };
    let result = open_project(&context, "org.example.open", &mut app).unwrap();
    let session = open_session(context.clone(), &result).unwrap();
    assert!(session.dirty.application);
    assert_eq!(session.current_save_id.as_deref(), Some("s1"));
    let _ = fs::remove_dir_all(r);
}

#[test]
fn blocked_open_cannot_create_current_project_session() {
    let r = root("session-blocked");
    let context = ProjectContext::new(&r);
    write_project(&context, None);
    let mut app = Inspector {
        seen_version: None,
        compatibility: ProjectDataCompatibility::Incompatible {
            reason: None,
            handled: false,
        },
        consistency: ProjectDataConsistency::Consistent,
    };
    let result = open_project(&context, "org.example.open", &mut app).unwrap();
    assert_eq!(
        open_session(context.clone(), &result),
        Err(ProjectOpenSessionError::CannotContinue)
    );
    let _ = fs::remove_dir_all(r);
}

#[test]
fn pending_resource_journal_is_reported_without_blocking_open() {
    let r = root("pending-journal");
    let context = ProjectContext::new(&r);
    write_project(&context, None);
    fs::create_dir_all(context.framework_directory()).unwrap();
    fs::write(
        context.framework_settings_path(),
        FrameworkSettings::default().to_toml().unwrap(),
    )
    .unwrap();
    let journal = ResourceOperationJournal {
        format_version: 1,
        operation_id: "pending-1".into(),
        kind: StoredOperationKind::Delete,
        execution_route: StoredExecutionRoute::Framework,
        items: vec![],
        application: None,
    };
    journal.save(&context).unwrap();
    let mut app = Inspector {
        seen_version: None,
        compatibility: ProjectDataCompatibility::Compatible,
        consistency: ProjectDataConsistency::Consistent,
    };
    let result = open_project(&context, "org.example.open", &mut app).unwrap();
    assert!(
        matches!(result.pending_resource_operation,PendingResourceOperationOpen::Pending(ref pending) if pending.operation_id=="pending-1")
    );
    assert!(open_requires_user_decision(&result));
    assert!(open_can_continue(&result));
    assert!(open_session(context.clone(), &result).is_ok());
    let _ = fs::remove_dir_all(r);
}

#[test]
fn invalid_resource_journal_is_reported_without_failing_project_parse() {
    let r = root("invalid-journal");
    let context = ProjectContext::new(&r);
    write_project(&context, None);
    fs::create_dir_all(context.framework_directory()).unwrap();
    fs::write(
        context.framework_settings_path(),
        FrameworkSettings::default().to_toml().unwrap(),
    )
    .unwrap();
    fs::write(
        context
            .framework_directory()
            .join("pending_resource_operation.toml"),
        "not = [valid",
    )
    .unwrap();
    let mut app = Inspector {
        seen_version: None,
        compatibility: ProjectDataCompatibility::Compatible,
        consistency: ProjectDataConsistency::Consistent,
    };
    let result = open_project(&context, "org.example.open", &mut app).unwrap();
    assert!(matches!(
        result.pending_resource_operation,
        PendingResourceOperationOpen::Invalid { .. }
    ));
    assert!(open_requires_user_decision(&result));
    assert!(open_can_continue(&result));
    let _ = fs::remove_dir_all(r);
}

#[test]
fn committed_generation_never_opens_stale_legacy_files() {
    use workflow_ide_framework::project_open::ProjectOpenError;
    use workflow_ide_framework::project_save_generation::committed_path;
    let r = root("generation-block");
    let context = ProjectContext::new(&r);
    write_project(&context, None);
    let mut app = Inspector {
        seen_version: None,
        compatibility: ProjectDataCompatibility::Compatible,
        consistency: ProjectDataConsistency::Consistent,
    };
    let pointer = committed_path(&context);
    fs::create_dir_all(pointer.parent().unwrap()).unwrap();
    fs::write(&pointer, "format_version = 1\ngeneration_id = 'snapshot'\n").unwrap();
    assert!(matches!(
        open_project(&context, "org.example.open", &mut app),
        Err(ProjectOpenError::Generation(_))
    ));
    fs::create_dir_all(
        context
            .framework_directory()
            .join("save-generations/snapshot/application"),
    )
    .unwrap();
    fs::write(
        context
            .framework_directory()
            .join("save-generations/snapshot/project.toml"),
        "test",
    )
    .unwrap();
    fs::write(
        context
            .framework_directory()
            .join("save-generations/snapshot/framework_settings.toml"),
        "test",
    )
    .unwrap();
    assert!(matches!(
        open_project(&context, "org.example.open", &mut app),
        Err(ProjectOpenError::GenerationOpenNotSupported)
    ));
    assert!(app.seen_version.is_none());
    fs::remove_dir_all(r).unwrap();
}

struct GenerationInspector {
    enabled: bool,
    called: bool,
}

impl workflow_ide_framework::project_adapter::ApplicationProjectAdapter for GenerationInspector {
    type Error = &'static str;

    fn initialize_project(&mut self, _: &ProjectContext) -> Result<(), Self::Error> {
        Ok(())
    }

    fn inspect_project_data(
        &mut self,
        _: &ProjectContext,
        _: Option<&str>,
    ) -> Result<ProjectDataCompatibility, Self::Error> {
        panic!("legacy inspection must never run for a committed generation")
    }

    fn check_project_consistency(
        &mut self,
        _: &ProjectContext,
    ) -> Result<ProjectDataConsistency, Self::Error> {
        panic!("legacy consistency must never run for a committed generation")
    }

    fn inspect_project_data_generation(
        &mut self,
        project_root: &ProjectContext,
        generation: &ProjectContext,
        version: Option<&str>,
    ) -> Result<Option<(ProjectDataCompatibility, ProjectDataConsistency)>, Self::Error> {
        self.called = true;
        if !self.enabled {
            return Ok(None);
        }
        assert_ne!(project_root.root(), generation.root());
        assert_eq!(version, Some("generation-v1"));
        assert_eq!(
            fs::read_to_string(generation.application_directory().join("catalog.json")).unwrap(),
            "generation-data"
        );
        assert_eq!(
            fs::read_to_string(project_root.application_directory().join("catalog.json")).unwrap(),
            "stale-legacy"
        );
        Ok(Some((
            ProjectDataCompatibility::Compatible,
            ProjectDataConsistency::Consistent,
        )))
    }

    fn save_project_data(
        &mut self,
        _: &ProjectContext,
        _: &str,
    ) -> Result<workflow_ide_framework::project_save::ApplicationSaveResult, Self::Error> {
        panic!("save must not run during inspection")
    }

    fn save_project_data_as(
        &mut self,
        _: Option<&ProjectContext>,
        _: &ProjectContext,
        _: &str,
    ) -> Result<workflow_ide_framework::project_save::ApplicationSaveResult, Self::Error> {
        panic!("save-as must not run during inspection")
    }
}

fn committed_fixture(name: &str) -> (PathBuf, ProjectContext) {
    use workflow_ide_framework::project_save_generation::{
        begin_generation, validate_and_commit_generation,
    };
    let r = root(name);
    let context = ProjectContext::new(&r);
    fs::create_dir_all(context.application_directory()).unwrap();
    fs::write(
        context.application_directory().join("catalog.json"),
        "stale-legacy",
    )
    .unwrap();
    let stage = begin_generation(&context, "snapshot").unwrap();
    fs::create_dir_all(stage.join("application")).unwrap();
    fs::write(stage.join("application/catalog.json"), "generation-data").unwrap();
    let project = ProjectFile {
        project: ProjectMetadata {
            format_version: PROJECT_FORMAT_VERSION,
            name: "Generation Project".into(),
            description: None,
            language: "en-US".into(),
            save_id: "generation-save".into(),
            saved_at: "now".into(),
        },
        application: ApplicationMetadata {
            id: "org.example.open".into(),
            name: "Open App".into(),
            description: None,
            language: "en-US".into(),
            data_version: Some("generation-v1".into()),
        },
    };
    fs::write(stage.join("project.toml"), project.to_toml().unwrap()).unwrap();
    let settings = FrameworkSettings {
        save_id: Some("generation-save".into()),
        ..Default::default()
    };
    fs::write(
        stage.join("framework_settings.toml"),
        settings.to_toml().unwrap(),
    )
    .unwrap();
    validate_and_commit_generation(
        &context,
        "snapshot",
        "org.example.open",
        "generation-save",
        |_| Ok(()),
    )
    .unwrap();
    (r, context)
}

#[test]
fn committed_inspection_uses_only_generation_snapshot() {
    use workflow_ide_framework::project_open::inspect_committed_generation;
    let (r, context) = committed_fixture("inspect-generation");
    let mut app = GenerationInspector {
        enabled: true,
        called: false,
    };
    let result = inspect_committed_generation(&context, "org.example.open", &mut app)
        .unwrap()
        .unwrap();
    assert!(app.called);
    assert_eq!(result.project_file.project.name, "Generation Project");
    assert_eq!(result.project_file.project.save_id, "generation-save");
    assert!(
        matches!(result.framework_settings, FrameworkSettingsOpen::Loaded(ref s) if s.save_id.as_deref() == Some("generation-save"))
    );
    fs::remove_dir_all(r).unwrap();
}

#[test]
fn committed_inspection_rejects_adapter_without_generation_support() {
    use workflow_ide_framework::project_open::{ProjectOpenError, inspect_committed_generation};
    let (r, context) = committed_fixture("inspect-unsupported");
    let mut app = GenerationInspector {
        enabled: false,
        called: false,
    };
    assert!(matches!(
        inspect_committed_generation(&context, "org.example.open", &mut app),
        Err(ProjectOpenError::GenerationOpenNotSupported)
    ));
    assert!(app.called);
    fs::remove_dir_all(r).unwrap();
}

#[test]
fn committed_inspection_rejects_corrupt_snapshot_before_adapter() {
    use workflow_ide_framework::project_open::{ProjectOpenError, inspect_committed_generation};
    use workflow_ide_framework::project_save_generation::generation_path;
    let (r, context) = committed_fixture("inspect-corrupt");
    let snapshot = generation_path(&context, "snapshot").unwrap();
    fs::write(
        snapshot.join("framework_settings.toml"),
        "format_version = 1\nsave_id = 'other'\n",
    )
    .unwrap();
    let mut app = GenerationInspector {
        enabled: true,
        called: false,
    };
    assert!(matches!(
        inspect_committed_generation(&context, "org.example.open", &mut app),
        Err(ProjectOpenError::Generation(_))
    ));
    assert!(!app.called);
    fs::remove_dir_all(r).unwrap();
}

#[test]
fn committed_inspection_returns_none_for_legacy_project() {
    use workflow_ide_framework::project_open::inspect_committed_generation;
    let r = root("inspect-legacy");
    let context = ProjectContext::new(&r);
    let mut app = GenerationInspector {
        enabled: true,
        called: false,
    };
    assert!(
        inspect_committed_generation(&context, "org.example.open", &mut app)
            .unwrap()
            .is_none()
    );
    assert!(!app.called);
    fs::remove_dir_all(r).unwrap();
}

#[test]
fn controller_opens_clean_generation_and_never_uses_legacy_save() {
    use workflow_ide_framework::project_controller::{ProjectCommandResult, ProjectController};
    let (r, context) = committed_fixture("controller-clean-generation");
    let mut app = GenerationInspector {
        enabled: true,
        called: false,
    };
    let mut controller = ProjectController::new("org.example.open", "Open App");
    assert_eq!(
        controller.open(context.clone(), &mut app),
        ProjectCommandResult::Completed
    );
    assert!(app.called);
    assert_eq!(
        controller.project_file.as_ref().unwrap().project.save_id,
        "generation-save"
    );
    assert_eq!(
        controller
            .session
            .as_ref()
            .unwrap()
            .current_save_id
            .as_deref(),
        Some("generation-save")
    );
    assert!(matches!(
        controller.save(&mut app, "next-save", "later"),
        ProjectCommandResult::Failed(_)
    ));
    assert!(matches!(
        controller.save_as(
            ProjectContext::new(r.join("copy")),
            "Copy",
            &mut app,
            "next-save",
            "later",
        ),
        ProjectCommandResult::Failed(_)
    ));
    assert_eq!(
        fs::read_to_string(context.application_directory().join("catalog.json")).unwrap(),
        "stale-legacy"
    );
    fs::remove_dir_all(r).unwrap();
}

#[test]
fn controller_generation_open_rejects_unsupported_and_preserves_current_session() {
    use workflow_ide_framework::project_controller::{ProjectCommandResult, ProjectController};
    let (r, context) = committed_fixture("controller-unsupported-generation");
    let mut controller = ProjectController::new("org.example.open", "Open App");
    let mut app = GenerationInspector {
        enabled: false,
        called: false,
    };
    assert!(matches!(
        controller.open(context, &mut app),
        ProjectCommandResult::Failed(_)
    ));
    assert!(!controller.is_open());
    assert!(app.called);
    fs::remove_dir_all(r).unwrap();
}

#[test]
fn accept_open_rejects_fabricated_generation_result() {
    use workflow_ide_framework::project_controller::{ProjectCommandResult, ProjectController};
    use workflow_ide_framework::project_open::inspect_committed_generation;
    let (r, context) = committed_fixture("controller-forged-generation");
    let mut app = GenerationInspector {
        enabled: true,
        called: false,
    };
    let mut result = inspect_committed_generation(&context, "org.example.open", &mut app)
        .unwrap()
        .unwrap();
    result.project_file.project.save_id = "forged".into();
    let mut controller = ProjectController::new("org.example.open", "Open App");
    assert!(matches!(
        controller.accept_open(context, result),
        ProjectCommandResult::Failed(_)
    ));
    assert!(!controller.is_open());
    fs::remove_dir_all(r).unwrap();
}

#[test]
fn controller_rejects_untrusted_but_authentic_generation_result() {
    use workflow_ide_framework::project_controller::{ProjectCommandResult, ProjectController};
    use workflow_ide_framework::project_open::inspect_committed_generation;
    let (root, context) = committed_fixture("controller-untrusted-generation");
    let mut app = GenerationInspector {
        enabled: true,
        called: false,
    };
    let result = inspect_committed_generation(&context, "org.example.open", &mut app)
        .unwrap()
        .unwrap();
    let mut controller = ProjectController::new("org.example.open", "Open App");
    assert!(matches!(
        controller.accept_open(context, result),
        ProjectCommandResult::Failed(_)
    ));
    assert!(!controller.is_open());
    fs::remove_dir_all(root).unwrap();
}
