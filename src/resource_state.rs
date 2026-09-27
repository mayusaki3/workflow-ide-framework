use crate::project_resource::{ResourceReference, ResourceRootStatus, ResourceScope, ResourceState};
use std::{fs, path::{Path, PathBuf}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceRoots {
    pub application: PathBuf,
    pub project: PathBuf,
}

impl ResourceRoots {
    pub fn resolve(&self, reference: &ResourceReference) -> PathBuf {
        match reference.scope {
            ResourceScope::Application => self.application.join(&reference.path),
            ResourceScope::Project => self.project.join(&reference.path),
            ResourceScope::External => reference.path.clone(),
        }
    }

    pub fn root_status(&self, scope: ResourceScope) -> Option<ResourceRootStatus> {
        let root = match scope {
            ResourceScope::Application => &self.application,
            ResourceScope::Project => &self.project,
            ResourceScope::External => return None,
        };
        Some(path_status(root))
    }

    pub fn state(&self, reference: &ResourceReference) -> ResourceState {
        if let Some(status) = self.root_status(reference.scope) {
            if status != ResourceRootStatus::Available {
                return ResourceState::RootUnavailable;
            }
        }
        let resolved = self.resolve(reference);
        match fs::metadata(resolved) {
            Ok(metadata) if metadata.is_file() => ResourceState::Available,
            _ => ResourceState::Missing,
        }
    }
}

fn path_status(path: &Path) -> ResourceRootStatus {
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_dir() => ResourceRootStatus::Available,
        Ok(_) => ResourceRootStatus::Inaccessible,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => ResourceRootStatus::Missing,
        Err(_) => ResourceRootStatus::Inaccessible,
    }
}
