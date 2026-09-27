use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const PROJECT_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectFile {
    pub project: ProjectMetadata,
    pub application: ApplicationMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub format_version: u32,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub language: String,
    pub save_id: String,
    pub saved_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplicationMetadata {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub language: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectFileError {
    Serialize(String),
    Parse(String),
    UnsupportedNewerFormat { found: u32, supported: u32 },
    InvalidFormatVersion(u32),
    ApplicationMismatch { expected: String, found: String },
}

impl ProjectFile {
    pub fn to_toml(&self) -> Result<String, ProjectFileError> {
        toml::to_string_pretty(self).map_err(|error| ProjectFileError::Serialize(error.to_string()))
    }

    pub fn from_toml(input: &str, expected_application_id: &str) -> Result<Self, ProjectFileError> {
        let file: Self = toml::from_str(input).map_err(|error| ProjectFileError::Parse(error.to_string()))?;
        if file.project.format_version == 0 {
            return Err(ProjectFileError::InvalidFormatVersion(0));
        }
        if file.project.format_version > PROJECT_FORMAT_VERSION {
            return Err(ProjectFileError::UnsupportedNewerFormat {
                found: file.project.format_version,
                supported: PROJECT_FORMAT_VERSION,
            });
        }
        if file.application.id != expected_application_id {
            return Err(ProjectFileError::ApplicationMismatch {
                expected: expected_application_id.to_owned(),
                found: file.application.id.clone(),
            });
        }
        Ok(file)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectContext {
    pub root: PathBuf,
}

impl ProjectContext {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn project_file_path(&self) -> PathBuf {
        self.root.join("project.toml")
    }

    pub fn framework_directory(&self) -> PathBuf {
        self.root.join("framework")
    }

    pub fn framework_settings_path(&self) -> PathBuf {
        self.framework_directory().join("framework_settings.toml")
    }

    pub fn resource_root(&self) -> PathBuf {
        self.root.join("resources")
    }

    pub fn application_directory(&self) -> PathBuf {
        self.root.join("application")
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}
