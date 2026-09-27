use crate::{
    framework_settings::FrameworkSettings,
    project::{ProjectContext, ProjectFile, ProjectFileError},
    project_io::atomic_write,
    project_lifecycle::ProjectSession,
};
use std::io;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationSaveResult {
    pub data_version: Option<String>,
}

pub trait ApplicationProjectSaver {
    type Error: std::fmt::Display;

    fn save_project_data(
        &mut self,
        context: &ProjectContext,
        save_id: &str,
    ) -> Result<ApplicationSaveResult, Self::Error>;
}

#[derive(Debug)]
pub enum ProjectSaveError {
    Application(String),
    FrameworkSerialize(String),
    FrameworkWrite(io::Error),
    ProjectSerialize(ProjectFileError),
    ProjectWrite(io::Error),
}

pub fn save_project<A: ApplicationProjectSaver>(
    session: &mut ProjectSession,
    context: &ProjectContext,
    project_file: &mut ProjectFile,
    framework_settings: &FrameworkSettings,
    application: &mut A,
    save_id: &str,
    saved_at: &str,
) -> Result<(), ProjectSaveError> {
    let mut pending = session.begin_save(save_id);

    let app = application
        .save_project_data(context, save_id)
        .map_err(|error| ProjectSaveError::Application(error.to_string()))?;
    pending = pending.application_saved();

    let framework_text = framework_settings
        .to_toml()
        .map_err(|error| ProjectSaveError::FrameworkSerialize(format!("{error:?}")))?;
    atomic_write(context.framework_settings_path().as_path(), framework_text.as_bytes())
        .map_err(ProjectSaveError::FrameworkWrite)?;
    pending = pending.framework_saved();

    project_file.project.save_id = save_id.to_owned();
    project_file.project.saved_at = saved_at.to_owned();
    project_file.application.data_version = app.data_version;
    let project_text = project_file.to_toml().map_err(ProjectSaveError::ProjectSerialize)?;
    atomic_write(context.project_file_path().as_path(), project_text.as_bytes())
        .map_err(ProjectSaveError::ProjectWrite)?;
    pending = pending.project_file_saved();

    session.complete_save(pending).expect("all save stages were completed");
    Ok(())
}
