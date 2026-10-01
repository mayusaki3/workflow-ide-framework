use workflow_ide_framework::{
    Application,
    project::ProjectContext,
    project_adapter::ApplicationProjectAdapter,
    project_resource::{ProjectDataCompatibility, ProjectDataConsistency},
    project_save::ApplicationSaveResult,
};

#[derive(Default)]
struct MinimumProjectAdapter;

impl ApplicationProjectAdapter for MinimumProjectAdapter {
    type Error = std::io::Error;

    fn initialize_project(&mut self, context: &ProjectContext) -> Result<(), Self::Error> {
        std::fs::create_dir_all(context.application_directory())
    }

    fn inspect_project_data(&mut self, _context: &ProjectContext, _stored_data_version: Option<&str>) -> Result<ProjectDataCompatibility, Self::Error> {
        Ok(ProjectDataCompatibility::Compatible)
    }

    fn check_project_consistency(&mut self, _context: &ProjectContext) -> Result<ProjectDataConsistency, Self::Error> {
        Ok(ProjectDataConsistency::Consistent)
    }

    fn save_project_data(&mut self, context: &ProjectContext, _save_id: &str) -> Result<ApplicationSaveResult, Self::Error> {
        std::fs::create_dir_all(context.application_directory())?;
        Ok(ApplicationSaveResult { data_version: None })
    }

    fn save_project_data_as(&mut self, _source: Option<&ProjectContext>, destination: &ProjectContext, save_id: &str) -> Result<ApplicationSaveResult, Self::Error> {
        self.save_project_data(destination, save_id)
    }
}

fn main() -> eframe::Result<()> {
    Application::new(
        "org.workflow-ide-framework.sample-minimum",
        "Workflow IDE Framework Minimum Sample",
    )
    .version(env!("CARGO_PKG_VERSION"))
    .project_adapter(MinimumProjectAdapter)
    // Intentionally no About or Project Properties renderer:
    // Framework fallback screens are the behavior under test.
    .run()
}
