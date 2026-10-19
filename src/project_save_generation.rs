//! Save-generation commit pointer. This module is independent of the legacy save path.
//! A generation is visible only after its complete directory is promoted and
//! its commit pointer is atomically replaced.
use crate::{project::ProjectContext, project_io::atomic_write};
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
pub fn promote_generation(context: &ProjectContext, id: &str) -> Result<(), GenerationError> {
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
pub fn commit_generation(context: &ProjectContext, id: &str) -> Result<(), GenerationError> {
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
