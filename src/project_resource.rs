use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewProjectStoragePolicy {
    Deferred,
    Required,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectDataCompatibility {
    Compatible,
    Converted { reason: Option<String>, handled: bool },
    Incompatible { reason: Option<String>, handled: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectDataConsistency {
    Consistent,
    Inconsistent {
        reason: Option<String>,
        can_open: bool,
        can_recover: bool,
        handled: bool,
    },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProjectDirtyState {
    pub metadata: bool,
    pub framework: bool,
    pub application: bool,
}

impl ProjectDirtyState {
    pub fn is_dirty(self) -> bool {
        self.metadata || self.framework || self.application
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceScope {
    Application,
    Project,
    External,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceReference {
    pub scope: ResourceScope,
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceReferenceError {
    ApplicationPathMustBeRelative,
    ProjectPathMustBeRelative,
    ExternalPathMustBeAbsolute,
    ParentTraversalNotAllowed,
    EmptyPath,
}

impl ResourceReference {
    pub fn new(scope: ResourceScope, path: impl Into<PathBuf>) -> Result<Self, ResourceReferenceError> {
        let path = normalize_path(path.into())?;
        match scope {
            ResourceScope::Application if path.is_absolute() => {
                return Err(ResourceReferenceError::ApplicationPathMustBeRelative);
            }
            ResourceScope::Project if path.is_absolute() => {
                return Err(ResourceReferenceError::ProjectPathMustBeRelative);
            }
            ResourceScope::External if !path.is_absolute() => {
                return Err(ResourceReferenceError::ExternalPathMustBeAbsolute);
            }
            _ => {}
        }
        Ok(Self { scope, path })
    }
}

fn normalize_path(path: PathBuf) -> Result<PathBuf, ResourceReferenceError> {
    if path.as_os_str().is_empty() {
        return Err(ResourceReferenceError::EmptyPath);
    }

    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => return Err(ResourceReferenceError::ParentTraversalNotAllowed),
            other => normalized.push(other.as_os_str()),
        }
    }

    if normalized.as_os_str().is_empty() {
        return Err(ResourceReferenceError::EmptyPath);
    }
    Ok(normalized)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceEntry {
    pub resource_id: String,
    pub reference: ResourceReference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceRootStatus {
    Available,
    Missing,
    Inaccessible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceWatcherStatus {
    Active,
    Degraded,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceState {
    Available,
    Missing,
    RootUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceSelectionMode {
    Single,
    Multiple,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTypeFilter {
    pub label: String,
    pub extensions: Vec<String>,
}

impl FileTypeFilter {
    pub fn new(label: impl Into<String>, extensions: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            label: label.into(),
            extensions: extensions
                .into_iter()
                .map(|extension| extension.into().trim().trim_start_matches('.').to_ascii_lowercase())
                .filter(|extension| !extension.is_empty())
                .collect(),
        }
    }

    pub fn accepts(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| self.extensions.iter().any(|allowed| allowed.eq_ignore_ascii_case(extension)))
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationJournalData {
    pub format: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceOperationDecision {
    Accept { journal_data: Option<ApplicationJournalData> },
    Reject { reason: Option<String>, handled: bool },
}
