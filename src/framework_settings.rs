use crate::{
    project_resource::{ResourceEntry, ResourceReference, ResourceReferenceError, ResourceScope},
    resource_registry::ResourceRegistry,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const FRAMEWORK_SETTINGS_FORMAT_VERSION: u32 = 1;

/// Backend-independent layout snapshot stored in Project framework settings.
/// The tree uses stable panel instance identities, not egui_dock node indices.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredWorkspaceLayout {
    pub containers: Vec<StoredWorkspaceContainer>,
    #[serde(default)]
    pub hidden_panels: Vec<StoredHiddenPanel>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredWorkspaceContainer {
    pub id: String,
    pub floating: bool,
    #[serde(default)]
    pub geometry: Option<StoredFloatingGeometry>,
    #[serde(default)]
    pub tree: Option<StoredDockNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredFloatingGeometry {
    pub position: [f32; 2],
    pub size: [f32; 2],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredPanelIdentity {
    pub definition_id: String,
    pub instance_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StoredDockNode {
    Tabs {
        panels: Vec<StoredPanelIdentity>,
        active: usize,
    },
    Split {
        axis: StoredSplitAxis,
        fraction: f32,
        first: Box<StoredDockNode>,
        second: Box<StoredDockNode>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StoredSplitAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredHiddenPanel {
    pub panel: StoredPanelIdentity,
    #[serde(default)]
    pub normal_container: Option<String>,
    #[serde(default)]
    pub floating_geometry: Option<StoredFloatingGeometry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrameworkSettings {
    pub format_version: u32,
    #[serde(default)]
    pub save_id: Option<String>,
    #[serde(default)]
    pub resources: Vec<StoredResourceEntry>,
    /// Project-scoped user workspace layouts, keyed by stable Workspace ID.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub workspace_layouts: std::collections::BTreeMap<String, StoredWorkspaceLayout>,
}

impl Default for FrameworkSettings {
    fn default() -> Self {
        Self {
            format_version: FRAMEWORK_SETTINGS_FORMAT_VERSION,
            save_id: None,
            resources: Vec::new(),
            workspace_layouts: std::collections::BTreeMap::new(),
        }
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

    pub fn to_resource(
        &self,
    ) -> Result<ResourceEntry, crate::project_resource::ResourceReferenceError> {
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
    UnsupportedNewerFormat {
        found: u32,
        supported: u32,
    },
    InvalidFormatVersion(u32),
    InvalidResourceReference {
        resource_id: String,
        error: ResourceReferenceError,
    },
}

impl FrameworkSettings {
    pub fn to_toml(&self) -> Result<String, FrameworkSettingsError> {
        toml::to_string_pretty(self)
            .map_err(|error| FrameworkSettingsError::Serialize(error.to_string()))
    }

    pub fn from_registry(registry: &ResourceRegistry) -> Self {
        Self {
            format_version: FRAMEWORK_SETTINGS_FORMAT_VERSION,
            save_id: None,
            resources: registry
                .entries()
                .iter()
                .map(StoredResourceEntry::from_resource)
                .collect(),
            workspace_layouts: std::collections::BTreeMap::new(),
        }
    }

    pub fn to_registry(&self) -> Result<ResourceRegistry, FrameworkSettingsError> {
        let mut entries = Vec::with_capacity(self.resources.len());
        for stored in &self.resources {
            entries.push(stored.to_resource().map_err(|error| {
                FrameworkSettingsError::InvalidResourceReference {
                    resource_id: stored.resource_id.clone(),
                    error,
                }
            })?);
        }
        Ok(ResourceRegistry::new(entries))
    }

    pub fn from_toml(input: &str) -> Result<Self, FrameworkSettingsError> {
        let settings: Self = toml::from_str(input)
            .map_err(|error| FrameworkSettingsError::Parse(error.to_string()))?;
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
