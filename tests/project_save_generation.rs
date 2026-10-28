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

struct StagedSaver {
    mode: &'static str,
    calls: usize,
}

impl workflow_ide_framework::project_adapter::ApplicationProjectAdapter for StagedSaver {
    type Error = String;

    fn initialize_project(&mut self, _: &ProjectContext) -> Result<(), String> {
        Ok(())
    }

    fn inspect_project_data(
        &mut self,
        _: &ProjectContext,
        _: Option<&str>,
    ) -> Result<workflow_ide_framework::project_resource::ProjectDataCompatibility, String> {
        panic!("legacy inspection must not be used by staged save")
    }

    fn check_project_consistency(
        &mut self,
        _: &ProjectContext,
    ) -> Result<workflow_ide_framework::project_resource::ProjectDataConsistency, String> {
        panic!("legacy consistency must not be used by staged save")
    }

    fn save_project_data(
        &mut self,
        _: &ProjectContext,
        _: &str,
    ) -> Result<workflow_ide_framework::project_save::ApplicationSaveResult, String> {
        panic!("legacy save must never be used")
    }

    fn validate_project_data_staged(
        &mut self,
        _: &ProjectContext,
        destination: &ProjectContext,
        stored_data_version: Option<&str>,
    ) -> Result<Option<()>, String> {
        match self.mode {
            "success" | "resave" | "failure" | "validation-error" | "validation-unsupported" => {
                if stored_data_version != Some("staged-v1") {
                    return Err("unexpected data version".into());
                }
                let data =
                    fs::read_to_string(destination.application_directory().join("catalog.json"))
                        .map_err(|e| e.to_string())?;
                if data.is_empty() {
                    return Err("empty staged payload".into());
                }
                match self.mode {
                    "validation-error" => Err("rejected staged payload".into()),
                    "validation-unsupported" => Ok(None),
                    _ => Ok(Some(())),
                }
            }
            "unsupported" => Ok(None),
            _ => unreachable!(),
        }
    }

    fn inspect_project_data_generation(
        &mut self,
        _: &ProjectContext,
        generation: &ProjectContext,
        version: Option<&str>,
    ) -> Result<
        Option<(
            workflow_ide_framework::project_resource::ProjectDataCompatibility,
            workflow_ide_framework::project_resource::ProjectDataConsistency,
        )>,
        String,
    > {
        use workflow_ide_framework::project_resource::{
            ProjectDataCompatibility as Compatibility, ProjectDataConsistency as Consistency,
        };
        assert_eq!(version, Some("staged-v1"));
        let data = fs::read_to_string(generation.application_directory().join("catalog.json"))
            .map_err(|e| e.to_string())?;
        if data.is_empty() {
            return Err("empty committed application".into());
        }
        match self.mode {
            "inspect-unsupported" => Ok(None),
            "inspect-error" => Err("committed inspection failed".into()),
            "inspect-incompatible" => Ok(Some((
                Compatibility::Incompatible {
                    reason: None,
                    handled: false,
                },
                Consistency::Consistent,
            ))),
            "inspect-inconsistent" => Ok(Some((
                Compatibility::Compatible,
                Consistency::Inconsistent {
                    reason: None,
                    can_open: false,
                    can_recover: false,
                    handled: false,
                },
            ))),
            _ => Ok(Some((Compatibility::Compatible, Consistency::Consistent))),
        }
    }

    fn save_project_data_as(
        &mut self,
        _: Option<&ProjectContext>,
        _: &ProjectContext,
        _: &str,
    ) -> Result<workflow_ide_framework::project_save::ApplicationSaveResult, String> {
        panic!("legacy save-as must never be used")
    }

