//! Read-only projection of Workspace registry state into egui_dock trees.
//! The Workspace registry remains the authoritative placement model.
use std::collections::BTreeMap;

use egui_dock::DockState;

use crate::workspace::{ContainerKind, PanelInstanceId, Workspace, WorkspaceError};

/// Stable backend-independent tab key; does not conflate panel definitions and instances.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DockPanelKey {
    pub definition_id: String,
    pub instance_id: String,
}

impl From<&PanelInstanceId> for DockPanelKey {
    fn from(panel: &PanelInstanceId) -> Self {
        Self { definition_id: panel.definition_id.clone(), instance_id: panel.instance_id.clone() }
    }
}

/// One dock tree per normal Container. Empty containers have no tree.
pub struct WorkspaceDockProjection {
    pub normal: BTreeMap<String, Option<DockState<DockPanelKey>>>,
    /// Floating containers must be rendered as independent single-panel windows,
    /// never as drop targets in a normal dock tree.
    pub floating: BTreeMap<String, DockPanelKey>,
}

pub fn project_workspace(workspace: &Workspace) -> Result<WorkspaceDockProjection, WorkspaceError> {
    let mut normal = BTreeMap::new();
    let mut floating = BTreeMap::new();
    for (id, container) in &workspace.containers {
        match container.kind {
            ContainerKind::Normal => {
                let tabs: Vec<DockPanelKey> = container.panels.iter().map(DockPanelKey::from).collect();
                let tree = if tabs.is_empty() { None } else { Some(DockState::new(tabs)) };
                normal.insert(id.clone(), tree);
            }
            ContainerKind::Floating => {
                if container.panels.len() != 1 || !workspace.floating_geometry.contains_key(id) {
                    return Err(WorkspaceError::InvalidContainerTarget);
                }
                floating.insert(id.clone(), DockPanelKey::from(&container.panels[0]));
            }
        }
    }
    Ok(WorkspaceDockProjection { normal, floating })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::{FloatingGeometry, WorkspaceRegistry, DEFAULT_CONTAINER_ID, DEFAULT_WORKSPACE_ID};

    #[test]
    fn empty_default_container_does_not_require_a_dock_tree() {
        let registry = WorkspaceRegistry::new();
        let projected = project_workspace(registry.get(DEFAULT_WORKSPACE_ID).unwrap()).unwrap();
        assert!(projected.normal[DEFAULT_CONTAINER_ID].is_none());
    }

    #[test]
    fn floating_panel_is_excluded_from_normal_dock_trees() {
        let mut registry = WorkspaceRegistry::new();
        let panel = PanelInstanceId::new("editor", "instance-1");
        registry.register_panel(panel.clone());
        registry.place_panel(DEFAULT_WORKSPACE_ID, &panel, DEFAULT_CONTAINER_ID).unwrap();
        let id = registry.float_panel(DEFAULT_WORKSPACE_ID, &panel,
            FloatingGeometry { position: [10.0, 20.0], size: [200.0, 100.0] }).unwrap();
        let projected = project_workspace(registry.get(DEFAULT_WORKSPACE_ID).unwrap()).unwrap();
        assert!(projected.normal[DEFAULT_CONTAINER_ID].is_none());
        assert_eq!(projected.floating[&id].instance_id, "instance-1");
    }

    #[test]
    fn normal_container_tabs_preserve_instance_identity() {
        let mut registry = WorkspaceRegistry::new();
        let a = PanelInstanceId::new("editor", "a");
        let b = PanelInstanceId::new("editor", "b");
        registry.register_panel(a.clone());
        registry.register_panel(b.clone());
        registry.place_panel(DEFAULT_WORKSPACE_ID, &a, DEFAULT_CONTAINER_ID).unwrap();
        registry.place_panel(DEFAULT_WORKSPACE_ID, &b, DEFAULT_CONTAINER_ID).unwrap();
        let projected = project_workspace(registry.get(DEFAULT_WORKSPACE_ID).unwrap()).unwrap();
        assert!(projected.normal[DEFAULT_CONTAINER_ID].is_some());
        assert!(projected.floating.is_empty());
    }
}
