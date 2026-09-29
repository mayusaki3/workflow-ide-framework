use crate::{
    project::ProjectContext,
    project_open::ApplicationProjectInspector,
    project_resource::{ProjectDataCompatibility, ProjectDataConsistency},
    project_save::{ApplicationProjectSaver, ApplicationSaveResult},
    project_save_as::ApplicationProjectSaveAs,
};

/// Unified Consumer/Application boundary for Project lifecycle operations.
///
/// The Framework owns Project lifecycle and persistence ordering, while the
/// Application owns the meaning and storage format of data under application/.
pub trait ApplicationProjectAdapter {
    type Error: std::fmt::Display;

    fn initialize_project(&mut self, context: &ProjectContext) -> Result<(), Self::Error>;

    fn inspect_project_data(
        &mut self,
        context: &ProjectContext,
        stored_data_version: Option<&str>,
    ) -> Result<ProjectDataCompatibility, Self::Error>;

    fn check_project_consistency(
        &mut self,
        context: &ProjectContext,
    ) -> Result<ProjectDataConsistency, Self::Error>;

    fn save_project_data(
        &mut self,
        context: &ProjectContext,
        save_id: &str,
    ) -> Result<ApplicationSaveResult, Self::Error>;

    fn save_project_data_as(
        &mut self,
        source: Option<&ProjectContext>,
        destination: &ProjectContext,
        save_id: &str,
    ) -> Result<ApplicationSaveResult, Self::Error>;
}

/// Adapts the unified public lifecycle boundary to the focused internal
/// Save/Open/Save As traits without duplicating Application logic.
pub struct ProjectAdapterBridge<'a, A> {
    application: &'a mut A,
}

impl<'a, A> ProjectAdapterBridge<'a, A> {
    pub fn new(application: &'a mut A) -> Self {
        Self { application }
    }

    pub fn initialize_project(&mut self, context: &ProjectContext) -> Result<(), A::Error>
    where
        A: ApplicationProjectAdapter,
    {
        self.application.initialize_project(context)
    }
}

impl<A: ApplicationProjectAdapter> ApplicationProjectInspector for ProjectAdapterBridge<'_, A> {
    type Error = A::Error;

    fn inspect_project_data(
        &mut self,
        context: &ProjectContext,
        stored_data_version: Option<&str>,
    ) -> Result<ProjectDataCompatibility, Self::Error> {
        self.application.inspect_project_data(context, stored_data_version)
    }

    fn check_project_consistency(
        &mut self,
        context: &ProjectContext,
    ) -> Result<ProjectDataConsistency, Self::Error> {
        self.application.check_project_consistency(context)
    }
}

impl<A: ApplicationProjectAdapter> ApplicationProjectSaver for ProjectAdapterBridge<'_, A> {
    type Error = A::Error;

    fn save_project_data(
        &mut self,
        context: &ProjectContext,
        save_id: &str,
    ) -> Result<ApplicationSaveResult, Self::Error> {
        self.application.save_project_data(context, save_id)
    }
}

impl<A: ApplicationProjectAdapter> ApplicationProjectSaveAs for ProjectAdapterBridge<'_, A> {
    type Error = A::Error;

    fn save_project_data_as(
        &mut self,
        source: Option<&ProjectContext>,
        destination: &ProjectContext,
        save_id: &str,
    ) -> Result<ApplicationSaveResult, Self::Error> {
        self.application
            .save_project_data_as(source, destination, save_id)
    }
}
