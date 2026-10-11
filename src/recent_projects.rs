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

fn config_base() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA").map(PathBuf::from)
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME")
            .map(|home| PathBuf::from(home).join("Library").join("Application Support"))
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        std::env::var_os("XDG_CONFIG_HOME")
            .filter(|value| Path::new(value).is_absolute())
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
    }
}

/// Resolve an application-owned settings directory name.
pub fn directory_name(application_id: &str, configured_name: Option<&str>) -> Option<String> {
    if let Some(name) = configured_name {
        if name.is_empty()
            || name == "."
            || name == ".."
            || name.ends_with('.')
            || name.ends_with(' ')
            || name.chars().any(|c| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
        {
            return None;
        }
        return Some(name.to_owned());
    }
    let safe_id: String = application_id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    (!safe_id.is_empty()).then_some(safe_id)
}

pub fn settings_path(application_id: &str, configured_name: Option<&str>) -> Option<PathBuf> {
    let name = directory_name(application_id, configured_name)?;
    config_base().map(|base| base.join(name).join("recent_projects.toml"))
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
    fn explicit_directory_name_and_invalid_names() {
        assert_eq!(directory_name("org.example.app", Some("ExampleApp")).as_deref(), Some("ExampleApp"));
        assert_eq!(directory_name("org.example.app", None).as_deref(), Some("org_example_app"));
        for invalid in ["", ".", "..", "../other", "a/b", "a\\\\b", "bad:name", "trailing."] {
            assert_eq!(directory_name("org.example.app", Some(invalid)), None);
        }
    }

    #[test]
    fn clear_empties_history() {
        let mut history = RecentProjects::default();
        history.record(PathBuf::from("sample"));
        history.clear();
        assert!(history.paths.is_empty());
    }
}
