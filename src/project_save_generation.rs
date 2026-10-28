//! Save-generation commit pointer. This module is independent of the legacy save path.
//! A generation is visible only after its complete directory is promoted and
//! its commit pointer is atomically replaced.
use crate::{
    framework_settings::{FRAMEWORK_SETTINGS_FORMAT_VERSION, FrameworkSettings},
    project::{PROJECT_FORMAT_VERSION, ProjectContext, ProjectFile},
    project_io::atomic_write,
};
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommittedGeneration {
    pub format_version: u32,
    pub generation_id: String,
}

#[derive(Debug)]
pub enum GenerationError {
    InvalidId,
    AlreadyExists,
    MissingCandidate,
    IncompleteCandidate,
    InvalidCommit(String),
    InvalidCandidate(String),
    MissingCommittedGeneration,
    Io(io::Error),
}

impl From<io::Error> for GenerationError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

fn validate_id(id: &str) -> Result<(), GenerationError> {
    if id.is_empty()
        || id.len() > 128
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(GenerationError::InvalidId);
    }
    Ok(())
}

pub fn generations_root(context: &ProjectContext) -> PathBuf {
    context.framework_directory().join("save-generations")
}

pub fn staging_path(context: &ProjectContext, id: &str) -> Result<PathBuf, GenerationError> {
    validate_id(id)?;
    Ok(generations_root(context).join("staging").join(id))
}

pub fn generation_path(context: &ProjectContext, id: &str) -> Result<PathBuf, GenerationError> {
    validate_id(id)?;
    Ok(generations_root(context).join(id))
}

/// Resolve a project-scoped resource inside a generation snapshot.
///
/// The reference must already have been validated as Project-scoped.
/// This function does not follow symlinks or verify file existence; callers
/// performing filesystem I/O must enforce those checks separately.
pub fn generation_project_resource_path(
    generation_root: &Path,
    reference: &crate::project_resource::ResourceReference,
) -> Result<PathBuf, GenerationError> {
    use crate::project_resource::ResourceScope;
    use std::path::Component;

    if reference.scope != ResourceScope::Project
        || reference.path.as_os_str().is_empty()
        || reference
            .path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(GenerationError::InvalidCandidate(
            "generation project resource must use a nonempty relative path without traversal"
                .into(),
        ));
    }
    Ok(generation_root.join("resources").join(&reference.path))
}

/// Read-only diagnostics for registered Project resources in a generation.
///
/// Missing resources remain registered and do not block a save. Unsafe paths,
/// including symlinks and junctions, are reported separately from missing files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationResourceStatus {
    Available,
    Missing,
    Unsafe,
    Inaccessible,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationResourceDiagnostic {
    pub resource_id: String,
    pub reference: crate::project_resource::ResourceReference,
    pub status: GenerationResourceStatus,
}

fn resource_metadata_is_unsafe(metadata: &fs::Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return true;
        }
    }
    false
}

fn resource_metadata_error_status(error: &io::Error) -> GenerationResourceStatus {
    if error.kind() == io::ErrorKind::NotFound {
        GenerationResourceStatus::Missing
    } else {
        GenerationResourceStatus::Inaccessible
    }
}

