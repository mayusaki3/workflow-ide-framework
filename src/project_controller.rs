use std::collections::BTreeMap;
use crate::{
    framework_settings::FrameworkSettings,
    workspace::{WorkspaceRegistry, WorkspaceError},
    workspace_dock::{WorkspaceDockProjection, snapshot_workspace_layout, restore_workspace_layout},
    project::{
        ApplicationMetadata, PROJECT_FORMAT_VERSION, ProjectContext, ProjectFile, ProjectMetadata,
    },
    project_adapter_erased::ErasedApplicationProjectAdapter,
    project_lifecycle::ProjectSession,
    project_open::{
        FrameworkSettingsOpen, ProjectOpenResult, open_project, open_requires_user_decision,
        open_session,
    },
    project_resource::NewProjectStoragePolicy,
    project_save::save_project,
    project_save_as::save_project_as,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectCommandResult {
    Completed,
    NeedsSaveLocation,
    NeedsOpenDecision(ProjectOpenResult),
    NeedsDirtyConfirmation,
    Failed(String),
}

pub struct ProjectController {
    application_id: String,
    application_name: String,
    pub session: Option<ProjectSession>,
    pub project_file: Option<ProjectFile>,
    pub framework_settings: FrameworkSettings,
}

impl ProjectController {
    pub fn new(application_id: impl Into<String>, application_name: impl Into<String>) -> Self {
        Self {
            application_id: application_id.into(),
            application_name: application_name.into(),
            session: None,
            project_file: None,
            framework_settings: FrameworkSettings::default(),
        }
    }

    pub fn new_project(
        &mut self,
        name: impl Into<String>,
        language: impl Into<String>,
        description: Option<String>,
        policy: NewProjectStoragePolicy,
        root: Option<ProjectContext>,
        application: &mut dyn ErasedApplicationProjectAdapter,
    ) -> ProjectCommandResult {
        let name = name.into();
        let language = language.into();
        let session = match ProjectSession::new_project(policy, root) {
            Ok(session) => session,
            Err(_) => return ProjectCommandResult::NeedsSaveLocation,
        };
        if let Some(context) = session.context() {
            if let Err(error) = application.initialize_project(context) {
                return ProjectCommandResult::Failed(error);
            }
        }
        self.project_file = Some(self.project_template(name, language, description));
        self.framework_settings = FrameworkSettings::default();
        self.session = Some(session);
        ProjectCommandResult::Completed
    }

    pub fn open(
        &mut self,
        context: ProjectContext,
        application: &mut dyn ErasedApplicationProjectAdapter,
    ) -> ProjectCommandResult {
        let result = match open_project(&context, &self.application_id, application) {
            Ok(result) => result,
            Err(error) => return ProjectCommandResult::Failed(format!("{error:?}")),
        };
        if open_requires_user_decision(&result) {
            return ProjectCommandResult::NeedsOpenDecision(result);
        }
        self.accept_open(context, result)
    }

    pub fn accept_open(
        &mut self,
        context: ProjectContext,
        result: ProjectOpenResult,
    ) -> ProjectCommandResult {
        let session = match open_session(context, &result) {
            Ok(session) => session,
            Err(error) => return ProjectCommandResult::Failed(format!("{error:?}")),
        };
        self.framework_settings = match &result.framework_settings {
            FrameworkSettingsOpen::Loaded(settings) => settings.clone(),
            _ => FrameworkSettings::default(),
        };
        self.project_file = Some(result.project_file);
        self.session = Some(session);
        ProjectCommandResult::Completed
    }

    /// Capture the live layout of every workspace before saving the project.
    /// The settings map is replaced only when every snapshot succeeds.
    pub fn capture_workspace_layouts(
        &mut self,
        registry: &WorkspaceRegistry,
        projections: &BTreeMap<String, WorkspaceDockProjection>,
    ) -> Result<(), WorkspaceError> {
        let mut layouts = BTreeMap::new();
        for workspace in registry.iter() {
            let projection = projections.get(&workspace.id)
                .ok_or(WorkspaceError::InvalidContainerTarget)?;
            layouts.insert(
                workspace.id.clone(),
                snapshot_workspace_layout(workspace, projection)?,
            );
        }
        self.framework_settings.workspace_layouts = layouts;
        self.framework_settings.selected_workspace_id = Some(registry.selected_id().to_owned());
        Ok(())
    }

    /// Restore layouts from the opened project's framework settings.
    /// Workspaces without stored layout keep their application-defined defaults.
    /// The caller must register panel instances before invoking this method.
    pub fn restore_project_workspace_layouts(
        &self,
        registry: &mut WorkspaceRegistry,
    ) -> Result<BTreeMap<String, WorkspaceDockProjection>, WorkspaceError> {
        let mut restored = BTreeMap::new();
        // Validate all snapshots on a detached registry before touching the live registry.
        // The live registry is changed only after every workspace succeeds.
        let mut candidate = registry.clone();
        // UI-created workspaces are project-owned and may not exist in a fresh host.
        // Recreate them on the detached registry before validating their layouts.
        // Application-owned workspaces must still be registered by the application.
        for id in self.framework_settings.workspace_layouts.keys() {
            if candidate.get(id).is_none() && id.starts_with("user.workspace.") {
                let ordinal = id.strip_prefix("user.workspace.").unwrap_or_default();
                if ordinal.is_empty() || !ordinal.chars().all(|ch| ch.is_ascii_digit()) {
                    return Err(WorkspaceError::WorkspaceNotFound(id.clone()));
                }
                candidate.register(id.clone(), format!("Workspace {ordinal}"))?;
            }
        }
        for (id, layout) in &self.framework_settings.workspace_layouts {
            let projection = restore_workspace_layout(&mut candidate, id, layout)?;
            restored.insert(id.clone(), projection);
        }
        if let Some(id) = self.framework_settings.selected_workspace_id.as_deref() {
            if candidate.get(id).is_some() {
                candidate.select(id)?;
            }
        }
        *registry = candidate;
        Ok(restored)
    }

    pub fn save(
        &mut self,
        application: &mut dyn ErasedApplicationProjectAdapter,
        save_id: &str,
        saved_at: &str,
    ) -> ProjectCommandResult {
        let Some(session) = self.session.as_mut() else {
            return ProjectCommandResult::Failed("no project is open".into());
        };
        let Some(context) = session.context().cloned() else {
            return ProjectCommandResult::NeedsSaveLocation;
        };
        let Some(project_file) = self.project_file.as_mut() else {
            return ProjectCommandResult::Failed("project metadata is unavailable".into());
        };
        match save_project(
            session,
            &context,
            project_file,
            &self.framework_settings,
            application,
            save_id,
            saved_at,
        ) {
            Ok(()) => ProjectCommandResult::Completed,
            Err(error) => ProjectCommandResult::Failed(format!("{error:?}")),
        }
    }

    pub fn save_as(
        &mut self,
        destination: ProjectContext,
        project_name: impl Into<String>,
        application: &mut dyn ErasedApplicationProjectAdapter,
        save_id: &str,
        saved_at: &str,
    ) -> ProjectCommandResult {
        let Some(session) = self.session.as_mut() else {
            return ProjectCommandResult::Failed("no project is open".into());
        };
        let Some(project_file) = self.project_file.as_mut() else {
            return ProjectCommandResult::Failed("project metadata is unavailable".into());
        };
        let previous_name = std::mem::replace(&mut project_file.project.name, project_name.into());
        let result = save_project_as(
            session,
            destination,
            project_file,
            &self.framework_settings,
            application,
            save_id,
            saved_at,
        );
        if result.is_err() {
            project_file.project.name = previous_name;
        }
        match result {
            Ok(()) => ProjectCommandResult::Completed,
            Err(error) => ProjectCommandResult::Failed(format!("{error:?}")),
        }
    }

    pub fn close(&mut self, confirmed_dirty: bool) -> ProjectCommandResult {
        if self
            .session
            .as_ref()
            .is_some_and(|session| session.dirty.is_dirty())
            && !confirmed_dirty
        {
            return ProjectCommandResult::NeedsDirtyConfirmation;
        }
        self.session = None;
        self.project_file = None;
        self.framework_settings = FrameworkSettings::default();
        ProjectCommandResult::Completed
    }

    pub fn is_open(&self) -> bool {
        self.session.is_some()
    }

    pub fn project_name(&self) -> Option<&str> {
        self.project_file
            .as_ref()
            .map(|file| file.project.name.as_str())
    }

    pub fn project_description(&self) -> Option<&str> {
        self.project_file
            .as_ref()
            .and_then(|file| file.project.description.as_deref())
    }

    pub fn update_project_metadata(
        &mut self,
        name: impl Into<String>,
        description: Option<String>,
    ) -> ProjectCommandResult {
        let Some(project_file) = self.project_file.as_mut() else {
            return ProjectCommandResult::Failed("no project is open".into());
        };
        let name = name.into();
        if name.trim().is_empty() {
            return ProjectCommandResult::Failed("project name is required".into());
        }
        project_file.project.name = name.trim().to_owned();
        project_file.project.description = description.and_then(|value| {
            let value = value.trim().to_owned();
            (!value.is_empty()).then_some(value)
        });
        if let Some(session) = self.session.as_mut() {
            session.mark_metadata_dirty();
        }
        ProjectCommandResult::Completed
    }

    fn project_template(
        &self,
        name: String,
        language: String,
        description: Option<String>,
    ) -> ProjectFile {
        ProjectFile {
            project: ProjectMetadata {
                format_version: PROJECT_FORMAT_VERSION,
                name,
                description,
                language: language.clone(),
                save_id: String::new(),
                saved_at: String::new(),
            },
            application: ApplicationMetadata {
                id: self.application_id.clone(),
                name: self.application_name.clone(),
                description: None,
                language,
                data_version: None,
            },
        }
    }
}
