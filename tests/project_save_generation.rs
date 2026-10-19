use std::{fs, path::Path};
use workflow_ide_framework::{
    project::ProjectContext,
    project_save_generation::{
        GenerationError, begin_generation, commit_generation, committed_path, generation_path,
        promote_generation, read_committed_generation, staging_path, validate_generation_contents,
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
    complete(&staging);
    promote_generation(&context, "generation-1").unwrap();
    assert!(read_committed_generation(&context).unwrap().is_none());
    commit_generation(&context, "generation-1").unwrap();
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
    complete(&begin_generation(&context, "old").unwrap());
    promote_generation(&context, "old").unwrap();
    commit_generation(&context, "old").unwrap();
    complete(&begin_generation(&context, "new").unwrap());
    assert_eq!(
        read_committed_generation(&context)
            .unwrap()
            .unwrap()
            .generation_id,
        "old"
    );
    promote_generation(&context, "new").unwrap();
    assert_eq!(
        read_committed_generation(&context)
            .unwrap()
            .unwrap()
            .generation_id,
        "old"
    );
    commit_generation(&context, "new").unwrap();
    assert_eq!(
        read_committed_generation(&context)
            .unwrap()
            .unwrap()
            .generation_id,
        "new"
    );
    assert!(generation_path(&context, "old").unwrap().is_dir());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn save_cons_003_incomplete_candidate_cannot_be_promoted_or_committed() {
    let (root, context) = setup("incomplete");
    begin_generation(&context, "partial").unwrap();
    assert!(matches!(
        promote_generation(&context, "partial"),
        Err(GenerationError::IncompleteCandidate)
    ));
    assert!(matches!(
        commit_generation(&context, "partial"),
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
    fs::write(stage.join("framework_settings.toml"), settings.to_toml().unwrap()).unwrap();
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
