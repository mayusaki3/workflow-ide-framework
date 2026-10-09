//! Read-only projection of Workspace registry state into egui_dock trees.
//! The Workspace registry remains the authoritative placement model.
use std::collections::BTreeMap;

use egui_dock::{DockState, Node};

use crate::framework_settings::{
    StoredDockNode, StoredFloatingGeometry, StoredHiddenPanel, StoredPanelIdentity,
    StoredSplitAxis, StoredWorkspaceContainer, StoredWorkspaceLayout,
};

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

/// Capture the live dock topology as backend-independent project settings.
/// The registry remains authoritative for placement and floating geometry.
pub fn snapshot_workspace_layout(
    workspace: &Workspace,
    projection: &WorkspaceDockProjection,
) -> Result<StoredWorkspaceLayout, WorkspaceError> {
    use std::collections::BTreeSet;

    let mut seen = BTreeSet::new();
    let mut containers = Vec::new();
    for (id, container) in &workspace.containers {
        let floating = container.kind == ContainerKind::Floating;
        let geometry = if floating {
            let value = workspace
                .floating_geometry
                .get(id)
                .ok_or(WorkspaceError::InvalidContainerTarget)?;
            Some(StoredFloatingGeometry {
                position: value.position,
                size: value.size,
            })
        } else {
            None
        };
        let tree = if floating {
            let key = projection
                .floating
                .get(id)
                .ok_or(WorkspaceError::InvalidContainerTarget)?;
            if container.panels.as_slice()
                != [PanelInstanceId::new(&key.definition_id, &key.instance_id)]
            {
                return Err(WorkspaceError::InvalidContainerTarget);
            }
            if !seen.insert(PanelInstanceId::new(&key.definition_id, &key.instance_id)) {
                return Err(WorkspaceError::InvalidContainerTarget);
            }
            Some(StoredDockNode::Tabs {
                panels: vec![StoredPanelIdentity {
                    definition_id: key.definition_id.clone(),
                    instance_id: key.instance_id.clone(),
                }],
                active: 0,
            })
        } else {
            let dock = projection
                .normal
                .get(id)
                .ok_or(WorkspaceError::InvalidContainerTarget)?;
            match dock {
                None => None,
                Some(dock) => {
                    let nodes = dock.main_surface();
                    let mut tabs = Vec::new();
                    let root = snapshot_node(nodes, 0, &mut tabs)?;
                    for panel in &tabs {
                        if !seen.insert(panel.clone()) {
                            return Err(WorkspaceError::InvalidContainerTarget);
                        }
                    }
                    let expected: BTreeSet<_> = container.panels.iter().cloned().collect();
                    if expected != tabs.into_iter().collect() {
                        return Err(WorkspaceError::InvalidContainerTarget);
                    }
                    Some(root)
                }
            }
        };
        containers.push(StoredWorkspaceContainer {
            id: id.clone(),
            floating,
            geometry,
            tree,
        });
    }
    let mut hidden_panels = Vec::new();
    for (panel, previous) in &workspace.hidden_panels {
        if !seen.insert(panel.clone()) {
            return Err(WorkspaceError::InvalidContainerTarget);
        }
        let (normal_container, floating_geometry) = match previous {
            crate::workspace::PreviousPlacement::Normal(id) => (Some(id.clone()), None),
            crate::workspace::PreviousPlacement::Floating(geometry) => (
                None,
                Some(StoredFloatingGeometry {
                    position: geometry.position,
                    size: geometry.size,
                }),
            ),
        };
        hidden_panels.push(StoredHiddenPanel {
            panel: StoredPanelIdentity {
                definition_id: panel.definition_id.clone(),
                instance_id: panel.instance_id.clone(),
            },
            normal_container,
            floating_geometry,
        });
    }
    Ok(StoredWorkspaceLayout {
        containers,
        hidden_panels,
    })
}