/// Inspect registered Project resources without copying, repairing, or mutating
/// the generation. Application and External references are not inspected.
pub fn diagnose_generation_project_resources(
    generation_root: &Path,
    settings: &FrameworkSettings,
) -> Vec<GenerationResourceDiagnostic> {
    use crate::framework_settings::StoredResourceScope;
    use crate::project_resource::ResourceScope;

    settings
        .resources
        .iter()
        .filter(|entry| entry.scope == StoredResourceScope::Project)
        .map(|entry| {
            let reference = crate::project_resource::ResourceReference {
                scope: ResourceScope::Project,
                path: entry.path.clone(),
            };
            let status = match generation_project_resource_path(generation_root, &reference) {
                Err(_) => GenerationResourceStatus::Unsafe,
                Ok(path) => {
                    let resource_root = generation_root.join("resources");
                    let mut current = resource_root.clone();
                    let mut unsafe_path = false;
                    let mut inaccessible = false;
                    let mut missing = false;
                    // Validate the root before examining descendants: never traverse
                    // through a root junction or symbolic link.
                    match fs::symlink_metadata(&resource_root) {
                        Ok(metadata)
                            if resource_metadata_is_unsafe(&metadata) || !metadata.is_dir() =>
                        {
                            unsafe_path = true;
                        }
                        Err(error) => match resource_metadata_error_status(&error) {
                            GenerationResourceStatus::Missing => missing = true,
                            _ => inaccessible = true,
                        },
                        _ => {}
                    }
                    if !unsafe_path && !inaccessible && !missing {
                        for component in reference.path.components() {
                            current.push(component.as_os_str());
                            match fs::symlink_metadata(&current) {
                                Ok(metadata) => {
                                    if resource_metadata_is_unsafe(&metadata)
                                        || (current != path && !metadata.is_dir())
                                        || (current == path && !metadata.is_file())
                                    {
                                        unsafe_path = true;
                                        break;
                                    }
                                }
                                Err(error) => {
                                    match resource_metadata_error_status(&error) {
                                        GenerationResourceStatus::Missing => missing = true,
                                        _ => inaccessible = true,
                                    }
                                    break;
                                }
                            }
                        }
                    }
                    if unsafe_path {
                        GenerationResourceStatus::Unsafe
                    } else if inaccessible {
                        GenerationResourceStatus::Inaccessible
                    } else if missing {
                        GenerationResourceStatus::Missing
                    } else {
                        GenerationResourceStatus::Available
                    }
                }
            };
            GenerationResourceDiagnostic {
                resource_id: entry.resource_id.clone(),
                reference,
                status,
            }
        })
        .collect()
}

pub fn committed_path(context: &ProjectContext) -> PathBuf {
    generations_root(context).join("committed.toml")
}

/// Create an empty, exclusive staging directory. The caller writes all three
/// required components and validates their contents before promotion.
pub fn begin_generation(context: &ProjectContext, id: &str) -> Result<PathBuf, GenerationError> {
    let path = staging_path(context, id)?;
    if generation_path(context, id)?.exists() || path.exists() {
        return Err(GenerationError::AlreadyExists);
    }
    fs::create_dir_all(path.parent().expect("staging path has parent"))?;
    fs::create_dir(&path)?;
    Ok(path)
}

fn require_complete(path: &Path) -> Result<(), GenerationError> {
    if !path.join("application").is_dir()
        || !path.join("framework_settings.toml").is_file()
        || !path.join("project.toml").is_file()
    {
        return Err(GenerationError::IncompleteCandidate);
    }
    Ok(())
}

/// Promote a complete candidate without changing the currently committed ID.
fn promote_generation(context: &ProjectContext, id: &str) -> Result<(), GenerationError> {
    let source = staging_path(context, id)?;
    let target = generation_path(context, id)?;
    if target.exists() {
        return Err(GenerationError::AlreadyExists);
    }
    if !source.is_dir() {
        return Err(GenerationError::MissingCandidate);
    }
    require_complete(&source)?;
    fs::rename(source, target)?;
    Ok(())
}

/// Commit a previously promoted generation. Never points to an incomplete candidate.
fn commit_generation(context: &ProjectContext, id: &str) -> Result<(), GenerationError> {
    let path = generation_path(context, id)?;
    if !path.is_dir() {
        return Err(GenerationError::MissingCandidate);
    }
    require_complete(&path)?;
    let marker = CommittedGeneration {
        format_version: 1,
        generation_id: id.to_owned(),
    };
    let text =
        toml::to_string(&marker).map_err(|e| GenerationError::InvalidCommit(e.to_string()))?;
    atomic_write(&committed_path(context), text.as_bytes())?;
    Ok(())
}

/// None means legacy format; an invalid pointer is always an error, not legacy fallback.
pub fn read_committed_generation(
    context: &ProjectContext,
) -> Result<Option<CommittedGeneration>, GenerationError> {
    let text = match fs::read_to_string(committed_path(context)) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let marker: CommittedGeneration =
        toml::from_str(&text).map_err(|e| GenerationError::InvalidCommit(e.to_string()))?;
    if marker.format_version != 1 {
        return Err(GenerationError::InvalidCommit(
            "unsupported format version".into(),
        ));
    }
    validate_id(&marker.generation_id)?;
    let path = generation_path(context, &marker.generation_id)?;
    if !path.is_dir() {
        return Err(GenerationError::MissingCommittedGeneration);
    }
    require_complete(&path)?;
    Ok(Some(marker))
}

/// Persisted project storage format, derived from the committed pointer rather
/// than from adapter capabilities or legacy metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectStorageFormat {
    Legacy,
    Generation,
}

