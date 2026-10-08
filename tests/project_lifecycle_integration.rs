use std::{fs, path::PathBuf};
use workflow_ide_framework::{
    framework_settings::FrameworkSettings,
    project::{
        ApplicationMetadata, PROJECT_FORMAT_VERSION, ProjectContext, ProjectFile, ProjectMetadata,
    },
    project_lifecycle::ProjectSession,
    project_resource::NewProjectStoragePolicy,
    project_save::{ApplicationProjectSaver, ApplicationSaveResult, save_project},
    project_save_as::{ApplicationProjectSaveAs, save_project_as},
};

fn root(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("wfide-lifecycle-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

fn metadata() -> ProjectFile {
    ProjectFile {
        project: ProjectMetadata {
            format_version: PROJECT_FORMAT_VERSION,
            name: "Lifecycle".into(),
            description: None,
            language: "en-US".into(),
            save_id: String::new(),
            saved_at: String::new(),
        },
        application: ApplicationMetadata {
            id: "org.example.lifecycle".into(),
            name: "Lifecycle".into(),
            description: None,
            language: "en-US".into(),
            data_version: None,
        },
    }
}

struct FirstSaver;
impl ApplicationProjectSaver for FirstSaver {
    type Error = &'static str;
    fn save_project_data(
        &mut self,
        context: &ProjectContext,
        save_id: &str,
    ) -> Result<ApplicationSaveResult, Self::Error> {
        fs::create_dir_all(context.application_directory()).map_err(|_| "mkdir")?;
        fs::write(context.application_directory().join("data"), save_id).map_err(|_| "write")?;
        Ok(ApplicationSaveResult {
            data_version: Some("1".into()),
        })
    }
}

struct AsSaver {
    fail: bool,
    source_seen: Option<PathBuf>,
}
impl ApplicationProjectSaveAs for AsSaver {
    type Error = &'static str;
    fn save_project_data_as(
        &mut self,
        source: Option<&ProjectContext>,
        destination: &ProjectContext,
        save_id: &str,
    ) -> Result<ApplicationSaveResult, Self::Error> {
        self.source_seen = source.map(|s| s.root.clone());
        if self.fail {
            return Err("save as failed");
        }
        fs::create_dir_all(destination.application_directory()).map_err(|_| "mkdir")?;
        fs::write(destination.application_directory().join("data"), save_id)
            .map_err(|_| "write")?;
        Ok(ApplicationSaveResult {
            data_version: Some("2".into()),
        })
    }
}

#[test]
fn deferred_first_save_creates_formal_project_after_root_is_selected() {
    let destination = ProjectContext::new(root("first-save"));
    let mut session = ProjectSession::new_project(NewProjectStoragePolicy::Deferred, None).unwrap();
    assert!(session.context().is_none());

    session.set_storage(destination.clone());
    let mut file = metadata();
    save_project(
        &mut session,
        &destination,
        &mut file,
        &FrameworkSettings::default(),
        &mut FirstSaver,
        "first-save-id",
        "now",
    )
    .unwrap();

    assert_eq!(session.context(), Some(&destination));
    assert!(!session.dirty.is_dirty());
    assert!(destination.project_file_path().is_file());
    assert!(destination.framework_settings_path().is_file());
    assert!(destination.application_directory().join("data").is_file());
    let _ = fs::remove_dir_all(destination.root());
}

#[test]
fn save_as_success_copies_project_resources_and_switches_current_root() {
    let source = ProjectContext::new(root("as-source"));
    let destination = ProjectContext::new(root("as-destination"));
    fs::create_dir_all(source.resource_root().join("textures")).unwrap();
    fs::write(
        source.resource_root().join("textures").join("robot.png"),
        b"resource",
    )
    .unwrap();

    let mut session =
        ProjectSession::new_project(NewProjectStoragePolicy::Required, Some(source.clone()))
            .unwrap();
    let mut file = metadata();
    let mut app = AsSaver {
        fail: false,
        source_seen: None,
    };
    save_project_as(
        &mut session,
        destination.clone(),
        &mut file,
        &FrameworkSettings::default(),
        &mut app,
        "as-id",
        "now",
    )
    .unwrap();

    assert_eq!(app.source_seen.as_deref(), Some(source.root()));
    assert_eq!(session.context(), Some(&destination));
    assert!(
        destination
            .resource_root()
            .join("textures")
            .join("robot.png")
            .is_file()
    );
    assert!(destination.project_file_path().is_file());
    assert!(!session.dirty.is_dirty());
    let _ = fs::remove_dir_all(source.root());
    let _ = fs::remove_dir_all(destination.root());
}

#[test]
fn save_as_application_failure_keeps_original_project_current() {
    let source = ProjectContext::new(root("as-fail-source"));
    let destination = ProjectContext::new(root("as-fail-destination"));
    fs::create_dir_all(source.resource_root()).unwrap();
    fs::write(
        source.resource_root().join("must-not-copy.txt"),
        b"resource",
    )
    .unwrap();
    let mut session =
        ProjectSession::new_project(NewProjectStoragePolicy::Required, Some(source.clone()))
            .unwrap();
    let original_dirty = session.dirty;
    let mut file = metadata();
    let mut app = AsSaver {
        fail: true,
        source_seen: None,
    };

    assert!(
        save_project_as(
            &mut session,
            destination.clone(),
            &mut file,
            &FrameworkSettings::default(),
            &mut app,
            "bad-id",
            "now"
        )
        .is_err()
    );
    assert_eq!(session.context(), Some(&source));
    assert_eq!(session.dirty, original_dirty);
    assert!(!destination.project_file_path().exists());
    assert!(
        !destination
            .resource_root()
            .join("must-not-copy.txt")
            .exists()
    );
    let _ = fs::remove_dir_all(source.root());
    let _ = fs::remove_dir_all(destination.root());
}
