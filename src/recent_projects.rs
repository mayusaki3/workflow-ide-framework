use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const LIMIT: usize = 10;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RecentProjects {
    #[serde(default)]
    pub paths: Vec<PathBuf>,
}

impl RecentProjects {
    pub fn record(&mut self, path: PathBuf) {
        self.paths.retain(|existing| existing != &path);
        self.paths.insert(0, path);
        self.paths.truncate(LIMIT);
    }

    pub fn clear(&mut self) {
        self.paths.clear();
    }

    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|text| toml::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        std::fs::write(path, text)
    }
}

pub fn settings_path(application_id: &str) -> Option<PathBuf> {
    let base = std::env::var_os("APPDATA")
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME"))
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config").into_os_string()))?;
    let safe_id: String = application_id.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect();
    Some(PathBuf::from(base).join("workflow-ide-framework").join(safe_id).join("recent_projects.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newest_first_unique_and_capped() {
        let mut history = RecentProjects::default();
        for n in 0..12 {
            history.record(PathBuf::from(format!("project-{n}")));
        }
        history.record(PathBuf::from("project-5"));
        assert_eq!(history.paths.len(), 10);
        assert_eq!(history.paths[0], PathBuf::from("project-5"));
        assert_eq!(history.paths.iter().filter(|p| **p == PathBuf::from("project-5")).count(), 1);
    }

    #[test]
    fn clear_empties_history() {
        let mut history = RecentProjects::default();
        history.record(PathBuf::from("sample"));
        history.clear();
        assert!(history.paths.is_empty());
    }
}
