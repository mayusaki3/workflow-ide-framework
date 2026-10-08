//! Read-only projection of Workspace registry state into egui_dock trees.
//! The Workspace registry remains the authoritative placement model.
use std::collections::BTreeMap;

use egui_dock::DockState;

use crate::workspace::{
    ContainerKind, FloatingGeometry, PanelInstanceId, Workspace, WorkspaceError, WorkspaceRegistry,
};

/// Stable backend-independent tab key; does not conflate panel definitions and instances.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DockPanelKey {
    pub definition_id: String,
    pub instance_id: String,
}

impl From<&PanelInstanceId> for DockPanelKey {
    fn from(panel: &PanelInstanceId) -> Self {
        Self {
            definition_id: panel.definition_id.clone(),
            instance_id: panel.instance_id.clone(),
        }
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
                let tabs: Vec<DockPanelKey> =
                    container.panels.iter().map(DockPanelKey::from).collect();
                let tree = if tabs.is_empty() {
                    None
                } else {
                    Some(DockState::new(tabs))
                };
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

/// Reconcile all normal dock trees atomically. Tab moves between normal
/// containers are accepted, but no panel may disappear, duplicate, or enter
/// a floating container. Preserve each live DockState's split topology.
pub fn sync_normal_tab_order(
    registry: &mut WorkspaceRegistry,
    workspace_id: &str,
    projection: &WorkspaceDockProjection,
) -> Result<(), WorkspaceError> {
    use std::collections::BTreeSet;
    let ws = registry.get_mut_workspace_for_dock(workspace_id)?;
    let expected_containers: BTreeSet<_> = ws
        .containers
        .iter()
        .filter(|(_, c)| c.kind == ContainerKind::Normal)
        .map(|(id, _)| id.clone())
        .collect();
    let actual_containers: BTreeSet<_> = projection.normal.keys().cloned().collect();
    if expected_containers != actual_containers {
        return Err(WorkspaceError::InvalidContainerTarget);
    }
    let expected_panels: BTreeSet<_> = ws
        .containers
        .values()
        .flat_map(|c| c.panels.iter().cloned())
        .collect();
    let mut observed_panels = BTreeSet::new();
    let mut updates = Vec::new();
    for (container_id, tree) in &projection.normal {
        let mut observed = Vec::new();
        if let Some(tree) = tree {
            for node in tree.main_surface().iter() {
                if let egui_dock::Node::Leaf(leaf) = node {
                    for tab in &leaf.tabs {
                        let panel = PanelInstanceId::new(&tab.definition_id, &tab.instance_id);
                        if !observed_panels.insert(panel.clone()) {
                            return Err(WorkspaceError::InvalidContainerTarget);
                        }
                        observed.push(panel);
                    }
                }
            }
        }
        updates.push((container_id.clone(), observed));
    }
    for (container_id, panel) in &projection.floating {
        let container = ws
            .containers
            .get(container_id)
            .ok_or_else(|| WorkspaceError::ContainerNotFound(container_id.clone()))?;
        let identity = PanelInstanceId::new(&panel.definition_id, &panel.instance_id);
        if container.kind != ContainerKind::Floating
            || container.panels.as_slice() != [identity.clone()]
            || !observed_panels.insert(identity)
        {
            return Err(WorkspaceError::InvalidContainerTarget);
        }
    }
    if expected_panels != observed_panels {
        return Err(WorkspaceError::InvalidContainerTarget);
    }
    for (id, panels) in updates {
        ws.containers
            .get_mut(&id)
            .expect("validated container")
            .panels = panels;
    }
    Ok(())
}

/// Build the default Container's initial split tree from the application layout.
/// The registry owns panel placement; this only chooses their visual arrangement.
pub fn apply_initial_layout(
    projection: &mut WorkspaceDockProjection,
    layout: &crate::layout::LayoutConfig,
) -> Result<(), WorkspaceError> {
    use crate::workspace::DEFAULT_CONTAINER_ID;
    let Some(slot) = projection.normal.get_mut(DEFAULT_CONTAINER_ID) else {
        return Err(WorkspaceError::ContainerNotFound(
            DEFAULT_CONTAINER_ID.into(),
        ));
    };
    // Read the source panel identities from the initial layout instead of
    // depending on the egui_dock Node iterator representation.
    let all = layout
        .root_panel_ids
        .iter()
        .chain(
            layout
                .splits
                .iter()
                .flat_map(|split| split.panel_ids.iter()),
        )
        .map(|id| DockPanelKey {
            definition_id: id.clone(),
            instance_id: id.clone(),
        })
        .collect::<Vec<_>>();
    // The projection is initially a single root leaf. Count its tabs using
    // the public tree iterator rather than matching egui_dock's tuple variant.
    let expected_count = slot
        .as_ref()
        .map(|dock| {
            dock.main_surface()
                .iter()
                .filter_map(|node| match node {
                    egui_dock::Node::Leaf(leaf) => Some(leaf.tabs.len()),
                    _ => None,
                })
                .sum::<usize>()
        })
        .unwrap_or(0);
    if all.len() != expected_count {
        return Err(WorkspaceError::InvalidContainerTarget);
    }
    let mut keys = BTreeMap::new();
    for key in all {
        keys.insert(key.definition_id.clone(), key);
    }
    let mut used = std::collections::BTreeSet::new();
    let mut lookup = |id: &str| -> Result<DockPanelKey, WorkspaceError> {
        let key = keys
            .get(id)
            .ok_or_else(|| WorkspaceError::PanelNotRegistered(id.into()))?;
        if !used.insert(id.to_owned()) {
            return Err(WorkspaceError::PanelAlreadyPlaced(id.into()));
        }
        Ok(key.clone())
    };
    let roots = layout
        .root_panel_ids
        .iter()
        .map(|id| lookup(id))
        .collect::<Result<Vec<_>, _>>()?;
    if roots.is_empty() {
        return Ok(());
    }
    let mut dock = DockState::new(roots);
    for split in &layout.splits {
        if !split.fraction.is_finite()
            || !(0.0..=1.0).contains(&split.fraction)
            || split.panel_ids.is_empty()
        {
            return Err(WorkspaceError::InvalidContainerTarget);
        }
        let anchor = keys
            .get(&split.anchor_panel_id)
            .ok_or_else(|| WorkspaceError::PanelNotRegistered(split.anchor_panel_id.clone()))?;
        let (node, _) = dock
            .main_surface()
            .find_tab(anchor)
            .ok_or(WorkspaceError::InvalidContainerTarget)?;
        let tabs = split
            .panel_ids
            .iter()
            .map(|id| lookup(id))
            .collect::<Result<Vec<_>, _>>()?;
        let tree = dock.main_surface_mut();
        match split.direction {
            crate::layout::SplitDirection::Left => {
                tree.split_left(node, split.fraction, tabs);
            }
            crate::layout::SplitDirection::Right => {
                tree.split_right(node, split.fraction, tabs);
            }
            crate::layout::SplitDirection::Above => {
                tree.split_above(node, split.fraction, tabs);
            }
            crate::layout::SplitDirection::Below => {
                tree.split_below(node, split.fraction, tabs);
            }
        }
    }
    if used.len() != keys.len() {
        return Err(WorkspaceError::InvalidContainerTarget);
    }
    if let Some(selected) = &layout.selected_panel_id {
        let key = keys
            .get(selected)
            .ok_or_else(|| WorkspaceError::PanelNotRegistered(selected.clone()))?;
        let (node, tab) = dock
            .main_surface()
            .find_tab(key)
            .ok_or(WorkspaceError::InvalidContainerTarget)?;
        dock.main_surface_mut()
            .set_active_tab(node, tab)
            .map_err(|_| WorkspaceError::InvalidContainerTarget)?;
    }
    *slot = Some(dock);
    Ok(())
}

/// Resolve the initial legacy application layout into the default Workspace.
/// This is an additive bridge: the existing DockState<String> remains active
/// until the host can render multiple containers.
pub fn import_legacy_root_panels(
    registry: &mut WorkspaceRegistry,
    panel_ids: &[String],
) -> Result<(), WorkspaceError> {
    use crate::workspace::{DEFAULT_CONTAINER_ID, DEFAULT_WORKSPACE_ID};
    // Validate all references before changing any placements.
    let ws = registry
        .get(DEFAULT_WORKSPACE_ID)
        .ok_or_else(|| WorkspaceError::WorkspaceNotFound(DEFAULT_WORKSPACE_ID.into()))?;
    let mut seen = std::collections::BTreeSet::new();
    for id in panel_ids {
        if !seen.insert(id)
            || ws
                .containers
                .values()
                .any(|c| c.panels.iter().any(|p| p.definition_id == *id))
        {
            return Err(WorkspaceError::PanelAlreadyPlaced(id.clone()));
        }
    }
    for id in panel_ids {
        let instance = PanelInstanceId::new(id.clone(), id.clone());
        registry.register_panel(instance.clone());
        registry.place_panel(DEFAULT_WORKSPACE_ID, &instance, DEFAULT_CONTAINER_ID)?;
    }
    Ok(())
}

/// A UI gesture is validated and applied to the authoritative registry.
/// Floating windows are not drop targets.
#[derive(Debug, Clone, PartialEq)]
pub enum DockGesture {
    MoveToNormal {
        panel: PanelInstanceId,
        container: String,
    },
    Float {
        panel: PanelInstanceId,
        geometry: FloatingGeometry,
    },
    Hide {
        panel: PanelInstanceId,
    },
    ShowPrevious {
        panel: PanelInstanceId,
    },
    ShowInNormal {
        panel: PanelInstanceId,
        container: String,
    },
    ResizeFloating {
        container: String,
        geometry: FloatingGeometry,
    },
}

/// Call only after the UI gesture has completed; on error, re-project from
/// the unchanged registry instead of retaining the UI's provisional state.
pub fn apply_dock_gesture(
    registry: &mut WorkspaceRegistry,
    workspace: &str,
    gesture: DockGesture,
) -> Result<(), WorkspaceError> {
    match gesture {
        DockGesture::MoveToNormal { panel, container } => {
            registry.move_panel(workspace, &panel, &container)
        }
        DockGesture::Float { panel, geometry } => registry
            .float_panel(workspace, &panel, geometry)
            .map(|_| ()),
        DockGesture::Hide { panel } => registry.hide_panel(workspace, &panel),
        DockGesture::ShowPrevious { panel } => registry.show_panel(workspace, &panel, None),
        DockGesture::ShowInNormal { panel, container } => {
            registry.show_panel(workspace, &panel, Some(&container))
        }
        DockGesture::ResizeFloating {
            container,
            geometry,
        } => registry.update_floating_geometry(workspace, &container, geometry),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::{
        DEFAULT_CONTAINER_ID, DEFAULT_WORKSPACE_ID, FloatingGeometry, WorkspaceRegistry,
    };

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
        registry
            .place_panel(DEFAULT_WORKSPACE_ID, &panel, DEFAULT_CONTAINER_ID)
            .unwrap();
        let id = registry
            .float_panel(
                DEFAULT_WORKSPACE_ID,
                &panel,
                FloatingGeometry {
                    position: [10.0, 20.0],
                    size: [200.0, 100.0],
                },
            )
            .unwrap();
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
        registry
            .place_panel(DEFAULT_WORKSPACE_ID, &a, DEFAULT_CONTAINER_ID)
            .unwrap();
        registry
            .place_panel(DEFAULT_WORKSPACE_ID, &b, DEFAULT_CONTAINER_ID)
            .unwrap();
        let projected = project_workspace(registry.get(DEFAULT_WORKSPACE_ID).unwrap()).unwrap();
        assert!(projected.normal[DEFAULT_CONTAINER_ID].is_some());
        assert!(projected.floating.is_empty());
    }

    #[test]
    fn gestures_move_panel_and_preserve_authoritative_state_on_error() {
        let mut registry = WorkspaceRegistry::new();
        registry
            .add_container(DEFAULT_WORKSPACE_ID, "right")
            .unwrap();
        let panel = PanelInstanceId::new("editor", "one");
        registry.register_panel(panel.clone());
        registry
            .place_panel(DEFAULT_WORKSPACE_ID, &panel, DEFAULT_CONTAINER_ID)
            .unwrap();
        apply_dock_gesture(
            &mut registry,
            DEFAULT_WORKSPACE_ID,
            DockGesture::MoveToNormal {
                panel: panel.clone(),
                container: "right".into(),
            },
        )
        .unwrap();
        assert_eq!(
            registry.get(DEFAULT_WORKSPACE_ID).unwrap().containers["right"].panels,
            vec![panel.clone()]
        );
        assert_eq!(
            apply_dock_gesture(
                &mut registry,
                DEFAULT_WORKSPACE_ID,
                DockGesture::MoveToNormal {
                    panel: panel.clone(),
                    container: "missing".into()
                }
            ),
            Err(WorkspaceError::ContainerNotFound("missing".into()))
        );
        assert_eq!(
            registry.get(DEFAULT_WORKSPACE_ID).unwrap().containers["right"].panels,
            vec![panel]
        );
    }

    #[test]
    fn gestures_hide_and_restore_floating_panel() {
        let mut registry = WorkspaceRegistry::new();
        let panel = PanelInstanceId::new("editor", "one");
        registry.register_panel(panel.clone());
        registry
            .place_panel(DEFAULT_WORKSPACE_ID, &panel, DEFAULT_CONTAINER_ID)
            .unwrap();
        let geometry = FloatingGeometry {
            position: [11.0, 22.0],
            size: [333.0, 222.0],
        };
        apply_dock_gesture(
            &mut registry,
            DEFAULT_WORKSPACE_ID,
            DockGesture::Float {
                panel: panel.clone(),
                geometry,
            },
        )
        .unwrap();
        apply_dock_gesture(
            &mut registry,
            DEFAULT_WORKSPACE_ID,
            DockGesture::Hide {
                panel: panel.clone(),
            },
        )
        .unwrap();
        apply_dock_gesture(
            &mut registry,
            DEFAULT_WORKSPACE_ID,
            DockGesture::ShowPrevious { panel },
        )
        .unwrap();
        let ws = registry.get(DEFAULT_WORKSPACE_ID).unwrap();
        assert_eq!(ws.floating_geometry.values().next(), Some(&geometry));
    }

    #[test]
    fn legacy_panel_import_preserves_ids() {
        let mut registry = WorkspaceRegistry::new();
        import_legacy_root_panels(&mut registry, &["one".into(), "two".into()]).unwrap();
        let ws = registry.get(DEFAULT_WORKSPACE_ID).unwrap();
        assert_eq!(ws.containers[DEFAULT_CONTAINER_ID].panels.len(), 2);
        assert_eq!(
            ws.containers[DEFAULT_CONTAINER_ID].panels[0].definition_id,
            "one"
        );
    }
    #[test]
    fn legacy_panel_import_rejects_duplicate_without_partial_placement() {
        let mut registry = WorkspaceRegistry::new();
        let result = import_legacy_root_panels(&mut registry, &["one".into(), "one".into()]);
        assert!(matches!(result, Err(WorkspaceError::PanelAlreadyPlaced(_))));
        assert!(
            registry.get(DEFAULT_WORKSPACE_ID).unwrap().containers[DEFAULT_CONTAINER_ID]
                .panels
                .is_empty()
        );
    }
}