/// Detect the persisted storage format without mutating project contents.
/// An invalid committed pointer or incomplete committed generation is an error,
/// never a reason to fall back to the legacy format.
///
/// Application payload validation remains the responsibility of generation-aware
/// Open and Save operations; this function only checks the commit structure.
pub fn detect_project_storage_format(
    context: &ProjectContext,
) -> Result<ProjectStorageFormat, GenerationError> {
    match read_committed_generation(context)? {
        Some(_) => Ok(ProjectStorageFormat::Generation),
        None => Ok(ProjectStorageFormat::Legacy),
    }
}

/// Validate the contents of a save candidate before it becomes visible.
/// The application payload remains application-owned and needs its own adapter validation.
pub fn validate_generation_contents(
    path: &Path,
    expected_application_id: &str,
    expected_save_id: &str,
) -> Result<(), GenerationError> {
    require_complete(path)?;
    let project_text = fs::read_to_string(path.join("project.toml"))?;
    let project = ProjectFile::from_toml(&project_text, expected_application_id)
        .map_err(|e| GenerationError::InvalidCandidate(format!("project.toml: {e:?}")))?;
    if project.project.format_version != PROJECT_FORMAT_VERSION
        || project.project.save_id != expected_save_id
    {
        return Err(GenerationError::InvalidCandidate(
            "project format version or save_id mismatch".into(),
        ));
    }
    let framework_text = fs::read_to_string(path.join("framework_settings.toml"))?;
    let framework = FrameworkSettings::from_toml(&framework_text).map_err(|e| {
        GenerationError::InvalidCandidate(format!("framework_settings.toml: {e:?}"))
    })?;
    if framework.format_version != FRAMEWORK_SETTINGS_FORMAT_VERSION
        || framework.save_id.as_deref() != Some(expected_save_id)
    {
        return Err(GenerationError::InvalidCandidate(
            "framework format version or save_id mismatch".into(),
        ));
    }
    Ok(())
}

/// Validate and publish a staged generation as one controlled operation.
///
/// The application callback must validate the staged application directory
/// without modifying the committed generation or legacy project files.
/// On validation failure, the committed pointer is left unchanged.
pub fn validate_and_commit_generation<F>(
    context: &ProjectContext,
    generation_id: &str,
    expected_application_id: &str,
    expected_save_id: &str,
    validate_application: F,
) -> Result<(), GenerationError>
where
    F: FnOnce(&Path) -> Result<(), GenerationError>,
{
    let stage = staging_path(context, generation_id)?;
    if !stage.is_dir() {
        return Err(GenerationError::MissingCandidate);
    }
    validate_generation_contents(&stage, expected_application_id, expected_save_id)?;
    validate_application(&stage.join("application"))?;
    promote_generation(context, generation_id)?;
    commit_generation(context, generation_id)
}

/// Resolve the committed generation only after verifying its stored contents.
/// A corrupt committed generation is an error; it must never trigger legacy fallback.
pub fn read_validated_committed_generation<F>(
    context: &ProjectContext,
    expected_application_id: &str,
    validate_application: F,
) -> Result<Option<CommittedGeneration>, GenerationError>
where
    F: FnOnce(&Path) -> Result<(), GenerationError>,
{
    let Some(marker) = read_committed_generation(context)? else {
        return Ok(None);
    };
    let path = generation_path(context, &marker.generation_id)?;
    let project_text = fs::read_to_string(path.join("project.toml"))?;
    let project = ProjectFile::from_toml(&project_text, expected_application_id)
        .map_err(|e| GenerationError::InvalidCandidate(format!("project.toml: {e:?}")))?;
    validate_generation_contents(&path, expected_application_id, &project.project.save_id)?;
    validate_application(&path.join("application"))?;
    Ok(Some(marker))
}

/// The physical paths of the validated committed snapshot.
///
/// These paths are intentionally explicit: a generation stores framework
/// settings at its root, unlike the legacy ProjectContext layout.
/// Callers must not treat a generation directory as a legacy project root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommittedGenerationPaths {
    pub generation_id: String,
    pub project_file: PathBuf,
    pub framework_settings: PathBuf,
    pub application_directory: PathBuf,
}

