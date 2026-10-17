//! OS-backed project directory notifications.
//! Events are coalesced by ProjectPanelRefreshHandle.
use crate::project_panel_api::ProjectPanelRefreshHandle;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};

pub struct ProjectDirectoryWatcher {
    _watcher: RecommendedWatcher,
    paths: Vec<PathBuf>,
    root: PathBuf,
    requested_paths: Vec<PathBuf>,
}

impl ProjectDirectoryWatcher {
    pub fn start(
        root: &Path,
        relative_paths: &[PathBuf],
        refresh: ProjectPanelRefreshHandle,
    ) -> notify::Result<Self> {
        let canonical_root = std::fs::canonicalize(root)?;
        let mut requested_paths = relative_paths.to_vec();
        requested_paths.sort();
        requested_paths.dedup();
        let mut paths = Vec::new();
        for relative in relative_paths {
            if relative.as_os_str().is_empty()
                || relative
                    .components()
                    .any(|component| !matches!(component, std::path::Component::Normal(_)))
            {
                return Err(notify::Error::generic("invalid project directory path"));
            }
            let target = canonical_root.join(relative);
            if !target.exists() {
                // Watch the project root so a missing mount can appear later.
                continue;
            }
            let canonical = std::fs::canonicalize(&target)?;
            if !canonical.starts_with(&canonical_root) || !canonical.is_dir() {
                return Err(notify::Error::generic("directory outside project root"));
            }
            if !paths.contains(&canonical) {
                paths.push(canonical);
            }
        }
        let mut watcher =
            notify::recommended_watcher(move |event: notify::Result<notify::Event>| match event {
                Ok(_) => refresh.invalidate_application_project_panel(),
                Err(error) => {
                    tracing::warn!(target: "wfide::project_panel", ?error, "directory watch error");
                }
            })?;
        // Watching the root also survives removal and recreation of mount directories.
        // Individual directory watches are lost when the watched directory is deleted.
        if !requested_paths.is_empty() {
            watcher.watch(&canonical_root, RecursiveMode::Recursive)?;
        }
        if !requested_paths.is_empty() {
            paths = vec![canonical_root.clone()];
        }
        Ok(Self {
            _watcher: watcher,
            paths,
            root: canonical_root,
            requested_paths,
        })
    }

    /// Keep an existing watcher when the project and requested mounts are unchanged.
    pub fn matches(&self, root: &Path, relative_paths: &[PathBuf]) -> bool {
        let Ok(root) = std::fs::canonicalize(root) else {
            return false;
        };
        let mut requested = relative_paths.to_vec();
        requested.sort();
        requested.dedup();
        self.root == root && self.requested_paths == requested
    }

    pub fn watched_paths(&self) -> &[PathBuf] {
        &self.paths
    }
}