fn snapshot_node(
    nodes: &egui_dock::Tree<DockPanelKey>,
    index: usize,
    tabs: &mut Vec<PanelInstanceId>,
) -> Result<StoredDockNode, WorkspaceError> {
    let node = nodes
        .iter()
        .nth(index)
        .ok_or(WorkspaceError::InvalidContainerTarget)?;
    match node {
        Node::Leaf(leaf) => {
            let panels = leaf
                .tabs
                .iter()
                .map(|key| {
                    tabs.push(PanelInstanceId::new(&key.definition_id, &key.instance_id));
                    StoredPanelIdentity {
                        definition_id: key.definition_id.clone(),
                        instance_id: key.instance_id.clone(),
                    }
                })
                .collect();
            Ok(StoredDockNode::Tabs {
                panels,
                active: leaf.active.0,
            })
        }
        Node::Horizontal(parent) | Node::Vertical(parent) => {
            let fraction = parent.fraction;
            if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction) {
                return Err(WorkspaceError::InvalidContainerTarget);
            }
            let axis = if matches!(node, Node::Horizontal(_)) {
                StoredSplitAxis::Horizontal
            } else {
                StoredSplitAxis::Vertical
            };
            Ok(StoredDockNode::Split {
                axis,
                fraction,
                first: Box::new(snapshot_node(nodes, index * 2 + 1, tabs)?),
                second: Box::new(snapshot_node(nodes, index * 2 + 2, tabs)?),
            })
        }
        _ => Err(WorkspaceError::InvalidContainerTarget),
    }
}

/// Reconstruct dock trees from a validated project snapshot without changing
/// the registry or the current projection on failure.
pub fn restore_workspace_projection(
    workspace: &Workspace,
    stored: &StoredWorkspaceLayout,
) -> Result<WorkspaceDockProjection, WorkspaceError> {
    use std::collections::BTreeSet;
    let mut projection = project_workspace(workspace)?;
    let expected_ids: BTreeSet<_> = workspace.containers.keys().cloned().collect();
    let actual_ids: BTreeSet<_> = stored.containers.iter().map(|c| c.id.clone()).collect();
    if expected_ids != actual_ids || stored.containers.len() != expected_ids.len() {
        return Err(WorkspaceError::InvalidContainerTarget);
    }
    let mut observed = BTreeSet::new();
    for container in &stored.containers {
        let source = workspace.containers.get(&container.id)
            .ok_or(WorkspaceError::InvalidContainerTarget)?;
        if container.floating != (source.kind == ContainerKind::Floating) {
            return Err(WorkspaceError::InvalidContainerTarget);
        }
        if container.floating {
            let geometry = container.geometry.as_ref()
                .ok_or(WorkspaceError::InvalidContainerTarget)?;
            let expected = workspace.floating_geometry.get(&container.id)
                .ok_or(WorkspaceError::InvalidContainerTarget)?;
            if geometry.position != expected.position || geometry.size != expected.size {
                return Err(WorkspaceError::InvalidContainerTarget);
            }
            let Some(StoredDockNode::Tabs { panels, active: 0 }) = &container.tree else {
                return Err(WorkspaceError::InvalidContainerTarget);
            };
            if panels.len() != 1 {
                return Err(WorkspaceError::InvalidContainerTarget);
            }
            let identity = PanelInstanceId::new(&panels[0].definition_id, &panels[0].instance_id);
            if source.panels.as_slice() != [identity.clone()] || !observed.insert(identity) {
                return Err(WorkspaceError::InvalidContainerTarget);
            }
        } else {
            if container.geometry.is_some() {
                return Err(WorkspaceError::InvalidContainerTarget);
            }
            let (dock, panels) = match &container.tree {
                None => (None, Vec::new()),
                Some(node) => {
                    let mut panels = Vec::new();
                    let dock = restore_dock_node(node, &mut panels)?;
                    (Some(dock), panels)
                }
            };
            let expected: BTreeSet<_> = source.panels.iter().cloned().collect();
            let actual: BTreeSet<_> = panels.iter().cloned().collect();
            if expected != actual || panels.len() != actual.len() {
                return Err(WorkspaceError::InvalidContainerTarget);
            }
            for panel in panels {
                if !observed.insert(panel) {
                    return Err(WorkspaceError::InvalidContainerTarget);
                }
            }
            projection.normal.insert(container.id.clone(), dock);
        }
    }
    let expected_hidden: BTreeSet<_> = workspace.hidden_panels.keys().cloned().collect();
    let mut actual_hidden = BTreeSet::new();
    for hidden in &stored.hidden_panels {
        let panel = PanelInstanceId::new(&hidden.panel.definition_id, &hidden.panel.instance_id);
        if !actual_hidden.insert(panel.clone()) || observed.contains(&panel) {
            return Err(WorkspaceError::InvalidContainerTarget);
        }
        let Some(previous) = workspace.hidden_panels.get(&panel) else {
            return Err(WorkspaceError::InvalidContainerTarget);
        };
        let matches = match previous {
            crate::workspace::PreviousPlacement::Normal(id) =>
                hidden.normal_container.as_ref() == Some(id) && hidden.floating_geometry.is_none(),
            crate::workspace::PreviousPlacement::Floating(geometry) =>
                hidden.normal_container.is_none() && hidden.floating_geometry.as_ref().is_some_and(|stored|
                    stored.position == geometry.position && stored.size == geometry.size),
        };
        if !matches {
            return Err(WorkspaceError::InvalidContainerTarget);
        }
    }
    if actual_hidden != expected_hidden {
        return Err(WorkspaceError::InvalidContainerTarget);
    }
    Ok(projection)
}