/// Resolve a validated committed snapshot, or None only for a legacy project.
/// An invalid commit pointer or invalid generation is never a legacy fallback.
pub fn resolve_validated_committed_paths<F>(
    context: &ProjectContext,
    expected_application_id: &str,
    validate_application: F,
) -> Result<Option<CommittedGenerationPaths>, GenerationError>
where
    F: FnOnce(&Path) -> Result<(), GenerationError>,
{
    let Some(marker) = read_validated_committed_generation(
        context,
        expected_application_id,
        validate_application,
    )?
    else {
        return Ok(None);
    };
    let root = generation_path(context, &marker.generation_id)?;
    Ok(Some(CommittedGenerationPaths {
        generation_id: marker.generation_id,
        project_file: root.join("project.toml"),
        framework_settings: root.join("framework_settings.toml"),
        application_directory: root.join("application"),
    }))
}

/// Stage and publish one complete save without modifying the legacy layout.
///
/// This is an explicit opt-in API, not yet connected to ProjectController::save.
/// The application must support staged saving; an unsupported adapter is rejected
/// rather than falling back to the non-transactional legacy save path.
/// In-memory session and metadata are updated only after the commit pointer
/// has been published successfully.
pub struct GenerationSaveIdentity<'a> {
    pub generation_id: &'a str,
    pub save_id: &'a str,
    pub saved_at: &'a str,
}

/// Save a committed generation to a different, initially empty project root.
/// The original session and metadata are replaced only after destination commit.
pub fn save_project_generation_as(
    session: &mut crate::project_lifecycle::ProjectSession,
    source: &ProjectContext,
    destination: &ProjectContext,
    project_file: &mut ProjectFile,
    framework_settings: &FrameworkSettings,
    application: &mut dyn crate::project_adapter_erased::ErasedApplicationProjectAdapter,
    identity: GenerationSaveIdentity<'_>,
) -> Result<(), GenerationSaveError> {
    if source.root() == destination.root() {
        return Err(GenerationSaveError::Generation(
            GenerationError::InvalidCandidate("Save As destination must differ from source".into()),
        ));
    }
    if destination.root().exists()
        && fs::read_dir(destination.root())
            .map_err(GenerationSaveError::Io)?
            .next()
            .is_some()
    {
        return Err(GenerationSaveError::Generation(
            GenerationError::InvalidCandidate("Save As destination must be empty".into()),
        ));
    }
    let paths = resolve_validated_committed_paths(source, &project_file.application.id, |_| Ok(()))
        .map_err(GenerationSaveError::Generation)?
        .ok_or(GenerationSaveError::Generation(
            GenerationError::MissingCommittedGeneration,
        ))?;
    let committed_text =
        fs::read_to_string(&paths.project_file).map_err(GenerationSaveError::Io)?;
    let committed_project = ProjectFile::from_toml(&committed_text, &project_file.application.id)
        .map_err(|error| {
        GenerationSaveError::Generation(GenerationError::InvalidCandidate(format!("{error:?}")))
    })?;
    let source_generation = ProjectContext::new(
        paths
            .application_directory
            .parent()
            .expect("generation has parent"),
    );
    use crate::project_resource::{ProjectDataCompatibility, ProjectDataConsistency};
    match application
        .inspect_project_data_generation(
            source,
            &source_generation,
            committed_project.application.data_version.as_deref(),
        )
        .map_err(GenerationSaveError::Application)?
    {
        Some((ProjectDataCompatibility::Compatible, ProjectDataConsistency::Consistent)) => {}
        _ => {
            return Err(GenerationSaveError::Generation(
                GenerationError::InvalidCandidate(
                    "source generation application is not compatible and consistent".into(),
                ),
            ));
        }
    }
    let mut candidate_session = crate::project_lifecycle::ProjectSession {
        dirty: session.dirty,
        current_save_id: session.current_save_id.clone(),
        storage: crate::project_lifecycle::ProjectStorageState::Stored(destination.clone()),
    };
    let mut candidate_project = project_file.clone();
    save_project_generation_from(
        &mut candidate_session,
        destination,
        &source_generation,
        &mut candidate_project,
        framework_settings,
        application,
        identity,
    )?;
    *session = candidate_session;
    *project_file = candidate_project;
    Ok(())
}

