use crate::{
    project::ProjectContext,
    project_resource::{NewProjectStoragePolicy, ProjectDirtyState},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectStorageState {
    Unsaved,
    Stored(ProjectContext),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSession {
    pub dirty: ProjectDirtyState,
    pub current_save_id: Option<String>,
    pub storage: ProjectStorageState,
}

impl Default for ProjectSession {
    fn default() -> Self {
        Self {
            dirty: ProjectDirtyState::default(),
            current_save_id: None,
            storage: ProjectStorageState::Unsaved,
        }
    }
}

impl ProjectSession {
    pub fn new_project(
        policy: NewProjectStoragePolicy,
        root: Option<ProjectContext>,
    ) -> Result<Self, NewProjectError> {
        let storage = match (policy, root) {
            (NewProjectStoragePolicy::Deferred, None) => ProjectStorageState::Unsaved,
            (_, Some(context)) => ProjectStorageState::Stored(context),
            (NewProjectStoragePolicy::Required, None) => {
                return Err(NewProjectError::StorageRequired);
            }
        };
        Ok(Self {
            dirty: ProjectDirtyState {
                metadata: true,
                framework: true,
                application: true,
            },
            current_save_id: None,
            storage,
        })
    }

    pub fn mark_metadata_dirty(&mut self) {
        self.dirty.metadata = true;
    }
    pub fn mark_framework_dirty(&mut self) {
        self.dirty.framework = true;
    }
    pub fn mark_application_dirty(&mut self) {
        self.dirty.application = true;
    }

    pub fn context(&self) -> Option<&ProjectContext> {
        match &self.storage {
            ProjectStorageState::Unsaved => None,
            ProjectStorageState::Stored(context) => Some(context),
        }
    }

    pub fn set_storage(&mut self, context: ProjectContext) {
        self.storage = ProjectStorageState::Stored(context);
    }

    pub fn begin_save(&self, save_id: impl Into<String>) -> PendingProjectSave {
        PendingProjectSave {
            save_id: save_id.into(),
            application_saved: false,
            framework_saved: false,
            project_file_saved: false,
        }
    }

    pub fn complete_save(&mut self, pending: PendingProjectSave) -> Result<(), SaveStateError> {
        if !pending.application_saved {
            return Err(SaveStateError::ApplicationNotSaved);
        }
        if !pending.framework_saved {
            return Err(SaveStateError::FrameworkNotSaved);
        }
        if !pending.project_file_saved {
            return Err(SaveStateError::ProjectFileNotSaved);
        }
        self.current_save_id = Some(pending.save_id);
        self.dirty = ProjectDirtyState::default();
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewProjectError {
    StorageRequired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingProjectSave {
    pub save_id: String,
    application_saved: bool,
    framework_saved: bool,
    project_file_saved: bool,
}

impl PendingProjectSave {
    pub fn application_saved(mut self) -> Self {
        self.application_saved = true;
        self
    }
    pub fn framework_saved(mut self) -> Self {
        self.framework_saved = true;
        self
    }
    pub fn project_file_saved(mut self) -> Self {
        self.project_file_saved = true;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveStateError {
    ApplicationNotSaved,
    FrameworkNotSaved,
    ProjectFileNotSaved,
}
