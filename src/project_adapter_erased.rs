use crate::{
    project::ProjectContext,
    project_adapter::ApplicationProjectAdapter,
    project_resource::{ProjectDataCompatibility, ProjectDataConsistency},
    project_save::ApplicationSaveResult,
};

/// Object-safe form used by the Framework host while preserving the typed
/// ApplicationProjectAdapter as the Consumer-facing implementation contract.
pub trait ErasedApplicationProjectAdapter {
    fn initialize_project(&mut self, context: &ProjectContext) -> Result<(), String>;
    fn inspect_project_data(
        &mut self,
        context: &ProjectContext,
        stored_data_version: Option<&str>,
    ) -> Result<ProjectDataCompatibility, String>;
    fn check_project_consistency(
        &mut self,
        context: &ProjectContext,
    ) -> Result<ProjectDataConsistency, String>;
    fn save_project_data(
        &mut self,
        context: &ProjectContext,
        save_id: &str,
    ) -> Result<ApplicationSaveResult, String>;
    fn save_project_data_as(
        &mut self,
        source: Option<&ProjectContext>,
        destination: &ProjectContext,
        save_id: &str,
    ) -> Result<ApplicationSaveResult, String>;
}

impl<A: ApplicationProjectAdapter> ErasedApplicationProjectAdapter for A {
    fn initialize_project(&mut self, context: &ProjectContext) -> Result<(), String> {
        ApplicationProjectAdapter::initialize_project(self, context).map_err(|e| e.to_string())
    }

    fn inspect_project_data(
        &mut self,
        context: &ProjectContext,
        stored_data_version: Option<&str>,
    ) -> Result<ProjectDataCompatibility, String> {
        ApplicationProjectAdapter::inspect_project_data(self, context, stored_data_version)
            .map_err(|e| e.to_string())
    }

    fn check_project_consistency(
        &mut self,
        context: &ProjectContext,
    ) -> Result<ProjectDataConsistency, String> {
        ApplicationProjectAdapter::check_project_consistency(self, context).map_err(|e| e.to_string())
    }

    fn save_project_data(
        &mut self,
        context: &ProjectContext,
        save_id: &str,
    ) -> Result<ApplicationSaveResult, String> {
        ApplicationProjectAdapter::save_project_data(self, context, save_id).map_err(|e| e.to_string())
    }

    fn save_project_data_as(
        &mut self,
        source: Option<&ProjectContext>,
        destination: &ProjectContext,
        save_id: &str,
    ) -> Result<ApplicationSaveResult, String> {
        ApplicationProjectAdapter::save_project_data_as(self, source, destination, save_id)
            .map_err(|e| e.to_string())
    }
}

impl crate::project_open::ApplicationProjectInspector for dyn ErasedApplicationProjectAdapter + '_ {
    type Error = String;
    fn inspect_project_data(&mut self, context: &ProjectContext, stored_data_version: Option<&str>) -> Result<ProjectDataCompatibility, Self::Error> {
        ErasedApplicationProjectAdapter::inspect_project_data(self, context, stored_data_version)
    }
    fn check_project_consistency(&mut self, context: &ProjectContext) -> Result<ProjectDataConsistency, Self::Error> {
        ErasedApplicationProjectAdapter::check_project_consistency(self, context)
    }
}

impl crate::project_save::ApplicationProjectSaver for dyn ErasedApplicationProjectAdapter + '_ {
    type Error = String;
    fn save_project_data(&mut self, context: &ProjectContext, save_id: &str) -> Result<ApplicationSaveResult, Self::Error> {
        ErasedApplicationProjectAdapter::save_project_data(self, context, save_id)
    }
}

impl crate::project_save_as::ApplicationProjectSaveAs for dyn ErasedApplicationProjectAdapter + '_ {
    type Error = String;
    fn save_project_data_as(&mut self, source: Option<&ProjectContext>, destination: &ProjectContext, save_id: &str) -> Result<ApplicationSaveResult, Self::Error> {
        ErasedApplicationProjectAdapter::save_project_data_as(self, source, destination, save_id)
    }
}