pub fn save_project_generation(
    session: &mut crate::project_lifecycle::ProjectSession,
    context: &ProjectContext,
    project_file: &mut ProjectFile,
    framework_settings: &FrameworkSettings,
    application: &mut dyn crate::project_adapter_erased::ErasedApplicationProjectAdapter,
    identity: GenerationSaveIdentity<'_>,
) -> Result<(), GenerationSaveError> {
    let GenerationSaveIdentity {
        generation_id,
        save_id,
        saved_at,
    } = identity;
    // Resolve the active snapshot before creating a new stage. An invalid
    // committed generation must never silently fall back to legacy files.
    let source_generation =
        resolve_validated_committed_paths(context, &project_file.application.id, |_| Ok(()))
            .map_err(GenerationSaveError::Generation)?;
    if let Some(paths) = source_generation.as_ref() {
        use crate::project_resource::{ProjectDataCompatibility, ProjectDataConsistency};
        let stored = fs::read_to_string(&paths.project_file).map_err(GenerationSaveError::Io)?;
        let committed_project = ProjectFile::from_toml(&stored, &project_file.application.id)
            .map_err(|e| {
                GenerationSaveError::Generation(GenerationError::InvalidCandidate(format!(
                    "committed project: {e:?}"
                )))
            })?;
        let generation_root = paths
            .application_directory
            .parent()
            .expect("generation has parent");
        let generation_context = ProjectContext::new(generation_root);
        let inspection = application
            .inspect_project_data_generation(
                context,
                &generation_context,
                committed_project.application.data_version.as_deref(),
            )
            .map_err(GenerationSaveError::Application)?;
        match inspection {
            Some((ProjectDataCompatibility::Compatible, ProjectDataConsistency::Consistent)) => {}
            Some(_) => {
                return Err(GenerationSaveError::Generation(
                    GenerationError::InvalidCandidate(
                        "committed application data is not compatible and consistent".into(),
                    ),
                ));
            }
            None => {
                return Err(GenerationSaveError::Generation(
                    GenerationError::InvalidCandidate(
                        "committed application generation inspection is not supported".into(),
                    ),
                ));
            }
        }
    }
    let source_context = source_generation
        .as_ref()
        .map(|paths| {
            ProjectContext::new(
                paths
                    .application_directory
                    .parent()
                    .expect("generation application directory has a parent"),
            )
        })
        .unwrap_or_else(|| context.clone());
    save_project_generation_from(
        session,
        context,
        &source_context,
        project_file,
        framework_settings,
        application,
        GenerationSaveIdentity {
            generation_id,
            save_id,
            saved_at,
        },
    )
}

/// Copy registered project resources from the source snapshot into the new stage.
/// Missing references are left unresolved for compatibility with existing
/// registry-only projects; existing symlinks and non-files are rejected.
fn copy_generation_project_resources(
    source: &Path,
    stage: &Path,
    settings: &FrameworkSettings,
) -> Result<(), GenerationSaveError> {
    use crate::project_resource::{ResourceReference, ResourceScope};
    for entry in &settings.resources {
        if entry.scope != crate::framework_settings::StoredResourceScope::Project {
            continue;
        }
        let reference = ResourceReference::new(ResourceScope::Project, entry.path.clone())
            .map_err(|error| {
                GenerationSaveError::Generation(GenerationError::InvalidCandidate(format!(
                    "invalid project resource {}: {error:?}",
                    entry.resource_id
                )))
            })?;
        let from = generation_project_resource_path(source, &reference)
            .map_err(GenerationSaveError::Generation)?;
        let to = generation_project_resource_path(stage, &reference)
            .map_err(GenerationSaveError::Generation)?;
        let root = source.join("resources");
        let mut ancestor = from.parent();
        while let Some(path) = ancestor {
            if !path.starts_with(&root) {
                break;
            }
            match fs::symlink_metadata(path) {
                Ok(meta) if meta.file_type().is_symlink() || !meta.is_dir() => {
                    return Err(GenerationSaveError::Generation(
                        GenerationError::InvalidCandidate(format!(
                            "unsafe project resource ancestor: {}",
                            path.display()
                        )),
                    ));
                }
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(GenerationSaveError::Io(error)),
            }
            if path == root {
                break;
            }
            ancestor = path.parent();
        }
        match fs::symlink_metadata(&from) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(GenerationSaveError::Io(error)),
            Ok(meta) if !meta.is_file() || meta.file_type().is_symlink() => {
                return Err(GenerationSaveError::Generation(
                    GenerationError::InvalidCandidate(format!(
                        "project resource is not a regular file: {}",
                        from.display()
                    )),
                ));
            }
            Ok(_) => {}
        }
        fs::create_dir_all(to.parent().expect("resource has parent"))
            .map_err(GenerationSaveError::Io)?;
        fs::copy(&from, &to).map_err(GenerationSaveError::Io)?;
    }
    Ok(())
}

