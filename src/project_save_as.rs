use crate::{
    framework_settings::FrameworkSettings,
    project::{ProjectContext, ProjectFile},
    project_lifecycle::{ProjectSession, ProjectStorageState},
    project_save::{save_project, ApplicationSaveResult, ProjectSaveError},
};
use std::fs;

pub trait ApplicationProjectSaveAs {
    type Error: std::fmt::Display;
    fn save_project_data_as(
        &mut self,
        source: Option<&ProjectContext>,
        destination: &ProjectContext,
        save_id: &str,
    ) -> Result<ApplicationSaveResult, Self::Error>;
}

pub struct SaveAsAdapter<'a, A> {
    application: &'a mut A,
    source: Option<&'a ProjectContext>,
}
impl<A: ApplicationProjectSaveAs> crate::project_save::ApplicationProjectSaver for SaveAsAdapter<'_, A> {
    type Error = A::Error;
    fn save_project_data(&mut self, destination: &ProjectContext, save_id: &str) -> Result<ApplicationSaveResult, Self::Error> {
        self.application.save_project_data_as(self.source, destination, save_id)
    }
}

pub fn save_project_as<A: ApplicationProjectSaveAs>(
    session: &mut ProjectSession,
    destination: ProjectContext,
    project_file: &mut ProjectFile,
    framework_settings: &FrameworkSettings,
    application: &mut A,
    save_id: &str,
    saved_at: &str,
) -> Result<(), ProjectSaveError> {
    let source = session.context().cloned();

    let mut candidate = ProjectSession {
        dirty: session.dirty,
        current_save_id: session.current_save_id.clone(),
        storage: ProjectStorageState::Stored(destination.clone()),
    };
    let mut adapter = SaveAsAdapter { application, source: source.as_ref() };

    let app = adapter.application
        .save_project_data_as(adapter.source, &destination, save_id)
        .map_err(|error| ProjectSaveError::Application(error.to_string()))?;

    if let Some(source_context) = &source {
        let source_resources = source_context.resource_root();
        if source_resources.is_dir() {
            copy_directory(&source_resources, &destination.resource_root())
                .map_err(ProjectSaveError::FrameworkWrite)?;
        }
    }

    struct CompletedSaveAs(ApplicationSaveResult);
    impl crate::project_save::ApplicationProjectSaver for CompletedSaveAs {
        type Error = std::convert::Infallible;
        fn save_project_data(&mut self, _context: &ProjectContext, _save_id: &str) -> Result<ApplicationSaveResult, Self::Error> {
            Ok(self.0.clone())
        }
    }
    let mut completed = CompletedSaveAs(app);
    save_project(&mut candidate, &destination, project_file, framework_settings, &mut completed, save_id, saved_at)?;
    *session = candidate;
    Ok(())
}

fn copy_directory(source: &std::path::Path, destination: &std::path::Path) -> std::io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_directory(&source_path, &destination_path)?;
        } else {
            fs::copy(source_path, destination_path)?;
        }
    }
    Ok(())
}