    fn save_project_data_staged(
        &mut self,
        source: &ProjectContext,
        destination: &ProjectContext,
        save_id: &str,
    ) -> Result<Option<workflow_ide_framework::project_save::ApplicationSaveResult>, String> {
        self.calls += 1;
        assert_ne!(source.root(), destination.root());
        match self.mode {
            "unsupported" => Ok(None),
            "failure" => Err("staged application failure".into()),
            "success" | "resave" | "validation-error" | "validation-unsupported" => {
                if self.mode == "resave" {
                    assert_eq!(
                        fs::read_to_string(source.application_directory().join("catalog.json"))
                            .unwrap(),
                        "save-old"
                    );
                }
                fs::create_dir_all(destination.application_directory()).unwrap();
                fs::write(
                    destination.application_directory().join("catalog.json"),
                    save_id,
                )
                .unwrap();
                Ok(Some(
                    workflow_ide_framework::project_save::ApplicationSaveResult {
                        data_version: Some("staged-v1".into()),
                    },
                ))
            }
            _ => unreachable!(),
        }
    }
}

fn save_fixture(
    name: &str,
) -> (
    std::path::PathBuf,
    ProjectContext,
    workflow_ide_framework::project_lifecycle::ProjectSession,
    workflow_ide_framework::project::ProjectFile,
) {
    use workflow_ide_framework::project::{ApplicationMetadata, ProjectFile, ProjectMetadata};
    use workflow_ide_framework::project_resource::NewProjectStoragePolicy;
    let (root, context) = setup(name);
    let session = workflow_ide_framework::project_lifecycle::ProjectSession::new_project(
        NewProjectStoragePolicy::Required,
        Some(context.clone()),
    )
    .unwrap();
    let project = ProjectFile {
        project: ProjectMetadata {
            format_version: 1,
            name: "Generation".into(),
            description: None,
            language: "en-US".into(),
            save_id: String::new(),
            saved_at: String::new(),
        },
        application: ApplicationMetadata {
            id: "org.example.test".into(),
            name: "Test".into(),
            description: None,
            language: "en-US".into(),
            data_version: None,
        },
    };
    (root, context, session, project)
}

fn save_identity<'a>(
    generation_id: &'a str,
    save_id: &'a str,
) -> workflow_ide_framework::project_save_generation::GenerationSaveIdentity<'a> {
    workflow_ide_framework::project_save_generation::GenerationSaveIdentity {
        generation_id,
        save_id,
        saved_at: "2026-10-22",
    }
}