fn save_project_generation_from(
    session: &mut crate::project_lifecycle::ProjectSession,
    context: &ProjectContext,
    source_context: &ProjectContext,
    project_file: &mut ProjectFile,
    framework_settings: &FrameworkSettings,
    application: &mut dyn crate::project_adapter_erased::ErasedApplicationProjectAdapter,
    identity: GenerationSaveIdentity<'_>,
) -> Result<(), GenerationSaveError> {
    let GenerationSaveIdentity {
        generation_id,
        save_id,
        saved_at,
    } = identity;
    let stage =
        begin_generation(context, generation_id).map_err(GenerationSaveError::Generation)?;
    // Only the staging directory created by this invocation is eligible for
    // cleanup. Never remove an already promoted or committed generation.
    let staged_result = (|| -> Result<ProjectFile, GenerationSaveError> {
        let stage_context = ProjectContext::new(&stage);
        copy_generation_project_resources(source_context.root(), &stage, framework_settings)?;
        let app = application
            .save_project_data_staged(source_context, &stage_context, save_id)
            .map_err(GenerationSaveError::Application)?
            .ok_or(GenerationSaveError::StagedSaveNotSupported)?;

        let mut settings_candidate = framework_settings.clone();
        settings_candidate.save_id = Some(save_id.to_owned());
        let settings_text = settings_candidate.to_toml().map_err(|e| {
            GenerationSaveError::Serialization(format!("framework settings: {e:?}"))
        })?;
        fs::write(stage.join("framework_settings.toml"), settings_text)
            .map_err(GenerationSaveError::Io)?;

        let mut project_candidate = project_file.clone();
        project_candidate.project.save_id = save_id.to_owned();
        project_candidate.project.saved_at = saved_at.to_owned();
        project_candidate.application.data_version = app.data_version;
        let project_text = project_candidate
            .to_toml()
            .map_err(|e| GenerationSaveError::Serialization(format!("project: {e:?}")))?;
        fs::write(stage.join("project.toml"), project_text).map_err(GenerationSaveError::Io)?;

        validate_and_commit_generation(
            context,
            generation_id,
            &project_candidate.application.id,
            save_id,
            |_| match application.validate_project_data_staged(
                context,
                &stage_context,
                project_candidate.application.data_version.as_deref(),
            ) {
                Ok(Some(())) => Ok(()),
                Ok(None) => Err(GenerationError::InvalidCandidate(
                    "application staged validation is not supported".into(),
                )),
                Err(error) => Err(GenerationError::InvalidCandidate(format!(
                    "application staged validation failed: {error}"
                ))),
            },
        )
        .map_err(GenerationSaveError::Generation)?;

        Ok(project_candidate)
    })();
    let project_candidate = match staged_result {
        Ok(candidate) => candidate,
        Err(error) => {
            if stage.is_dir() {
                // Preserve the original failure. A failed cleanup can be
                // retried explicitly using the uncommitted staging path.
                let _ = fs::remove_dir_all(&stage);
            }
            return Err(error);
        }
    };

    let pending = session
        .begin_save(save_id)
        .application_saved()
        .framework_saved()
        .project_file_saved();
    session
        .complete_save(pending)
        .expect("all save stages completed");
    *project_file = project_candidate;
    Ok(())
}

#[derive(Debug)]
pub enum GenerationSaveError {
    Generation(GenerationError),
    Application(String),
    StagedSaveNotSupported,
    Serialization(String),
    Io(io::Error),
}

#[cfg(test)]
mod generation_resource_diagnostic_error_tests {
    use super::{GenerationResourceStatus, resource_metadata_error_status};
    use std::io;

    #[test]
    fn res_diag_012_permission_denied_is_inaccessible() {
        let error = io::Error::from(io::ErrorKind::PermissionDenied);
        assert_eq!(
            resource_metadata_error_status(&error),
            GenerationResourceStatus::Inaccessible
        );
    }

    #[test]
    fn res_diag_013_missing_path_is_missing() {
        let error = io::Error::from(io::ErrorKind::NotFound);
        assert_eq!(
            resource_metadata_error_status(&error),
            GenerationResourceStatus::Missing
        );
    }

    #[test]
    fn res_diag_014_other_io_failure_is_inaccessible() {
        let error = io::Error::from(io::ErrorKind::Other);
        assert_eq!(
            resource_metadata_error_status(&error),
            GenerationResourceStatus::Inaccessible
        );
    }
}
