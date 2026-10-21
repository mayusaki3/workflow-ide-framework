use std::{fs, path::Path};
use workflow_ide_framework::{
    project::ProjectContext,
    project_save_generation::{
        GenerationError, begin_generation, committed_path, generation_path,
        read_committed_generation, read_validated_committed_generation,
        resolve_validated_committed_paths, staging_path, validate_and_commit_generation,
        validate_generation_contents,
    },
};

fn setup(name: &str) -> (std::path::PathBuf, ProjectContext) {
    let root = std::env::temp_dir().join(format!("wfide-generation-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let context = ProjectContext::new(&root);
    (root, context)
}

fn complete(path: &Path) {
    fs::create_dir_all(path.join("application")).unwrap();
    fs::write(path.join("application").join("catalog.json"), "{}").unwrap();
    fs::write(path.join("framework_settings.toml"), "format_version = 1\n").unwrap();
    fs::write(path.join("project.toml"), "[project]\n").unwrap();
}

#[test]
fn save_cons_001_commits_only_complete_generation() {
    let (root, context) = setup("success");
    assert!(read_committed_generation(&context).unwrap().is_none());
    let staging = begin_generation(&context, "generation-1").unwrap();
    complete_valid(&staging, "save-1");
    assert!(read_committed_generation(&context).unwrap().is_none());
    validate_and_commit_generation(
        &context,
        "generation-1",
        "org.example.test",
        "save-1",
        |_| Ok(()),
    )
    .unwrap();
    assert_eq!(
        read_committed_generation(&context)
            .unwrap()
            .unwrap()
            .generation_id,
        "generation-1"
    );
    assert!(
        generation_path(&context, "generation-1")
            .unwrap()
            .join("application/catalog.json")
            .is_file()
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn save_cons_004_uncommitted_candidate_does_not_replace_previous_generation() {
    let (root, context) = setup("interrupt");
    complete_valid(&begin_generation(&context, "old").unwrap(), "save-old");
    validate_and_commit_generation(&context, "old", "org.example.test", "save-old", |_| Ok(()))
        .unwrap();
    complete_valid(&begin_generation(&context, "new").unwrap(), "save-new");
    assert_eq!(
        read_committed_generation(&context)
            .unwrap()
            .unwrap()
            .generation_id,
        "old"
    );
    fs::rename(
        staging_path(&context, "new").unwrap(),
        generation_path(&context, "new").unwrap(),
    )
    .unwrap();
    assert_eq!(
        read_committed_generation(&context)
            .unwrap()
            .unwrap()
            .generation_id,
        "old"
    );
    assert!(generation_path(&context, "new").unwrap().is_dir());
    assert_eq!(
        read_committed_generation(&context)
            .unwrap()
            .unwrap()
            .generation_id,
        "old"
    );
    assert!(generation_path(&context, "old").unwrap().is_dir());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn save_cons_003_incomplete_candidate_cannot_be_promoted_or_committed() {
    let (root, context) = setup("incomplete");
    begin_generation(&context, "partial").unwrap();
    assert!(matches!(
        validate_and_commit_generation(
            &context,
            "partial",
            "org.example.test",
            "save-partial",
            |_| Ok(())
        ),
        Err(GenerationError::IncompleteCandidate)
    ));
    assert!(matches!(
        validate_and_commit_generation(
            &context,
            "missing",
            "org.example.test",
            "save-missing",
            |_| Ok(())
        ),
        Err(GenerationError::MissingCandidate)
    ));
    assert!(read_committed_generation(&context).unwrap().is_none());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn save_cons_004_corrupt_or_missing_commit_target_is_not_legacy_fallback() {
    let (root, context) = setup("corrupt");
    fs::create_dir_all(context.framework_directory().join("save-generations")).unwrap();
    fs::write(committed_path(&context), "not valid toml = [").unwrap();
    assert!(matches!(
        read_committed_generation(&context),
        Err(GenerationError::InvalidCommit(_))
    ));
    fs::write(
        committed_path(&context),
        "format_version = 1\ngeneration_id = 'missing'\n",
    )
    .unwrap();
    assert!(matches!(
        read_committed_generation(&context),
        Err(GenerationError::MissingCommittedGeneration)
    ));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn save_cons_010_rejects_invalid_and_duplicate_generation_ids() {
    let (root, context) = setup("ids");
    for id in ["", "../escape", "nested/path", "a.b"] {
        assert!(matches!(
            staging_path(&context, id),
            Err(GenerationError::InvalidId)
        ));
    }
    begin_generation(&context, "unique").unwrap();
    assert!(matches!(
        begin_generation(&context, "unique"),
        Err(GenerationError::AlreadyExists)
    ));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn save_cons_rejects_invalid_project_and_mismatched_save_ids() {
    let (root, context) = setup("content-validation");
    let stage = begin_generation(&context, "candidate").unwrap();
    complete(&stage);
    assert!(matches!(
        validate_generation_contents(&stage, "org.example.test", "generation-42"),
        Err(GenerationError::InvalidCandidate(_))
    ));
    let project = workflow_ide_framework::project::ProjectFile {
        project: workflow_ide_framework::project::ProjectMetadata {
            format_version: 1,
            name: "Test".into(),
            description: None,
            language: "en-US".into(),
            save_id: "generation-42".into(),
            saved_at: "now".into(),
        },
        application: workflow_ide_framework::project::ApplicationMetadata {
            id: "org.example.test".into(),
            name: "Test".into(),
            description: None,
            language: "en-US".into(),
            data_version: None,
        },
    };
    fs::write(stage.join("project.toml"), project.to_toml().unwrap()).unwrap();
    assert!(matches!(
        validate_generation_contents(&stage, "org.example.test", "generation-42"),
        Err(GenerationError::InvalidCandidate(_))
    ));
    let settings = workflow_ide_framework::framework_settings::FrameworkSettings {
        save_id: Some("generation-42".into()),
        ..Default::default()
    };
    fs::write(
        stage.join("framework_settings.toml"),
        settings.to_toml().unwrap(),
    )
    .unwrap();
    validate_generation_contents(&stage, "org.example.test", "generation-42").unwrap();
    assert!(matches!(
        validate_generation_contents(&stage, "org.example.other", "generation-42"),
        Err(GenerationError::InvalidCandidate(_))
    ));
    assert!(matches!(
        validate_generation_contents(&stage, "org.example.test", "generation-43"),
        Err(GenerationError::InvalidCandidate(_))
    ));
    fs::remove_dir_all(root).unwrap();
}

fn complete_valid(path: &Path, save_id: &str) {
    complete(path);
    let project = workflow_ide_framework::project::ProjectFile {
        project: workflow_ide_framework::project::ProjectMetadata {
            format_version: 1,
            name: "Test".into(),
            description: None,
            language: "en-US".into(),
            save_id: save_id.into(),
            saved_at: "now".into(),
        },
        application: workflow_ide_framework::project::ApplicationMetadata {
            id: "org.example.test".into(),
            name: "Test".into(),
            description: None,
            language: "en-US".into(),
            data_version: None,
        },
    };
    fs::write(path.join("project.toml"), project.to_toml().unwrap()).unwrap();
    let settings = workflow_ide_framework::framework_settings::FrameworkSettings {
        save_id: Some(save_id.into()),
        ..Default::default()
    };
    fs::write(
        path.join("framework_settings.toml"),
        settings.to_toml().unwrap(),
    )
    .unwrap();
}

#[test]
fn save_cons_validated_publish_preserves_pointer_on_failure() {
    let (root, context) = setup("validated-publish");
    let old = begin_generation(&context, "old").unwrap();
    complete_valid(&old, "save-old");
    validate_and_commit_generation(&context, "old", "org.example.test", "save-old", |app| {
        assert!(app.join("catalog.json").is_file());
        Ok(())
    })
    .unwrap();
    let next = begin_generation(&context, "new").unwrap();
    complete_valid(&next, "save-new");
    assert!(matches!(
        validate_and_commit_generation(&context, "new", "org.example.test", "wrong", |_| Ok(())),
        Err(GenerationError::InvalidCandidate(_))
    ));
    assert!(matches!(
        validate_and_commit_generation(&context, "new", "org.example.test", "save-new", |_| {
            Err(GenerationError::InvalidCandidate(
                "application rejected".into(),
            ))
        }),
        Err(GenerationError::InvalidCandidate(_))
    ));
    assert_eq!(
        read_committed_generation(&context)
            .unwrap()
            .unwrap()
            .generation_id,
        "old"
    );
    assert!(staging_path(&context, "new").unwrap().is_dir());
    assert!(!generation_path(&context, "new").unwrap().exists());
    validate_and_commit_generation(&context, "new", "org.example.test", "save-new", |_| Ok(()))
        .unwrap();
    assert_eq!(
        read_committed_generation(&context)
            .unwrap()
            .unwrap()
            .generation_id,
        "new"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn save_cons_read_rejects_corrupt_committed_contents_without_fallback() {
    let (root, context) = setup("validated-read");
    assert!(
        read_validated_committed_generation(&context, "org.example.test", |_| Ok(()))
            .unwrap()
            .is_none()
    );
    let stage = begin_generation(&context, "good").unwrap();
    complete_valid(&stage, "save-good");
    validate_and_commit_generation(
        &context,
        "good",
        "org.example.test",
        "save-good",
        |_| Ok(()),
    )
    .unwrap();
    assert_eq!(
        read_validated_committed_generation(&context, "org.example.test", |app| {
            assert!(app.join("catalog.json").is_file());
            Ok(())
        })
        .unwrap()
        .unwrap()
        .generation_id,
        "good"
    );
    assert!(matches!(
        read_validated_committed_generation(&context, "wrong.application", |_| Ok(())),
        Err(GenerationError::InvalidCandidate(_))
    ));
    assert!(matches!(
        read_validated_committed_generation(&context, "org.example.test", |_| {
            Err(GenerationError::InvalidCandidate("invalid app data".into()))
        }),
        Err(GenerationError::InvalidCandidate(_))
    ));
    let committed = generation_path(&context, "good").unwrap();
    fs::write(
        committed.join("framework_settings.toml"),
        "format_version = 1\nsave_id = 'wrong'\n",
    )
    .unwrap();
    assert!(matches!(
        read_validated_committed_generation(&context, "org.example.test", |_| Ok(())),
        Err(GenerationError::InvalidCandidate(_))
    ));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn save_cons_resolved_paths_are_generation_scoped_not_legacy_layout() {
    let (root, context) = setup("resolved-paths");
    assert!(
        resolve_validated_committed_paths(&context, "org.example.test", |_| Ok(()))
            .unwrap()
            .is_none()
    );
    let stage = begin_generation(&context, "snapshot").unwrap();
    complete_valid(&stage, "save-snapshot");
    validate_and_commit_generation(
        &context,
        "snapshot",
        "org.example.test",
        "save-snapshot",
        |_| Ok(()),
    )
    .unwrap();
    let resolved = resolve_validated_committed_paths(&context, "org.example.test", |application| {
        assert!(application.join("catalog.json").is_file());
        Ok(())
    })
    .unwrap()
    .unwrap();
    let generation = generation_path(&context, "snapshot").unwrap();
    assert_eq!(resolved.generation_id, "snapshot");
    assert_eq!(resolved.project_file, generation.join("project.toml"));
    assert_eq!(
        resolved.framework_settings,
        generation.join("framework_settings.toml")
    );
    assert_eq!(
        resolved.application_directory,
        generation.join("application")
    );
    assert_ne!(resolved.project_file, context.project_file_path());
    assert_ne!(
        resolved.framework_settings,
        context.framework_settings_path()
    );
    assert_ne!(
        resolved.application_directory,
        context.application_directory()
    );
    fs::write(committed_path(&context), "invalid = [").unwrap();
    assert!(matches!(
        resolve_validated_committed_paths(&context, "org.example.test", |_| Ok(())),
        Err(GenerationError::InvalidCommit(_))
    ));
    fs::remove_dir_all(root).unwrap();
}
