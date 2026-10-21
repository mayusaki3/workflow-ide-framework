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
