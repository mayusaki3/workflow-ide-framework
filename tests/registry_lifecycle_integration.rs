use std::fs;
use workflow_ide_framework::{
    project::ProjectContext,
    project_adapter::ApplicationProjectAdapter,
    project_controller::{ProjectCommandResult, ProjectController},
    project_resource::{
        NewProjectStoragePolicy, ProjectDataCompatibility, ProjectDataConsistency,
        ResourceReference, ResourceScope,
    },
    project_save::ApplicationSaveResult,
};

#[derive(Default)]
struct App;
impl ApplicationProjectAdapter for App {
    type Error = &'static str;
    fn initialize_project(&mut self, _context: &ProjectContext) -> Result<(), Self::Error> {
        Ok(())
    }
    fn inspect_project_data(
        &mut self,
        _context: &ProjectContext,
        _version: Option<&str>,
    ) -> Result<ProjectDataCompatibility, Self::Error> {
        Ok(ProjectDataCompatibility::Compatible)
    }
    fn check_project_consistency(
        &mut self,
        _context: &ProjectContext,
    ) -> Result<ProjectDataConsistency, Self::Error> {
        Ok(ProjectDataConsistency::Consistent)
    }
    fn save_project_data(
        &mut self,
        _context: &ProjectContext,
        _save_id: &str,
    ) -> Result<ApplicationSaveResult, Self::Error> {
        Ok(ApplicationSaveResult {
            data_version: Some("1".into()),
        })
    }
    fn save_project_data_as(
        &mut self,
        _source: Option<&ProjectContext>,
        _destination: &ProjectContext,
        _save_id: &str,
    ) -> Result<ApplicationSaveResult, Self::Error> {
        Ok(ApplicationSaveResult {
            data_version: Some("1".into()),
        })
    }
}

fn context(label: &str) -> ProjectContext {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let path =
        std::env::temp_dir().join(format!("wfide-reg-life-{label}-{}-{n}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
    ProjectContext::new(path)
}

fn reference(path: &str) -> ResourceReference {
    ResourceReference::new(ResourceScope::Project, path).unwrap()
}

fn new_project(controller: &mut ProjectController, app: &mut App, context: ProjectContext) {
    assert_eq!(
        controller.new_project(
            "Registry",
            "ja-JP",
            None,
            NewProjectStoragePolicy::Required,
            Some(context),
            app
        ),
        ProjectCommandResult::Completed
    );
}

#[test]
fn reg_life_001_new_register_save_close_open_restores_stable_id() {
    let root = context("save-open");
    let mut app = App;
    let mut controller = ProjectController::new("org.test.registry", "Registry");
    new_project(&mut controller, &mut app, root.clone());
    let id = controller
        .register_resource(reference("models/robot.urdf"))
        .unwrap();
    assert_eq!(
        controller.save(&mut app, "reg-life-001", "2026-10-18T00:00:00Z"),
        ProjectCommandResult::Completed
    );
    assert!(!controller.has_unsaved_changes());
    assert_eq!(controller.close(false), ProjectCommandResult::Completed);
    assert!(controller.resource_registry().is_none());
    assert_eq!(
        controller.open(root.clone(), &mut app),
        ProjectCommandResult::Completed
    );
    assert_eq!(
        controller
            .resource_registry()
            .unwrap()
            .find_by_reference(&reference("models/robot.urdf"))
            .unwrap()
            .resource_id,
        id
    );
    assert!(!controller.has_unsaved_changes());
    let _ = fs::remove_dir_all(root.root());
}

#[test]
fn reg_life_002_save_as_preserves_stable_id_and_restores_it() {
    let source = context("source");
    let destination = context("destination");
    let mut app = App;
    let mut controller = ProjectController::new("org.test.registry", "Registry");
    new_project(&mut controller, &mut app, source.clone());
    let id = controller
        .register_resource(reference("robot.urdf"))
        .unwrap();
    assert_eq!(
        controller.save(&mut app, "reg-life-002a", "2026-10-18T00:00:00Z"),
        ProjectCommandResult::Completed
    );
    assert_eq!(
        controller.save_as(
            destination.clone(),
            "Registry Copy",
            &mut app,
            "reg-life-002b",
            "2026-10-18T00:01:00Z"
        ),
        ProjectCommandResult::Completed
    );
    assert_eq!(
        controller
            .resource_registry()
            .unwrap()
            .find_by_id(&id)
            .unwrap()
            .reference,
        reference("robot.urdf")
    );
    assert_eq!(controller.close(false), ProjectCommandResult::Completed);
    assert_eq!(
        controller.open(destination.clone(), &mut app),
        ProjectCommandResult::Completed
    );
    assert_eq!(
        controller
            .resource_registry()
            .unwrap()
            .find_by_reference(&reference("robot.urdf"))
            .unwrap()
            .resource_id,
        id
    );
    let _ = fs::remove_dir_all(source.root());
    let _ = fs::remove_dir_all(destination.root());
}

#[test]
fn reg_life_003_new_project_does_not_inherit_previous_registry() {
    let first = context("first");
    let second = context("second");
    let mut app = App;
    let mut controller = ProjectController::new("org.test.registry", "Registry");
    new_project(&mut controller, &mut app, first.clone());
    let first_id = controller
        .register_resource(reference("first.urdf"))
        .unwrap();
    assert_eq!(controller.close(true), ProjectCommandResult::Completed);
    new_project(&mut controller, &mut app, second.clone());
    assert!(controller.resource_registry().unwrap().entries().is_empty());
    let second_id = controller
        .register_resource(reference("second.urdf"))
        .unwrap();
    assert_ne!(first_id, second_id);
    assert!(
        controller
            .resource_registry()
            .unwrap()
            .find_by_id(&first_id)
            .is_none()
    );
    let _ = fs::remove_dir_all(first.root());
    let _ = fs::remove_dir_all(second.root());
}
