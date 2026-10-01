use workflow_ide_framework::{
    project_controller::{ProjectCommandResult, ProjectController},
    project_resource::NewProjectStoragePolicy,
    project_adapter::ApplicationProjectAdapter,
    project::ProjectContext,
    project_resource::{ProjectDataCompatibility, ProjectDataConsistency},
    project_save::ApplicationSaveResult,
};

#[derive(Default)]
struct App;
impl ApplicationProjectAdapter for App {
    type Error = &'static str;
    fn initialize_project(&mut self, _context: &ProjectContext) -> Result<(), Self::Error> { Ok(()) }
    fn inspect_project_data(&mut self, _context: &ProjectContext, _version: Option<&str>) -> Result<ProjectDataCompatibility, Self::Error> { Ok(ProjectDataCompatibility::Compatible) }
    fn check_project_consistency(&mut self, _context: &ProjectContext) -> Result<ProjectDataConsistency, Self::Error> { Ok(ProjectDataConsistency::Consistent) }
    fn save_project_data(&mut self, _context: &ProjectContext, _save_id: &str) -> Result<ApplicationSaveResult, Self::Error> { Ok(ApplicationSaveResult { data_version: Some("1".into()) }) }
    fn save_project_data_as(&mut self, _source: Option<&ProjectContext>, _destination: &ProjectContext, _save_id: &str) -> Result<ApplicationSaveResult, Self::Error> { Ok(ApplicationSaveResult { data_version: Some("1".into()) }) }
}

#[test]
fn deferred_new_routes_save_to_save_as_and_close_requires_confirmation() {
    let mut controller = ProjectController::new("org.test", "Test");
    let mut app = App;
    assert_eq!(controller.new_project("Untitled", "en", None, NewProjectStoragePolicy::Deferred, None, &mut app), ProjectCommandResult::Completed);
    assert!(controller.is_open());
    assert_eq!(controller.save(&mut app, "save-1", "2026-09-29T00:00:00Z"), ProjectCommandResult::NeedsSaveLocation);
    assert_eq!(controller.close(false), ProjectCommandResult::NeedsDirtyConfirmation);
    assert!(controller.is_open());
    assert_eq!(controller.close(true), ProjectCommandResult::Completed);
    assert!(!controller.is_open());
}

#[test]
fn required_new_without_root_requests_location() {
    let mut controller = ProjectController::new("org.test", "Test");
    let mut app = App;
    assert_eq!(controller.new_project("Untitled", "en", None, NewProjectStoragePolicy::Required, None, &mut app), ProjectCommandResult::NeedsSaveLocation);
    assert!(!controller.is_open());
}

#[test]
fn new_project_keeps_name_description_and_language_metadata() {
    let mut controller = ProjectController::new("org.test", "Test");
    let mut app = App;
    assert_eq!(
        controller.new_project("Demo", "ja-JP", Some("説明".into()), NewProjectStoragePolicy::Deferred, None, &mut app),
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
    let mut adapter = TestAdapter::default();
    assert_eq!(
        controller.new_project("Before", "en-US", None, NewProjectStoragePolicy::Deferred, None, &mut adapter),
        ProjectCommandResult::Completed
    );
    controller.session.as_mut().unwrap().dirty = Default::default();

    assert_eq!(
        controller.update_project_metadata("After", Some(" Project description ".into())),
        ProjectCommandResult::Completed
    );
    assert_eq!(controller.project_name(), Some("After"));
    assert_eq!(controller.project_description(), Some("Project description"));
    assert!(controller.session.as_ref().unwrap().dirty.metadata);
}
