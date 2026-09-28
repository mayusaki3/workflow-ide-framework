use std::{fs, path::PathBuf};
use workflow_ide_framework::{
    framework_settings::{FrameworkSettings, StoredResourceEntry, StoredResourceScope},
    project::{ApplicationMetadata, ProjectContext, ProjectFile, ProjectMetadata, PROJECT_FORMAT_VERSION},
    project_lifecycle::ProjectSession,
    project_save::{save_project, ApplicationProjectSaver, ApplicationSaveResult},
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

struct Saver { fail: bool, calls: usize }
impl ApplicationProjectSaver for Saver {
    type Error = &'static str;
    fn save_project_data(&mut self, context: &ProjectContext, save_id: &str) -> Result<ApplicationSaveResult, Self::Error> {
        self.calls += 1;
        if self.fail { return Err("application save failed"); }
        fs::create_dir_all(context.application_directory()).map_err(|_| "mkdir")?;
        fs::write(context.application_directory().join("data.txt"), save_id).map_err(|_| "write")?;
        Ok(ApplicationSaveResult { data_version: Some("7".into()) })
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
    let mut saver = Saver { fail: false, calls: 0 };

    save_project(&mut session, &context, &mut file, &settings, &mut saver, "save-42", "2026-09-27T13:18:00+09:00").unwrap();

    assert_eq!(saver.calls, 1);
    assert!(!session.dirty.is_dirty());
    assert_eq!(session.current_save_id.as_deref(), Some("save-42"));
    assert!(context.framework_settings_path().is_file());
    assert!(context.project_file_path().is_file());
    assert_eq!(fs::read_to_string(context.application_directory().join("data.txt")).unwrap(), "save-42");

    let stored = ProjectFile::from_toml(&fs::read_to_string(context.project_file_path()).unwrap(), "org.example.test").unwrap();
    assert_eq!(stored.project.save_id, "save-42");
    let stored_framework=FrameworkSettings::from_toml(&fs::read_to_string(context.framework_settings_path()).unwrap()).unwrap();
    assert_eq!(stored_framework.save_id.as_deref(),Some("save-42"));
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
    let mut saver = Saver { fail: true, calls: 0 };

    assert!(save_project(&mut session, &context, &mut file, &FrameworkSettings::default(), &mut saver, "failed-save", "now").is_err());
    assert!(session.dirty.is_dirty());
    assert!(!context.framework_settings_path().exists());
    assert!(!context.project_file_path().exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn failed_project_file_write_does_not_mutate_in_memory_project_metadata() {
    let root=temp_root("save-project-fail");let context=ProjectContext::new(&root);
    fs::create_dir_all(context.project_file_path()).unwrap();
    let mut file=project_file();let original=file.clone();let mut session=ProjectSession::default();session.mark_metadata_dirty();
    let mut saver=Saver{fail:false,calls:0};
    assert!(save_project(&mut session,&context,&mut file,&FrameworkSettings::default(),&mut saver,"new-save","new-time").is_err());
    assert_eq!(file,original);assert!(session.dirty.is_dirty());
    let _=fs::remove_dir_all(root);
}
