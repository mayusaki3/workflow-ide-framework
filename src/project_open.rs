use crate::{
    framework_settings::FrameworkSettings,
    project::{ProjectContext, ProjectFile, ProjectFileError},
    project_lifecycle::{ProjectSession, ProjectStorageState},
    project_resource::{ProjectDataCompatibility, ProjectDataConsistency},
    resource_journal::ResourceOperationJournal,
};
use std::{fs, io};

pub trait ApplicationProjectInspector {
    type Error: std::fmt::Display;

    fn inspect_project_data(
        &mut self,
        context: &ProjectContext,
        stored_data_version: Option<&str>,
    ) -> Result<ProjectDataCompatibility, Self::Error>;

    fn check_project_consistency(
        &mut self,
        context: &ProjectContext,
    ) -> Result<ProjectDataConsistency, Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameworkSettingsOpen {
    Loaded(FrameworkSettings),
    MissingUseDefaults,
    InvalidUseDefaults {
        reason: String,
    },
    SaveIdMismatch {
        project_save_id: String,
        framework_save_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectOpenResult {
    pub project_file: ProjectFile,
    pub framework_settings: FrameworkSettingsOpen,
    pub pending_resource_operation: PendingResourceOperationOpen,
    pub application_compatibility: ProjectDataCompatibility,
    pub application_consistency: ProjectDataConsistency,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingResourceOperationOpen {
    None,
    Pending(ResourceOperationJournal),
    Invalid { reason: String },
}

#[derive(Debug)]
pub enum ProjectOpenError {
    ReadProject(io::Error),
    ProjectFile(ProjectFileError),
    ReadFramework(io::Error),
    Application(String),
}

pub fn open_project<A: ApplicationProjectInspector + ?Sized>(
    context: &ProjectContext,
    expected_application_id: &str,
    application: &mut A,
) -> Result<ProjectOpenResult, ProjectOpenError> {
    let project_text =
        fs::read_to_string(context.project_file_path()).map_err(ProjectOpenError::ReadProject)?;
    let project_file = ProjectFile::from_toml(&project_text, expected_application_id)
        .map_err(ProjectOpenError::ProjectFile)?;

    let framework_settings = match fs::read_to_string(context.framework_settings_path()) {
        Ok(text) => match FrameworkSettings::from_toml(&text) {
            Ok(settings) => match settings.save_id.as_deref() {
                Some(framework_save_id) if framework_save_id != project_file.project.save_id => {
                    FrameworkSettingsOpen::SaveIdMismatch {
                        project_save_id: project_file.project.save_id.clone(),
                        framework_save_id: framework_save_id.to_owned(),
                    }
                }
                _ => FrameworkSettingsOpen::Loaded(settings),
            },
            Err(error) => FrameworkSettingsOpen::InvalidUseDefaults {
                reason: format!("{error:?}"),
            },
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            FrameworkSettingsOpen::MissingUseDefaults
        }
        Err(error) => return Err(ProjectOpenError::ReadFramework(error)),
    };

    let pending_resource_operation = match ResourceOperationJournal::load(context) {
        Ok(Some(journal)) => PendingResourceOperationOpen::Pending(journal),
        Ok(None) => PendingResourceOperationOpen::None,
        Err(error) => PendingResourceOperationOpen::Invalid {
            reason: error.to_string(),
        },
    };

    let application_compatibility = application
        .inspect_project_data(context, project_file.application.data_version.as_deref())
        .map_err(|error| ProjectOpenError::Application(error.to_string()))?;

    let application_consistency = application
        .check_project_consistency(context)
        .map_err(|error| ProjectOpenError::Application(error.to_string()))?;

    Ok(ProjectOpenResult {
        project_file,
        framework_settings,
        pending_resource_operation,
        application_compatibility,
        application_consistency,
    })
}

pub fn open_requires_user_decision(result: &ProjectOpenResult) -> bool {
    !matches!(result.framework_settings, FrameworkSettingsOpen::Loaded(_))
        || !matches!(
            result.pending_resource_operation,
            PendingResourceOperationOpen::None
        )
        || matches!(
            result.application_compatibility,
            ProjectDataCompatibility::Converted { .. }
                | ProjectDataCompatibility::Incompatible { .. }
        )
        || matches!(
            result.application_consistency,
            ProjectDataConsistency::Inconsistent { .. }
        )
}

pub fn open_can_continue(result: &ProjectOpenResult) -> bool {
    !matches!(
        result.application_compatibility,
        ProjectDataCompatibility::Incompatible { .. }
    ) && !matches!(
        result.application_consistency,
        ProjectDataConsistency::Inconsistent {
            can_open: false,
            ..
        }
    )
}

pub fn open_dirty_state(result: &ProjectOpenResult) -> crate::project_resource::ProjectDirtyState {
    crate::project_resource::ProjectDirtyState {
        metadata: false,
        framework: !matches!(result.framework_settings, FrameworkSettingsOpen::Loaded(_)),
        application: matches!(
            result.application_compatibility,
            ProjectDataCompatibility::Converted { .. }
        ),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectOpenSessionError {
    CannotContinue,
}

pub fn open_session(
    context: ProjectContext,
    result: &ProjectOpenResult,
) -> Result<ProjectSession, ProjectOpenSessionError> {
    if !open_can_continue(result) {
        return Err(ProjectOpenSessionError::CannotContinue);
    }
    Ok(ProjectSession {
        dirty: open_dirty_state(result),
        current_save_id: Some(result.project_file.project.save_id.clone()),
        storage: ProjectStorageState::Stored(context),
    })
}