fn restore_dock_node(
    node: &StoredDockNode,
    panels: &mut Vec<PanelInstanceId>,
) -> Result<DockState<DockPanelKey>, WorkspaceError> {
    let mut leaves = Vec::new();
    collect_stored_leaves(node, &mut leaves)?;
    let first = leaves.first().ok_or(WorkspaceError::InvalidContainerTarget)?;
    let mut dock = DockState::new(first.0.clone());
    restore_tree_branch(dock.main_surface_mut(), egui_dock::NodeIndex::root(), node)?;
    for (tabs, _) in leaves {
        panels.extend(tabs.into_iter().map(|key|
            PanelInstanceId::new(key.definition_id, key.instance_id)));
    }
    Ok(dock)
}

fn collect_stored_leaves(
    node: &StoredDockNode,
    leaves: &mut Vec<(Vec<DockPanelKey>, usize)>,
) -> Result<(), WorkspaceError> {
    match node {
        StoredDockNode::Tabs { panels, active } => {
            if panels.is_empty() || *active >= panels.len() {
                return Err(WorkspaceError::InvalidContainerTarget);
            }
            leaves.push((panels.iter().map(|panel| DockPanelKey {
                definition_id: panel.definition_id.clone(),
                instance_id: panel.instance_id.clone(),
            }).collect(), *active));
        }
        StoredDockNode::Split { fraction, first, second, .. } => {
            if !fraction.is_finite() || !(0.0..=1.0).contains(fraction) {
                return Err(WorkspaceError::InvalidContainerTarget);
            }
            collect_stored_leaves(first, leaves)?;
            collect_stored_leaves(second, leaves)?;
        }
    }
    Ok(())
}

fn first_leaf_tabs(node: &StoredDockNode) -> Vec<DockPanelKey> {
    match node {
        StoredDockNode::Tabs { panels, .. } => panels.iter().map(|panel| DockPanelKey {
            definition_id: panel.definition_id.clone(),
            instance_id: panel.instance_id.clone(),
        }).collect(),
        StoredDockNode::Split { first, .. } => first_leaf_tabs(first),
    }
}

