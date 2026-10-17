//! OS-backed project directory notifications.
//! Events are coalesced by ProjectPanelRefreshHandle.
use crate::project_panel_api::ProjectPanelRefreshHandle;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};

pub struct ProjectDirectoryWatcher {
    _watcher: RecommendedWatcher,
    paths: Vec<PathBuf>,
}

impl ProjectDirectoryWatcher {
    pub fn start(
        root: &Path,
        relative_paths: &[PathBuf],
        refresh: ProjectPanelRefreshHandle,
    ) -> notify::Result<Self> {
        let canonical_root = std::fs::canonicalize(root)?;
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
        for path in &paths {
            watcher.watch(path, RecursiveMode::Recursive)?;
        }
        Ok(Self { _watcher: watcher, paths })
    }

    pub fn watched_paths(&self) -> &[PathBuf] {
        &self.paths
    }
}
