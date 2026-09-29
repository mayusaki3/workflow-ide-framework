use workflow_ide_framework::{
    project::ProjectContext,
    project_adapter::{ApplicationProjectAdapter, ProjectAdapterBridge},
    project_open::ApplicationProjectInspector,
    project_resource::{ProjectDataCompatibility, ProjectDataConsistency},
    project_save::{ApplicationProjectSaver, ApplicationSaveResult},
    project_save_as::ApplicationProjectSaveAs,
};

#[derive(Default)]
struct App {
    initialized: bool,
    inspected: bool,
    consistency_checked: bool,
    saved: bool,
    saved_as: bool,
}

impl ApplicationProjectAdapter for App {
    type Error = &'static str;

    fn initialize_project(&mut self, _context: &ProjectContext) -> Result<(), Self::Error> {
        self.initialized = true;
        Ok(())
    }

    fn inspect_project_data(
        &mut self,
        _context: &ProjectContext,
        _stored_data_version: Option<&str>,
    ) -> Result<ProjectDataCompatibility, Self::Error> {
        self.inspected = true;
        Ok(ProjectDataCompatibility::Compatible)
    }

    fn check_project_consistency(
        &mut self,
        _context: &ProjectContext,
    ) -> Result<ProjectDataConsistency, Self::Error> {
        self.consistency_checked = true;
        Ok(ProjectDataConsistency::Consistent)
    }

    fn save_project_data(
        &mut self,
        _context: &ProjectContext,
        _save_id: &str,
    ) -> Result<ApplicationSaveResult, Self::Error> {
        self.saved = true;
        Ok(ApplicationSaveResult { data_version: Some("1".into()) })
    }

    fn save_project_data_as(
        &mut self,
        _source: Option<&ProjectContext>,
        _destination: &ProjectContext,
        _save_id: &str,
    ) -> Result<ApplicationSaveResult, Self::Error> {
        self.saved_as = true;
        Ok(ApplicationSaveResult { data_version: Some("1".into()) })
    }
}

#[test]
fn unified_adapter_bridges_all_project_lifecycle_callbacks() {
    let context = ProjectContext::new(std::env::temp_dir().join("wfide-project-adapter"));
    let mut app = App::default();

    {
        let mut bridge = ProjectAdapterBridge::new(&mut app);
        bridge.initialize_project(&context).unwrap();
        ApplicationProjectInspector::inspect_project_data(&mut bridge, &context, None).unwrap();
        ApplicationProjectInspector::check_project_consistency(&mut bridge, &context).unwrap();
        ApplicationProjectSaver::save_project_data(&mut bridge, &context, "save-1").unwrap();
        ApplicationProjectSaveAs::save_project_data_as(
            &mut bridge,
            Some(&context),
            &context,
            "save-2",
        )
        .unwrap();
    }

    assert!(app.initialized);
    assert!(app.inspected);
    assert!(app.consistency_checked);
    assert!(app.saved);
    assert!(app.saved_as);
}

#[test]
fn application_builder_accepts_unified_project_adapter() {
    let _application = workflow_ide_framework::Application::new(
        "org.workflow-ide-framework.adapter-test",
        "Adapter Test",
    )
    .project_adapter(App::default());
}
