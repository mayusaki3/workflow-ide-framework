use crate::project_resource::{ResourceEntry, ResourceReference, ResourceScope};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const FRAMEWORK_SETTINGS_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameworkSettings {
    pub format_version: u32,
    #[serde(default)]
    pub resources: Vec<StoredResourceEntry>,
}

impl Default for FrameworkSettings {
    fn default() -> Self {
        Self { format_version: FRAMEWORK_SETTINGS_FORMAT_VERSION, resources: Vec::new() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredResourceEntry {
    pub resource_id: String,
    pub scope: StoredResourceScope,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StoredResourceScope {
    Application,
    Project,
    External,
}

impl From<ResourceScope> for StoredResourceScope {
    fn from(value: ResourceScope) -> Self {
        match value {
            ResourceScope::Application => Self::Application,
            ResourceScope::Project => Self::Project,
            ResourceScope::External => Self::External,
        }
    }
}

impl StoredResourceEntry {
    pub fn from_resource(entry: &ResourceEntry) -> Self {
        Self {
            resource_id: entry.resource_id.clone(),
            scope: entry.reference.scope.into(),
            path: entry.reference.path.clone(),
        }
    }

    pub fn to_resource(&self) -> Result<ResourceEntry, crate::project_resource::ResourceReferenceError> {
        let scope = match self.scope {
            StoredResourceScope::Application => ResourceScope::Application,
            StoredResourceScope::Project => ResourceScope::Project,
            StoredResourceScope::External => ResourceScope::External,
        };
        Ok(ResourceEntry {
            resource_id: self.resource_id.clone(),
            reference: ResourceReference::new(scope, self.path.clone())?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameworkSettingsError {
    Serialize(String),
    Parse(String),
    UnsupportedNewerFormat { found: u32, supported: u32 },
    InvalidFormatVersion(u32),
}

impl FrameworkSettings {
    pub fn to_toml(&self) -> Result<String, FrameworkSettingsError> {
        toml::to_string_pretty(self).map_err(|error| FrameworkSettingsError::Serialize(error.to_string()))
    }

    pub fn from_toml(input: &str) -> Result<Self, FrameworkSettingsError> {
        let settings: Self = toml::from_str(input).map_err(|error| FrameworkSettingsError::Parse(error.to_string()))?;
        if settings.format_version == 0 {
            return Err(FrameworkSettingsError::InvalidFormatVersion(0));
        }
        if settings.format_version > FRAMEWORK_SETTINGS_FORMAT_VERSION {
            return Err(FrameworkSettingsError::UnsupportedNewerFormat {
                found: settings.format_version,
                supported: FRAMEWORK_SETTINGS_FORMAT_VERSION,
            });
        }
        Ok(settings)
    }
}