fn restore_tree_branch(
    tree: &mut egui_dock::Tree<DockPanelKey>,
    index: egui_dock::NodeIndex,
    node: &StoredDockNode,
) -> Result<(), WorkspaceError> {
    match node {
        StoredDockNode::Tabs { active, .. } => {
            tree.set_active_tab(index, egui_dock::TabIndex(*active))
                .map_err(|_| WorkspaceError::InvalidContainerTarget)
        }
        StoredDockNode::Split { axis, fraction, first, second } => {
            let right = first_leaf_tabs(second);
            match axis {
                StoredSplitAxis::Horizontal => { tree.split_right(index, *fraction, right); }
                StoredSplitAxis::Vertical => { tree.split_below(index, *fraction, right); }
            }
            let left_index = egui_dock::NodeIndex(index.0 * 2 + 1);
            let right_index = egui_dock::NodeIndex(index.0 * 2 + 2);
            restore_tree_branch(tree, left_index, first)?;
            restore_tree_branch(tree, right_index, second)
        }
    }
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
    fn snapshot_captures_live_split_and_selected_tab() {
        let mut registry = WorkspaceRegistry::new();
        let a = PanelInstanceId::new("editor", "a");
        let b = PanelInstanceId::new("editor", "b");
        let c = PanelInstanceId::new("editor", "c");
        for panel in [&a, &b, &c] {
            registry.register_panel(panel.clone());
            registry
                .place_panel(DEFAULT_WORKSPACE_ID, panel, DEFAULT_CONTAINER_ID)
                .unwrap();
        }
        let ws = registry.get(DEFAULT_WORKSPACE_ID).unwrap();
        let mut projection = project_workspace(ws).unwrap();
        let dock = projection.normal.get_mut(DEFAULT_CONTAINER_ID).unwrap().as_mut().unwrap();
        // Move c out of the root leaf before placing it in the new split.
        // split_right adds tabs; it does not remove existing occurrences.
        for node in dock.main_surface_mut().iter_mut() {
            if let Node::Leaf(leaf) = node {
                leaf.tabs.retain(|tab| tab.instance_id != c.instance_id);
            }
        }
        dock.main_surface_mut().split_right(
            egui_dock::NodeIndex::root(),
            0.4,
            vec![DockPanelKey::from(&c)],
        );
        let snapshot = snapshot_workspace_layout(ws, &projection).unwrap();
        let container = snapshot
            .containers
            .iter()
            .find(|container| container.id == DEFAULT_CONTAINER_ID)
            .unwrap();
        match container.tree.as_ref().unwrap() {
            StoredDockNode::Split { fraction, first, second, .. } => {
                assert_eq!(*fraction, 0.4);
                assert!(matches!(first.as_ref(), StoredDockNode::Tabs { .. }));
                assert!(matches!(second.as_ref(), StoredDockNode::Tabs { .. }));
            }
            other => panic!("expected split tree, got {other:?}"),
        }
    }

    #[test]
    fn restore_nested_split_round_trips_snapshot() {
        let mut registry = WorkspaceRegistry::new();
        for id in ["a", "b", "c", "d"] {
            let panel = PanelInstanceId::new("editor", id);
            registry.register_panel(panel.clone());
            registry.place_panel(DEFAULT_WORKSPACE_ID, &panel, DEFAULT_CONTAINER_ID).unwrap();
        }
        let ws = registry.get(DEFAULT_WORKSPACE_ID).unwrap();
        let mut projection = project_workspace(ws).unwrap();
        let dock = projection.normal.get_mut(DEFAULT_CONTAINER_ID).unwrap().as_mut().unwrap();
        let tree = dock.main_surface_mut();
        for node in tree.iter_mut() {
            if let Node::Leaf(leaf) = node {
                leaf.tabs.retain(|key| key.instance_id == "a" || key.instance_id == "b");
            }
        }
        tree.split_right(egui_dock::NodeIndex::root(), 0.4, vec![
            DockPanelKey::from(&PanelInstanceId::new("editor", "c")),
            DockPanelKey::from(&PanelInstanceId::new("editor", "d")),
        ]);
        tree.split_below(egui_dock::NodeIndex(1), 0.3, vec![
            DockPanelKey::from(&PanelInstanceId::new("editor", "b")),
        ]);
        for node in tree.iter_mut() {
            if let Node::Leaf(leaf) = node {
                if leaf.tabs.len() > 1 {
                    leaf.tabs.retain(|key| key.instance_id != "b");
                }
            }
        }
        tree.set_active_tab(egui_dock::NodeIndex(2), egui_dock::TabIndex(1)).unwrap();
        let stored = snapshot_workspace_layout(ws, &projection).unwrap();
        let restored = restore_workspace_projection(ws, &stored).unwrap();
        let again = snapshot_workspace_layout(ws, &restored).unwrap();
        assert_eq!(stored, again);
    }

    #[test]
    fn snapshot_rejects_duplicate_tabs() {
        let mut registry = WorkspaceRegistry::new();
        for id in ["a", "b"] {
            let panel = PanelInstanceId::new("editor", id);
            registry.register_panel(panel.clone());
            registry
                .place_panel(DEFAULT_WORKSPACE_ID, &panel, DEFAULT_CONTAINER_ID)
                .unwrap();
        }
        let ws = registry.get(DEFAULT_WORKSPACE_ID).unwrap();
        let mut projection = project_workspace(ws).unwrap();
        let dock = projection.normal.get_mut(DEFAULT_CONTAINER_ID).unwrap().as_mut().unwrap();
        for node in dock.main_surface_mut().iter_mut() {
            if let Node::Leaf(leaf) = node {
                leaf.tabs[1] = leaf.tabs[0].clone();
            }
        }
        assert!(matches!(
            snapshot_workspace_layout(ws, &projection),
            Err(WorkspaceError::InvalidContainerTarget)
        ));
    }

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

    // WS-015: a user tab reorder is reflected in the authoritative registry.
    #[test]
    fn sync_updates_tab_order_without_replacing_live_tree() {
        let mut registry = WorkspaceRegistry::new();
        let a = PanelInstanceId::new("editor", "a");
        let b = PanelInstanceId::new("editor", "b");
        for panel in [&a, &b] {
            registry.register_panel(panel.clone());
            registry.place_panel(DEFAULT_WORKSPACE_ID, panel, DEFAULT_CONTAINER_ID).unwrap();
        }
        let mut projection = project_workspace(registry.get(DEFAULT_WORKSPACE_ID).unwrap()).unwrap();
        let tree = projection.normal.get_mut(DEFAULT_CONTAINER_ID).unwrap().as_mut().unwrap();
        for node in tree.main_surface_mut().iter_mut() {
            if let egui_dock::Node::Leaf(leaf) = node {
                leaf.tabs.reverse();
            }
        }
        sync_normal_tab_order(&mut registry, DEFAULT_WORKSPACE_ID, &projection).unwrap();
        assert_eq!(registry.get(DEFAULT_WORKSPACE_ID).unwrap().containers[DEFAULT_CONTAINER_ID].panels, vec![b, a]);
    }

    // WS-021: an invalid dock tree cannot partially mutate panel placement.
    #[test]
    fn sync_rejects_duplicate_tab_atomically() {
        let mut registry = WorkspaceRegistry::new();
        let a = PanelInstanceId::new("editor", "a");
        let b = PanelInstanceId::new("editor", "b");
        for panel in [&a, &b] {
            registry.register_panel(panel.clone());
            registry.place_panel(DEFAULT_WORKSPACE_ID, panel, DEFAULT_CONTAINER_ID).unwrap();
        }
        let mut projection = project_workspace(registry.get(DEFAULT_WORKSPACE_ID).unwrap()).unwrap();
        let tree = projection.normal.get_mut(DEFAULT_CONTAINER_ID).unwrap().as_mut().unwrap();
        for node in tree.main_surface_mut().iter_mut() {
            if let egui_dock::Node::Leaf(leaf) = node {
                leaf.tabs[1] = leaf.tabs[0].clone();
            }
        }
        assert_eq!(sync_normal_tab_order(&mut registry, DEFAULT_WORKSPACE_ID, &projection), Err(WorkspaceError::InvalidContainerTarget));
        assert_eq!(registry.get(DEFAULT_WORKSPACE_ID).unwrap().containers[DEFAULT_CONTAINER_ID].panels, vec![a, b]);
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
