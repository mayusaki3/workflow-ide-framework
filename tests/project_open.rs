use std::{fs, path::PathBuf};
use workflow_ide_framework::{
    framework_settings::FrameworkSettings,
    project::{ApplicationMetadata, ProjectContext, ProjectFile, ProjectMetadata, PROJECT_FORMAT_VERSION},
    project_open::{open_project, open_requires_user_decision, ApplicationProjectInspector, FrameworkSettingsOpen},
    project_resource::{ProjectDataCompatibility, ProjectDataConsistency},
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
    fn inspect_project_data(&mut self, _: &ProjectContext, version: Option<&str>) -> Result<ProjectDataCompatibility, Self::Error> {
        self.seen_version = version.map(str::to_owned);
        Ok(self.compatibility.clone())
    }
    fn check_project_consistency(&mut self, _: &ProjectContext) -> Result<ProjectDataConsistency, Self::Error> {
        Ok(self.consistency.clone())
    }
}

#[test]
fn open_loads_framework_settings_and_passes_stored_version_to_application() {
    let r = root("ok"); let context = ProjectContext::new(&r); write_project(&context, Some("4"));
    fs::create_dir_all(context.framework_directory()).unwrap();
    fs::write(context.framework_settings_path(), FrameworkSettings::default().to_toml().unwrap()).unwrap();
    let mut app = Inspector { seen_version: None, compatibility: ProjectDataCompatibility::Compatible, consistency: ProjectDataConsistency::Consistent };
    let result = open_project(&context, "org.example.open", &mut app).unwrap();
    assert!(matches!(result.framework_settings, FrameworkSettingsOpen::Loaded(_)));
    assert_eq!(app.seen_version.as_deref(), Some("4"));
    assert!(!open_requires_user_decision(&result));
    let _=fs::remove_dir_all(r);
}

#[test]
fn missing_framework_settings_returns_defaults_decision_instead_of_blocking_open() {
    let r=root("missing"); let context=ProjectContext::new(&r); write_project(&context,None);
    let mut app=Inspector { seen_version:None, compatibility:ProjectDataCompatibility::Compatible, consistency:ProjectDataConsistency::Consistent };
    let result=open_project(&context,"org.example.open",&mut app).unwrap();
    assert_eq!(result.framework_settings,FrameworkSettingsOpen::MissingUseDefaults);
    assert!(open_requires_user_decision(&result));
    let _=fs::remove_dir_all(r);
}

#[test]
fn corrupt_framework_settings_is_recoverable_result_not_project_open_error() {
    let r=root("corrupt"); let context=ProjectContext::new(&r); write_project(&context,None);
    fs::create_dir_all(context.framework_directory()).unwrap();
    fs::write(context.framework_settings_path(),"not = [valid").unwrap();
    let mut app=Inspector { seen_version:None, compatibility:ProjectDataCompatibility::Compatible, consistency:ProjectDataConsistency::Consistent };
    let result=open_project(&context,"org.example.open",&mut app).unwrap();
    assert!(matches!(result.framework_settings,FrameworkSettingsOpen::InvalidUseDefaults{..}));
    let _=fs::remove_dir_all(r);
}

#[test]
fn incompatible_application_is_returned_for_ui_decision() {
    let r=root("incompatible"); let context=ProjectContext::new(&r); write_project(&context,Some("1"));
    let mut app=Inspector {
        seen_version:None,
        compatibility:ProjectDataCompatibility::Incompatible { reason:Some("newer data".into()), handled:false },
        consistency:ProjectDataConsistency::Consistent,
    };
    let result=open_project(&context,"org.example.open",&mut app).unwrap();
    assert!(open_requires_user_decision(&result));
    assert!(matches!(result.application_compatibility,ProjectDataCompatibility::Incompatible{..}));
    let _=fs::remove_dir_all(r);
}

#[test]
fn inconsistent_application_state_is_returned_without_automatic_recovery() {
    let r=root("inconsistent"); let context=ProjectContext::new(&r); write_project(&context,None);
    let mut app=Inspector {
        seen_version:None,
        compatibility:ProjectDataCompatibility::Compatible,
        consistency:ProjectDataConsistency::Inconsistent { reason:Some("save mismatch".into()), can_open:true, can_recover:true, handled:false },
    };
    let result=open_project(&context,"org.example.open",&mut app).unwrap();
    assert!(matches!(result.application_consistency,ProjectDataConsistency::Inconsistent{can_recover:true,..}));
    assert!(open_requires_user_decision(&result));
    let _=fs::remove_dir_all(r);
}

#[test]
fn framework_save_id_mismatch_requires_user_decision() {
    let r=root("save-id-mismatch"); let context=ProjectContext::new(&r); write_project(&context,None);
    fs::create_dir_all(context.framework_directory()).unwrap();
    let mut settings=FrameworkSettings::default(); settings.save_id=Some("different-save".into());
    fs::write(context.framework_settings_path(),settings.to_toml().unwrap()).unwrap();
    let mut app=Inspector { seen_version:None, compatibility:ProjectDataCompatibility::Compatible, consistency:ProjectDataConsistency::Consistent };
    let result=open_project(&context,"org.example.open",&mut app).unwrap();
    assert!(matches!(result.framework_settings,FrameworkSettingsOpen::SaveIdMismatch{ref project_save_id,ref framework_save_id}
        if project_save_id=="s1" && framework_save_id=="different-save"));
    assert!(open_requires_user_decision(&result));
    let _=fs::remove_dir_all(r);
}

#[test]
fn legacy_framework_settings_without_save_id_remains_openable() {
    let r=root("legacy-no-save-id"); let context=ProjectContext::new(&r); write_project(&context,None);
    fs::create_dir_all(context.framework_directory()).unwrap();
    fs::write(context.framework_settings_path(),"format_version = 1\nresources = []\n").unwrap();
    let mut app=Inspector { seen_version:None, compatibility:ProjectDataCompatibility::Compatible, consistency:ProjectDataConsistency::Consistent };
    let result=open_project(&context,"org.example.open",&mut app).unwrap();
    assert!(matches!(result.framework_settings,FrameworkSettingsOpen::Loaded(ref settings) if settings.save_id.is_none()));
    assert!(!open_requires_user_decision(&result));
    let _=fs::remove_dir_all(r);
}