#[test]
fn save_cons_staged_save_publishes_and_clears_dirty_only_after_success() {
    use workflow_ide_framework::project_save_generation::save_project_generation;
    let (root, context, mut session, mut project) = save_fixture("staged-success");
    let settings = workflow_ide_framework::framework_settings::FrameworkSettings::default();
    let mut app = StagedSaver {
        mode: "success",
        calls: 0,
    };
    save_project_generation(
        &mut session,
        &context,
        &mut project,
        &settings,
        &mut app,
        save_identity("generation-1", "save-1"),
    )
    .unwrap();
    assert_eq!(app.calls, 1);
    assert_eq!(
        read_committed_generation(&context)
            .unwrap()
            .unwrap()
            .generation_id,
        "generation-1"
    );
    assert_eq!(session.current_save_id.as_deref(), Some("save-1"));
    assert!(!session.dirty.is_dirty());
    assert_eq!(project.project.save_id, "save-1");
    assert_eq!(
        project.application.data_version.as_deref(),
        Some("staged-v1")
    );
    let stage = generation_path(&context, "generation-1").unwrap();
    assert_eq!(
        fs::read_to_string(stage.join("application/catalog.json")).unwrap(),
        "save-1"
    );
    assert!(!context.project_file_path().exists());
    assert!(!context.framework_settings_path().exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn save_cons_staged_save_failure_preserves_previous_commit_and_dirty_state() {
    use workflow_ide_framework::project_save_generation::{
        GenerationSaveError, save_project_generation,
    };
    let (root, context, mut session, mut project) = save_fixture("staged-failure");
    let settings = workflow_ide_framework::framework_settings::FrameworkSettings::default();
    let mut success = StagedSaver {
        mode: "success",
        calls: 0,
    };
    save_project_generation(
        &mut session,
        &context,
        &mut project,
        &settings,
        &mut success,
        save_identity("old", "save-old"),
    )
    .unwrap();
    session.mark_application_dirty();
    let original = project.clone();
    let mut failure = StagedSaver {
        mode: "failure",
        calls: 0,
    };
    assert!(matches!(
        save_project_generation(
            &mut session,
            &context,
            &mut project,
            &settings,
            &mut failure,
            save_identity("new", "save-new"),
        ),
        Err(GenerationSaveError::Application(_))
    ));
    assert_eq!(failure.calls, 1);
    assert_eq!(
        read_committed_generation(&context)
            .unwrap()
            .unwrap()
            .generation_id,
        "old"
    );
    assert_eq!(session.current_save_id.as_deref(), Some("save-old"));
    assert!(session.dirty.application);
    assert_eq!(project, original);
    assert!(!generation_path(&context, "new").unwrap().exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn save_cons_staged_save_unsupported_adapter_never_falls_back_to_legacy() {
    use workflow_ide_framework::project_save_generation::{
        GenerationSaveError, save_project_generation,
    };
    let (root, context, mut session, mut project) = save_fixture("staged-unsupported");
    let original = project.clone();
    let settings = workflow_ide_framework::framework_settings::FrameworkSettings::default();
    let mut app = StagedSaver {
        mode: "unsupported",
        calls: 0,
    };
    assert!(matches!(
        save_project_generation(
            &mut session,
            &context,
            &mut project,
            &settings,
            &mut app,
            save_identity("unsupported", "save-unsupported"),
        ),
        Err(GenerationSaveError::StagedSaveNotSupported)
    ));
    assert_eq!(app.calls, 1);
    assert!(read_committed_generation(&context).unwrap().is_none());
    assert!(session.dirty.is_dirty());
    assert!(session.current_save_id.is_none());
    assert_eq!(project, original);
    assert!(!context.project_file_path().exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn save_cons_validation_failure_keeps_previous_generation_and_cleans_stage() {
    use workflow_ide_framework::project_save_generation::{
        GenerationSaveError, save_project_generation, staging_path,
    };
    let (root, context, mut session, mut project) = save_fixture("validation-error");
    let settings = workflow_ide_framework::framework_settings::FrameworkSettings::default();
    let mut success = StagedSaver {
        mode: "success",
        calls: 0,
    };
    save_project_generation(
        &mut session,
        &context,
        &mut project,
        &settings,
        &mut success,
        save_identity("old", "save-old"),
    )
    .unwrap();
    session.mark_application_dirty();
    let previous = project.clone();
    let mut rejected = StagedSaver {
        mode: "validation-error",
        calls: 0,
    };
    assert!(matches!(
        save_project_generation(
            &mut session,
            &context,
            &mut project,
            &settings,
            &mut rejected,
            save_identity("new", "save-new"),
        ),
        Err(GenerationSaveError::Generation(
            GenerationError::InvalidCandidate(_)
        ))
    ));
    assert_eq!(rejected.calls, 1);
    assert_eq!(
        read_committed_generation(&context)
            .unwrap()
            .unwrap()
            .generation_id,
        "old"
    );
    assert_eq!(session.current_save_id.as_deref(), Some("save-old"));
    assert!(session.dirty.application);
    assert_eq!(project, previous);
    assert!(!staging_path(&context, "new").unwrap().exists());
    assert!(!generation_path(&context, "new").unwrap().exists());
    assert!(generation_path(&context, "old").unwrap().is_dir());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn save_cons_unsupported_validation_never_commits_and_cleans_stage() {
    use workflow_ide_framework::project_save_generation::{
        GenerationSaveError, save_project_generation, staging_path,
    };
    let (root, context, mut session, mut project) = save_fixture("validation-unsupported");
    let original = project.clone();
    let settings = workflow_ide_framework::framework_settings::FrameworkSettings::default();
    let mut app = StagedSaver {
        mode: "validation-unsupported",
        calls: 0,
    };
    assert!(matches!(
        save_project_generation(
            &mut session,
            &context,
            &mut project,
            &settings,
            &mut app,
            save_identity("candidate", "save-candidate"),
        ),
        Err(GenerationSaveError::Generation(
            GenerationError::InvalidCandidate(_)
        ))
    ));
    assert_eq!(app.calls, 1);
    assert!(read_committed_generation(&context).unwrap().is_none());
    assert!(session.current_save_id.is_none());
    assert!(session.dirty.is_dirty());
    assert_eq!(project, original);
    assert!(!staging_path(&context, "candidate").unwrap().exists());
    assert!(!generation_path(&context, "candidate").unwrap().exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn save_cons_resave_reads_previous_committed_application() {
    use workflow_ide_framework::project_save_generation::save_project_generation;
    let (root, context, mut session, mut project) = save_fixture("resave-source");
    let settings = workflow_ide_framework::framework_settings::FrameworkSettings::default();
    let mut first = StagedSaver {
        mode: "success",
        calls: 0,
    };
    save_project_generation(
        &mut session,
        &context,
        &mut project,
        &settings,
        &mut first,
        save_identity("old", "save-old"),
    )
    .unwrap();
    session.mark_application_dirty();
    let mut second = StagedSaver {
        mode: "resave",
        calls: 0,
    };
    save_project_generation(
        &mut session,
        &context,
        &mut project,
        &settings,
        &mut second,
        save_identity("new", "save-new"),
    )
    .unwrap();
    assert_eq!(second.calls, 1);
    assert_eq!(
        read_committed_generation(&context)
            .unwrap()
            .unwrap()
            .generation_id,
        "new"
    );
    assert_eq!(
        fs::read_to_string(
            generation_path(&context, "old")
                .unwrap()
                .join("application/catalog.json")
        )
        .unwrap(),
        "save-old"
    );
    assert_eq!(
        fs::read_to_string(
            generation_path(&context, "new")
                .unwrap()
                .join("application/catalog.json")
        )
        .unwrap(),
        "save-new"
    );
    assert!(!session.dirty.is_dirty());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn save_cons_corrupt_committed_source_blocks_resave_without_staging() {
    use workflow_ide_framework::project_save_generation::{
        GenerationSaveError, save_project_generation, staging_path,
    };
    let (root, context, mut session, mut project) = save_fixture("resave-corrupt");
    let settings = workflow_ide_framework::framework_settings::FrameworkSettings::default();
    let mut first = StagedSaver {
        mode: "success",
        calls: 0,
    };
    save_project_generation(
        &mut session,
        &context,
        &mut project,
        &settings,
        &mut first,
        save_identity("old", "save-old"),
    )
    .unwrap();
    session.mark_application_dirty();
    let previous = project.clone();
    fs::write(
        generation_path(&context, "old")
            .unwrap()
            .join("framework_settings.toml"),
        "invalid = [",
    )
    .unwrap();
    let mut second = StagedSaver {
        mode: "resave",
        calls: 0,
    };
    assert!(matches!(
        save_project_generation(
            &mut session,
            &context,
            &mut project,
            &settings,
            &mut second,
            save_identity("new", "save-new"),
        ),
        Err(GenerationSaveError::Generation(
            GenerationError::InvalidCandidate(_)
        ))
    ));
    assert_eq!(second.calls, 0);
    assert_eq!(
        read_committed_generation(&context)
            .unwrap()
            .unwrap()
            .generation_id,
        "old"
    );
    assert_eq!(session.current_save_id.as_deref(), Some("save-old"));
    assert!(session.dirty.application);
    assert_eq!(project, previous);
    assert!(!staging_path(&context, "new").unwrap().exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn save_cons_resave_rejects_unhealthy_or_unsupported_committed_application() {
    use workflow_ide_framework::project_save_generation::{
        GenerationSaveError, save_project_generation, staging_path,
    };
    for mode in [
        "inspect-unsupported",
        "inspect-error",
        "inspect-incompatible",
        "inspect-inconsistent",
    ] {
        let (root, context, mut session, mut project) = save_fixture(mode);
        let settings = workflow_ide_framework::framework_settings::FrameworkSettings::default();
        let mut first = StagedSaver {
            mode: "success",
            calls: 0,
        };
        save_project_generation(
            &mut session,
            &context,
            &mut project,
            &settings,
            &mut first,
            save_identity("old", "save-old"),
        )
        .unwrap();
        session.mark_application_dirty();
        let original = project.clone();
        let mut second = StagedSaver { mode, calls: 0 };
        assert!(
            matches!(
                save_project_generation(
                    &mut session,
                    &context,
                    &mut project,
                    &settings,
                    &mut second,
                    save_identity("new", "save-new"),
                ),
                Err(GenerationSaveError::Generation(
                    GenerationError::InvalidCandidate(_)
                )) | Err(GenerationSaveError::Application(_))
            ),
            "mode: {mode}"
        );
        assert_eq!(second.calls, 0, "mode: {mode}");
        assert_eq!(
            read_committed_generation(&context)
                .unwrap()
                .unwrap()
                .generation_id,
            "old"
        );
        assert_eq!(session.current_save_id.as_deref(), Some("save-old"));
        assert!(session.dirty.application);
        assert_eq!(project, original);
        assert!(!staging_path(&context, "new").unwrap().exists());
        assert!(generation_path(&context, "old").unwrap().is_dir());
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn save_cons_storage_format_detection_is_pointer_based_and_fails_closed() {
    use workflow_ide_framework::project_save_generation::{
        ProjectStorageFormat, committed_path, detect_project_storage_format,
        save_project_generation,
    };
    let (root, context, mut session, mut project) = save_fixture("format-detection");
    assert!(matches!(
        detect_project_storage_format(&context),
        Ok(ProjectStorageFormat::Legacy)
    ));

    let settings = workflow_ide_framework::framework_settings::FrameworkSettings::default();
    let mut application = StagedSaver {
        mode: "success",
        calls: 0,
    };
    save_project_generation(
        &mut session,
        &context,
        &mut project,
        &settings,
        &mut application,
        save_identity("format-old", "format-save"),
    )
    .unwrap();
    assert!(matches!(
        detect_project_storage_format(&context),
        Ok(ProjectStorageFormat::Generation)
    ));

    let committed = committed_path(&context);
    let original_pointer = fs::read_to_string(&committed).unwrap();
    fs::write(&committed, "not valid toml = [").unwrap();
    assert!(detect_project_storage_format(&context).is_err());

    fs::write(&committed, &original_pointer).unwrap();
    fs::remove_file(
        generation_path(&context, "format-old")
            .unwrap()
            .join("project.toml"),
    )
    .unwrap();
    assert!(detect_project_storage_format(&context).is_err());

    fs::remove_dir_all(root).unwrap();
}

fn controller_generation_fixture(
    name: &str,
) -> (
    std::path::PathBuf,
    ProjectContext,
    workflow_ide_framework::project_controller::ProjectController,
) {
    use workflow_ide_framework::{
        framework_settings::FrameworkSettings,
        project_controller::{ProjectCommandResult, ProjectController},
        project_save_generation::save_project_generation,
    };
    let (root, context, mut session, mut project) = save_fixture(name);
    let mut app = StagedSaver {
        mode: "success",
        calls: 0,
    };
    save_project_generation(
        &mut session,
        &context,
        &mut project,
        &FrameworkSettings::default(),
        &mut app,
        save_identity("original", "save-old"),
    )
    .unwrap();
    let mut controller = ProjectController::new("org.example.test", "Test");
    assert_eq!(
        controller.open(context.clone(), &mut app),
        ProjectCommandResult::Completed
    );
    (root, context, controller)
}

#[test]
fn controller_generation_save_round_trip_reopens_latest_snapshot() {
    use workflow_ide_framework::project_controller::{ProjectCommandResult, ProjectController};
    let (root, context, mut controller) = controller_generation_fixture("controller-roundtrip");
    controller.mark_unsaved();
    let mut app = StagedSaver {
        mode: "resave",
        calls: 0,
    };
    assert_eq!(
        controller.save(&mut app, "save-new", "2026-10-24"),
        ProjectCommandResult::Completed
    );
    assert_eq!(app.calls, 1);
    assert!(!controller.has_unsaved_changes());
    assert_eq!(
        controller
            .session
            .as_ref()
            .unwrap()
            .current_save_id
            .as_deref(),
        Some("save-new")
    );
    assert_eq!(
        controller.project_file.as_ref().unwrap().project.save_id,
        "save-new"
    );
    let pointer = read_committed_generation(&context).unwrap().unwrap();
    assert_ne!(pointer.generation_id, "original");
    assert_eq!(
        fs::read_to_string(
            generation_path(&context, &pointer.generation_id)
                .unwrap()
                .join("application/catalog.json")
        )
        .unwrap(),
        "save-new"
    );
    assert!(!context.project_file_path().exists());
    let mut reopened = ProjectController::new("org.example.test", "Test");
    assert_eq!(
        reopened.open(context.clone(), &mut app),
        ProjectCommandResult::Completed
    );
    assert_eq!(
        reopened.project_file.as_ref().unwrap().project.save_id,
        "save-new"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn controller_generation_save_failure_preserves_pointer_and_dirty_state() {
    use workflow_ide_framework::project_controller::ProjectCommandResult;
    let (root, context, mut controller) = controller_generation_fixture("controller-failure");
    controller.mark_unsaved();
    let mut app = StagedSaver {
        mode: "failure",
        calls: 0,
    };
    assert!(matches!(
        controller.save(&mut app, "save-new", "2026-10-24"),
        ProjectCommandResult::Failed(_)
    ));
    assert_eq!(app.calls, 1);
    assert!(controller.has_unsaved_changes());
    assert_eq!(
        controller
            .session
            .as_ref()
            .unwrap()
            .current_save_id
            .as_deref(),
        Some("save-old")
    );
    assert_eq!(
        controller.project_file.as_ref().unwrap().project.save_id,
        "save-old"
    );
    assert_eq!(
        read_committed_generation(&context)
            .unwrap()
            .unwrap()
            .generation_id,
        "original"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn controller_generation_save_unsupported_never_falls_back_to_legacy() {
    use workflow_ide_framework::project_controller::ProjectCommandResult;
    let (root, context, mut controller) = controller_generation_fixture("controller-unsupported");
    controller.mark_unsaved();
    let mut app = StagedSaver {
        mode: "unsupported",
        calls: 0,
    };
    assert!(matches!(
        controller.save(&mut app, "save-new", "2026-10-24"),
        ProjectCommandResult::Failed(_)
    ));
    assert_eq!(app.calls, 1);
    assert!(controller.has_unsaved_changes());
    assert_eq!(
        read_committed_generation(&context)
            .unwrap()
            .unwrap()
            .generation_id,
        "original"
    );
    assert!(!context.project_file_path().exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn controller_generation_save_as_commits_copy_and_switches_session() {
    use workflow_ide_framework::project_controller::ProjectCommandResult;
    let (root, source, mut controller) = controller_generation_fixture("save-as-copy");
    let destination = ProjectContext::new(root.join("copy"));
    controller.mark_unsaved();
    let mut app = StagedSaver {
        mode: "resave",
        calls: 0,
    };
    assert_eq!(
        controller.save_as(destination.clone(), "Copy", &mut app, "copy-save", "now"),
        ProjectCommandResult::Completed
    );
    assert_eq!(app.calls, 1);
    assert_eq!(
        controller.session.as_ref().unwrap().context(),
        Some(&destination)
    );
    assert_eq!(controller.project_name(), Some("Copy"));
    assert!(!controller.has_unsaved_changes());
    assert_eq!(
        read_committed_generation(&source)
            .unwrap()
            .unwrap()
            .generation_id,
        "original"
    );
    let pointer = read_committed_generation(&destination).unwrap().unwrap();
    assert_eq!(
        fs::read_to_string(
            generation_path(&destination, &pointer.generation_id)
                .unwrap()
                .join("application/catalog.json")
        )
        .unwrap(),
        "copy-save"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn controller_generation_save_as_failure_preserves_source_and_dirty_state() {
    use workflow_ide_framework::project_controller::ProjectCommandResult;
    let (root, source, mut controller) = controller_generation_fixture("save-as-failure");
    let destination = ProjectContext::new(root.join("copy"));
    let original_name = controller.project_name().map(str::to_owned);
    controller.mark_unsaved();
    let mut app = StagedSaver {
        mode: "failure",
        calls: 0,
    };
    assert!(matches!(
        controller.save_as(destination.clone(), "Copy", &mut app, "copy-save", "now"),
        ProjectCommandResult::Failed(_)
    ));
    assert_eq!(app.calls, 1);
    assert_eq!(
        controller.session.as_ref().unwrap().context(),
        Some(&source)
    );
    assert_eq!(controller.project_name(), original_name.as_deref());
    assert_eq!(
        controller
            .session
            .as_ref()
            .unwrap()
            .current_save_id
            .as_deref(),
        Some("save-old")
    );
    assert!(controller.has_unsaved_changes());
    assert_eq!(
        read_committed_generation(&source)
            .unwrap()
            .unwrap()
            .generation_id,
        "original"
    );
    assert!(read_committed_generation(&destination).unwrap().is_none());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn controller_generation_save_as_refuses_occupied_destination() {
    use workflow_ide_framework::project_controller::ProjectCommandResult;
    let (root, source, mut controller) = controller_generation_fixture("save-as-occupied");
    let destination = ProjectContext::new(root.join("occupied"));
    fs::create_dir_all(destination.root()).unwrap();
    fs::write(destination.root().join("keep.txt"), "keep").unwrap();
    let mut app = StagedSaver {
        mode: "resave",
        calls: 0,
    };
    assert!(matches!(
        controller.save_as(destination.clone(), "Copy", &mut app, "copy-save", "now"),
        ProjectCommandResult::Failed(_)
    ));
    assert_eq!(app.calls, 0);
    assert_eq!(
        fs::read_to_string(destination.root().join("keep.txt")).unwrap(),
        "keep"
    );
    assert_eq!(
        controller.session.as_ref().unwrap().context(),
        Some(&source)
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn controller_generation_save_as_preserves_project_resource_registry() {
    use workflow_ide_framework::{
        framework_settings::{FrameworkSettings, StoredResourceScope},
        project_controller::ProjectCommandResult,
        project_resource::{ResourceReference, ResourceScope},
    };
    let (root, source, mut controller) = controller_generation_fixture("save-as-project-registry");
    let destination = ProjectContext::new(root.join("copy"));
    let id = controller
        .register_resource(
            ResourceReference::new(ResourceScope::Project, "resources/robot.urdf").unwrap(),
        )
        .unwrap();
    let mut app = StagedSaver {
        mode: "resave",
        calls: 0,
    };
    assert_eq!(
        controller.save_as(destination.clone(), "Copy", &mut app, "copy-save", "now"),
        ProjectCommandResult::Completed
    );
    let pointer = read_committed_generation(&destination).unwrap().unwrap();
    let settings = FrameworkSettings::from_toml(
        &fs::read_to_string(
            generation_path(&destination, &pointer.generation_id)
                .unwrap()
                .join("framework_settings.toml"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(settings.resources.len(), 1);
    assert_eq!(settings.resources[0].resource_id, id);
    assert_eq!(settings.resources[0].scope, StoredResourceScope::Project);
    assert_eq!(
        settings.resources[0].path,
        std::path::PathBuf::from("resources/robot.urdf")
    );
    assert_eq!(
        read_committed_generation(&source)
            .unwrap()
            .unwrap()
            .generation_id,
        "original"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn controller_generation_save_as_preserves_external_resource_reference() {
    use workflow_ide_framework::{
        framework_settings::{FrameworkSettings, StoredResourceScope},
        project_controller::ProjectCommandResult,
        project_resource::{ResourceReference, ResourceScope},
    };
    let (root, _source, mut controller) = controller_generation_fixture("save-as-external-registry");
    let destination = ProjectContext::new(root.join("copy"));
    let external = root.join("external").join("model.urdf");
    let id = controller
        .register_resource(ResourceReference::new(ResourceScope::External, &external).unwrap())
        .unwrap();
    let mut app = StagedSaver {
        mode: "resave",
        calls: 0,
    };
    assert_eq!(
        controller.save_as(destination.clone(), "Copy", &mut app, "copy-save", "now"),
        ProjectCommandResult::Completed
    );
    let pointer = read_committed_generation(&destination).unwrap().unwrap();
    let settings = FrameworkSettings::from_toml(
        &fs::read_to_string(
            generation_path(&destination, &pointer.generation_id)
                .unwrap()
                .join("framework_settings.toml"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(settings.resources.len(), 1);
    assert_eq!(settings.resources[0].resource_id, id);
    assert_eq!(settings.resources[0].scope, StoredResourceScope::External);
    assert_eq!(settings.resources[0].path, external);
    fs::remove_dir_all(root).unwrap();
}
