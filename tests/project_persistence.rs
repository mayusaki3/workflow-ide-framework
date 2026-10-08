use workflow_ide_framework::{
    project::{
        ApplicationMetadata, PROJECT_FORMAT_VERSION, ProjectContext, ProjectFile, ProjectFileError,
        ProjectMetadata,
    },
    project_lifecycle::{ProjectSession, SaveStateError},
};

fn file() -> ProjectFile {
    ProjectFile {
        project: ProjectMetadata {
            format_version: PROJECT_FORMAT_VERSION,
            name: "日本語プロジェクト".into(),
            description: Some("説明".into()),
            language: "ja-JP".into(),
            save_id: "save-001".into(),
            saved_at: "2026-09-27T13:00:00+09:00".into(),
        },
        application: ApplicationMetadata {
            id: "org.example.sample".into(),
            name: "Sample".into(),
            description: None,
            language: "en-US".into(),
            data_version: Some("3".into()),
        },
    }
}

#[test]
fn project_file_round_trips_toml() {
    let original = file();
    let text = original.to_toml().unwrap();
    let parsed = ProjectFile::from_toml(&text, "org.example.sample").unwrap();
    assert_eq!(parsed, original);
}

#[test]
fn project_file_rejects_different_application() {
    let text = file().to_toml().unwrap();
    assert!(matches!(
        ProjectFile::from_toml(&text, "org.other.app"),
        Err(ProjectFileError::ApplicationMismatch { .. })
    ));
}

#[test]
fn project_file_rejects_newer_framework_format() {
    let mut newer = file();
    newer.project.format_version = PROJECT_FORMAT_VERSION + 1;
    let text = newer.to_toml().unwrap();
    assert_eq!(
        ProjectFile::from_toml(&text, "org.example.sample"),
        Err(ProjectFileError::UnsupportedNewerFormat {
            found: PROJECT_FORMAT_VERSION + 1,
            supported: PROJECT_FORMAT_VERSION
        })
    );
}

#[test]
fn project_context_uses_fixed_v010_paths() {
    let context = ProjectContext::new("demo");
    assert_eq!(
        context.project_file_path(),
        std::path::PathBuf::from("demo").join("project.toml")
    );
    assert_eq!(
        context.framework_settings_path(),
        std::path::PathBuf::from("demo")
            .join("framework")
            .join("framework_settings.toml")
    );
    assert_eq!(
        context.resource_root(),
        std::path::PathBuf::from("demo").join("resources")
    );
    assert_eq!(
        context.application_directory(),
        std::path::PathBuf::from("demo").join("application")
    );
}

#[test]
fn save_clears_dirty_only_after_all_three_parts_complete() {
    let mut session = ProjectSession::default();
    session.mark_metadata_dirty();
    session.mark_framework_dirty();
    session.mark_application_dirty();

    let incomplete = session
        .begin_save("save-002")
        .application_saved()
        .framework_saved();
    assert_eq!(
        session.complete_save(incomplete),
        Err(SaveStateError::ProjectFileNotSaved)
    );
    assert!(session.dirty.is_dirty());
    assert_eq!(session.current_save_id, None);

    let complete = session
        .begin_save("save-002")
        .application_saved()
        .framework_saved()
        .project_file_saved();
    session.complete_save(complete).unwrap();
    assert!(!session.dirty.is_dirty());
    assert_eq!(session.current_save_id.as_deref(), Some("save-002"));
}

#[test]
fn save_state_reports_first_missing_owner() {
    let mut session = ProjectSession::default();
    assert_eq!(
        session.complete_save(session.begin_save("x")),
        Err(SaveStateError::ApplicationNotSaved)
    );
    assert_eq!(
        session.complete_save(session.begin_save("x").application_saved()),
        Err(SaveStateError::FrameworkNotSaved)
    );
}
