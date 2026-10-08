use crate::{
    project_resource::{
        FileTypeFilter, ResourceReference, ResourceReferenceError, ResourceScope,
        ResourceSelectionMode,
    },
    resource_state::ResourceRoots,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileSelectionResult {
    Selected(Vec<ResourceReference>),
    Cancelled,
    Error(FileSelectionError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileSelectionError {
    InvalidCount {
        mode: ResourceSelectionMode,
        count: usize,
    },
    FileDoesNotExist(PathBuf),
    NotRegularFile(PathBuf),
    ExtensionNotAllowed(PathBuf),
    RootUnavailable(ResourceScope),
    OutsideRoot {
        scope: ResourceScope,
        path: PathBuf,
    },
    Reference(ResourceReferenceError),
    Io(String),
}

#[derive(Debug, Clone)]
pub struct FileSelectionRequest {
    pub mode: ResourceSelectionMode,
    pub filters: Vec<FileTypeFilter>,
}

impl FileSelectionRequest {
    pub fn validate(
        &self,
        roots: &ResourceRoots,
        scope: ResourceScope,
        selected_paths: Vec<PathBuf>,
    ) -> FileSelectionResult {
        let count = selected_paths.len();
        if count == 0 || (self.mode == ResourceSelectionMode::Single && count != 1) {
            return FileSelectionResult::Error(FileSelectionError::InvalidCount {
                mode: self.mode,
                count,
            });
        }

        let mut references = Vec::with_capacity(count);
        for path in selected_paths {
            match validate_one(roots, scope, &path, &self.filters) {
                Ok(reference) => references.push(reference),
                Err(error) => return FileSelectionResult::Error(error),
            }
        }
        FileSelectionResult::Selected(references)
    }
}

fn validate_one(
    roots: &ResourceRoots,
    scope: ResourceScope,
    path: &Path,
    filters: &[FileTypeFilter],
) -> Result<ResourceReference, FileSelectionError> {
    let metadata = fs::metadata(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            FileSelectionError::FileDoesNotExist(path.to_path_buf())
        } else {
            FileSelectionError::Io(error.to_string())
        }
    })?;
    if !metadata.is_file() {
        return Err(FileSelectionError::NotRegularFile(path.to_path_buf()));
    }
    if !filters.is_empty() && !filters.iter().any(|filter| filter.accepts(path)) {
        return Err(FileSelectionError::ExtensionNotAllowed(path.to_path_buf()));
    }

    match scope {
        ResourceScope::External => {
            let absolute = fs::canonicalize(path)
                .map_err(|error| FileSelectionError::Io(error.to_string()))?;
            ResourceReference::new(scope, absolute).map_err(FileSelectionError::Reference)
        }
        ResourceScope::Application | ResourceScope::Project => {
            let root = match scope {
                ResourceScope::Application => &roots.application,
                ResourceScope::Project => &roots.project,
                ResourceScope::External => unreachable!(),
            };
            let canonical_root =
                fs::canonicalize(root).map_err(|_| FileSelectionError::RootUnavailable(scope))?;
            let canonical_path = fs::canonicalize(path)
                .map_err(|error| FileSelectionError::Io(error.to_string()))?;
            let relative = canonical_path.strip_prefix(&canonical_root).map_err(|_| {
                FileSelectionError::OutsideRoot {
                    scope,
                    path: path.to_path_buf(),
                }
            })?;
            ResourceReference::new(scope, relative.to_path_buf())
                .map_err(FileSelectionError::Reference)
        }
    }
}
