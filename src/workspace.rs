//! Workspace and container registry, independent of the rendering backend.
use std::collections::{BTreeMap, BTreeSet};

pub const DEFAULT_WORKSPACE_ID: &str = "framework.workspace.default";
pub const DEFAULT_CONTAINER_ID: &str = "framework.container.default";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceError {
    DuplicateWorkspace(String),
    ReservedWorkspaceId,
    WorkspaceNotFound(String),
    CannotRemoveDefaultWorkspace,
    DuplicateContainer(String),
    ContainerNotFound(String),
    PanelNotRegistered(String),
    PanelAlreadyPlaced(String),
    PanelNotPlaced(String),
    FloatingContainerRejectsDrop,
    InvalidContainerTarget,
    InvalidFloatingGeometry,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PanelInstanceId {
    pub definition_id: String,
    pub instance_id: String,
}

impl PanelInstanceId {
    pub fn new(definition_id: impl Into<String>, instance_id: impl Into<String>) -> Self {
        Self {
            definition_id: definition_id.into(),
            instance_id: instance_id.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerKind {
    Normal,
    Floating,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    pub id: String,
    pub kind: ContainerKind,
    pub panels: Vec<PanelInstanceId>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatingGeometry {
    pub position: [f32; 2],
    pub size: [f32; 2],
}

impl FloatingGeometry {
    fn valid(self) -> bool {
        self.position.iter().all(|v| v.is_finite())
            && self.size.iter().all(|v| v.is_finite() && *v > 0.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PreviousPlacement {
    Normal(String),
    Floating(FloatingGeometry),
}

#[derive(Debug, Clone)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub containers: BTreeMap<String, Container>,
    pub floating_geometry: BTreeMap<String, FloatingGeometry>,
    pub hidden_panels: BTreeMap<PanelInstanceId, PreviousPlacement>,
    next_floating_id: u64,
}

impl Workspace {
    fn new(id: String, name: String) -> Self {
        let mut containers = BTreeMap::new();
        containers.insert(
            DEFAULT_CONTAINER_ID.into(),
            Container {
                id: DEFAULT_CONTAINER_ID.into(),
                kind: ContainerKind::Normal,
                panels: Vec::new(),
            },
        );
        Self {
            id,
            name,
            containers,
            floating_geometry: BTreeMap::new(),
            hidden_panels: BTreeMap::new(),
            next_floating_id: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkspaceRegistry {
    workspaces: BTreeMap<String, Workspace>,
    selected: String,
    registered_panels: BTreeSet<PanelInstanceId>,
}

impl Default for WorkspaceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceRegistry {
    pub fn new() -> Self {
        let mut workspaces = BTreeMap::new();
        workspaces.insert(
            DEFAULT_WORKSPACE_ID.into(),
            Workspace::new(DEFAULT_WORKSPACE_ID.into(), "Default".into()),
        );
        Self {
            workspaces,
            selected: DEFAULT_WORKSPACE_ID.into(),
            registered_panels: BTreeSet::new(),
        }
    }

    pub fn selected_id(&self) -> &str {
        &self.selected
    }
    pub fn get(&self, id: &str) -> Option<&Workspace> {
        self.workspaces.get(id)
    }
    /// Internal mutable access for validated dock-state reconciliation.
    pub(crate) fn get_mut_workspace_for_dock(
        &mut self,
        id: &str,
    ) -> Result<&mut Workspace, WorkspaceError> {
        self.workspaces
            .get_mut(id)
            .ok_or_else(|| WorkspaceError::WorkspaceNotFound(id.into()))
    }
    pub fn iter(&self) -> impl Iterator<Item = &Workspace> {
        self.workspaces.values()
    }

    pub fn register(
        &mut self,
        id: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<(), WorkspaceError> {
        let id = id.into();
        if id == DEFAULT_WORKSPACE_ID {
            return Err(WorkspaceError::ReservedWorkspaceId);
        }
        if self.workspaces.contains_key(&id) {
            return Err(WorkspaceError::DuplicateWorkspace(id));
        }
        self.workspaces
            .insert(id.clone(), Workspace::new(id, name.into()));
        Ok(())
    }

    pub fn unregister(&mut self, id: &str) -> Result<(), WorkspaceError> {
        if id == DEFAULT_WORKSPACE_ID {
            return Err(WorkspaceError::CannotRemoveDefaultWorkspace);
        }
        if self.workspaces.remove(id).is_none() {
            return Err(WorkspaceError::WorkspaceNotFound(id.into()));
        }
        if self.selected == id {
            self.selected = DEFAULT_WORKSPACE_ID.into();
        }
        Ok(())
    }

    pub fn select(&mut self, id: &str) -> Result<(), WorkspaceError> {
        if !self.workspaces.contains_key(id) {
            return Err(WorkspaceError::WorkspaceNotFound(id.into()));
        }
        self.selected = id.into();
        Ok(())
    }

    /// Check whether a saved panel identity is known to the host application.
    /// Replace one workspace only after its restored layout has been validated.
    pub(crate) fn replace_workspace_for_dock(
        &mut self,
        id: &str,
        mut workspace: Workspace,
    ) -> Result<(), WorkspaceError> {
        if !self.workspaces.contains_key(id) || workspace.id != id {
            return Err(WorkspaceError::WorkspaceNotFound(id.into()));
        }
        workspace.next_floating_id = workspace.containers.keys()
            .filter_map(|key| key.strip_prefix("framework.container.floating."))
            .filter_map(|suffix| suffix.parse::<u64>().ok())
            .max()
            .unwrap_or(0);
        self.workspaces.insert(id.into(), workspace);
        Ok(())
    }

    pub(crate) fn is_registered_panel_for_dock(&self, panel: &PanelInstanceId) -> bool {
        self.registered_panels.contains(panel)
    }

    pub fn register_panel(&mut self, panel: PanelInstanceId) -> bool {
        self.registered_panels.insert(panel)
    }

    pub fn add_container(
        &mut self,
        workspace: &str,
        id: impl Into<String>,
    ) -> Result<(), WorkspaceError> {
        let ws = self
            .workspaces
            .get_mut(workspace)
            .ok_or_else(|| WorkspaceError::WorkspaceNotFound(workspace.into()))?;
        let id = id.into();
        if ws.containers.contains_key(&id) {
            return Err(WorkspaceError::DuplicateContainer(id));
        }
        ws.containers.insert(
            id.clone(),
            Container {
                id,
                kind: ContainerKind::Normal,
                panels: Vec::new(),
            },
        );
        Ok(())
    }

    pub fn place_panel(
        &mut self,
        workspace: &str,
        panel: &PanelInstanceId,
        container: &str,
    ) -> Result<(), WorkspaceError> {
        if !self.registered_panels.contains(panel) {
            return Err(WorkspaceError::PanelNotRegistered(
                panel.definition_id.clone(),
            ));
        }
        let ws = self
            .workspaces
            .get_mut(workspace)
            .ok_or_else(|| WorkspaceError::WorkspaceNotFound(workspace.into()))?;
        if ws.containers.values().any(|c| c.panels.contains(panel)) {
            return Err(WorkspaceError::PanelAlreadyPlaced(
                panel.instance_id.clone(),
            ));
        }
        let target = ws
            .containers
            .get_mut(container)
            .ok_or_else(|| WorkspaceError::ContainerNotFound(container.into()))?;
        if target.kind == ContainerKind::Floating {
            return Err(WorkspaceError::FloatingContainerRejectsDrop);
        }
        target.panels.push(panel.clone());
        ws.hidden_panels.remove(panel);
        Ok(())
    }

    pub fn move_panel(
        &mut self,
        workspace: &str,
        panel: &PanelInstanceId,
        container: &str,
    ) -> Result<(), WorkspaceError> {
        let ws = self
            .workspaces
            .get_mut(workspace)
            .ok_or_else(|| WorkspaceError::WorkspaceNotFound(workspace.into()))?;
        let target = ws
            .containers
            .get(container)
            .ok_or_else(|| WorkspaceError::ContainerNotFound(container.into()))?;
        if target.kind == ContainerKind::Floating {
            return Err(WorkspaceError::FloatingContainerRejectsDrop);
        }
        let source = ws
            .containers
            .values()
            .find(|c| c.panels.contains(panel))
            .map(|c| c.id.clone())
            .ok_or_else(|| WorkspaceError::PanelNotPlaced(panel.instance_id.clone()))?;
        if source == container {
            return Ok(());
        }
        ws.containers
            .get_mut(&source)
            .unwrap()
            .panels
            .retain(|p| p != panel);
        ws.containers
            .get_mut(container)
            .unwrap()
            .panels
            .push(panel.clone());
        ws.hidden_panels.remove(panel);
        if ws
            .containers
            .get(&source)
            .is_some_and(|c| c.kind == ContainerKind::Floating && c.panels.is_empty())
        {
            ws.containers.remove(&source);
            ws.floating_geometry.remove(&source);
        }
        Ok(())
    }

    /// Create a dedicated floating container for an already placed panel.
    pub fn float_panel(
        &mut self,
        workspace: &str,
        panel: &PanelInstanceId,
        geometry: FloatingGeometry,
    ) -> Result<String, WorkspaceError> {
        if !geometry.valid() {
            return Err(WorkspaceError::InvalidFloatingGeometry);
        }
        let ws = self
            .workspaces
            .get_mut(workspace)
            .ok_or_else(|| WorkspaceError::WorkspaceNotFound(workspace.into()))?;
        let source = ws
            .containers
            .values()
            .find(|c| c.panels.contains(panel))
            .map(|c| c.id.clone())
            .ok_or_else(|| WorkspaceError::PanelNotPlaced(panel.instance_id.clone()))?;
        ws.next_floating_id += 1;
        let id = format!("framework.container.floating.{}", ws.next_floating_id);
        ws.containers
            .get_mut(&source)
            .unwrap()
            .panels
            .retain(|p| p != panel);
        if ws
            .containers
            .get(&source)
            .is_some_and(|c| c.kind == ContainerKind::Floating && c.panels.is_empty())
        {
            ws.containers.remove(&source);
            ws.floating_geometry.remove(&source);
        }
        ws.containers.insert(
            id.clone(),
            Container {
                id: id.clone(),
                kind: ContainerKind::Floating,
                panels: vec![panel.clone()],
            },
        );
        ws.floating_geometry.insert(id.clone(), geometry);
        ws.hidden_panels.remove(panel);
        Ok(id)
    }

    /// Hide a panel without destroying its instance or previous placement.
    pub fn hide_panel(
        &mut self,
        workspace: &str,
        panel: &PanelInstanceId,
    ) -> Result<(), WorkspaceError> {
        let ws = self
            .workspaces
            .get_mut(workspace)
            .ok_or_else(|| WorkspaceError::WorkspaceNotFound(workspace.into()))?;
        let source = ws
            .containers
            .values()
            .find(|c| c.panels.contains(panel))
            .map(|c| c.id.clone())
            .ok_or_else(|| WorkspaceError::PanelNotPlaced(panel.instance_id.clone()))?;
        let previous = if ws.containers[&source].kind == ContainerKind::Floating {
            PreviousPlacement::Floating(ws.floating_geometry[&source])
        } else {
            PreviousPlacement::Normal(source.clone())
        };
        ws.containers
            .get_mut(&source)
            .unwrap()
            .panels
            .retain(|p| p != panel);
        if matches!(previous, PreviousPlacement::Floating(_)) {
            ws.containers.remove(&source);
            ws.floating_geometry.remove(&source);
        }
        ws.hidden_panels.insert(panel.clone(), previous);
        Ok(())
    }

    /// Restore a hidden panel to its last placement, or to the default container.
    pub fn show_panel(
        &mut self,
        workspace: &str,
        panel: &PanelInstanceId,
        target: Option<&str>,
    ) -> Result<(), WorkspaceError> {
        if !self.registered_panels.contains(panel) {
            return Err(WorkspaceError::PanelNotRegistered(
                panel.definition_id.clone(),
            ));
        }
        let ws = self
            .workspaces
            .get(workspace)
            .ok_or_else(|| WorkspaceError::WorkspaceNotFound(workspace.into()))?;
        if ws.containers.values().any(|c| c.panels.contains(panel)) {
            return Err(WorkspaceError::PanelAlreadyPlaced(
                panel.instance_id.clone(),
            ));
        }
        if let Some(target) = target {
            // An invalid explicit target must fail instead of silently falling back.
            return self.place_panel(workspace, panel, target);
        }
        let previous = ws.hidden_panels.get(panel).cloned();
        match previous {
            Some(PreviousPlacement::Floating(geometry)) => {
                self.place_panel(workspace, panel, DEFAULT_CONTAINER_ID)?;
                self.float_panel(workspace, panel, geometry)?;
                Ok(())
            }
            Some(PreviousPlacement::Normal(id))
                if ws
                    .containers
                    .get(&id)
                    .is_some_and(|c| c.kind == ContainerKind::Normal) =>
            {
                self.place_panel(workspace, panel, &id)
            }
            _ => self.place_panel(workspace, panel, DEFAULT_CONTAINER_ID),
        }
    }

    pub fn update_floating_geometry(
        &mut self,
        workspace: &str,
        container: &str,
        geometry: FloatingGeometry,
    ) -> Result<(), WorkspaceError> {
        if !geometry.valid() {
            return Err(WorkspaceError::InvalidFloatingGeometry);
        }
        let ws = self
            .workspaces
            .get_mut(workspace)
            .ok_or_else(|| WorkspaceError::WorkspaceNotFound(workspace.into()))?;
        if !ws
            .containers
            .get(container)
            .is_some_and(|c| c.kind == ContainerKind::Floating)
        {
            return Err(WorkspaceError::InvalidContainerTarget);
        }
        ws.floating_geometry.insert(container.into(), geometry);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_workspace_and_container_exist() {
        let registry = WorkspaceRegistry::new();
        assert_eq!(registry.selected_id(), DEFAULT_WORKSPACE_ID);
        assert_eq!(
            registry.get(DEFAULT_WORKSPACE_ID).unwrap().containers.len(),
            1
        );
    }
    #[test]
    fn default_workspace_cannot_be_removed() {
        let mut registry = WorkspaceRegistry::new();
        assert_eq!(
            registry.unregister(DEFAULT_WORKSPACE_ID),
            Err(WorkspaceError::CannotRemoveDefaultWorkspace)
        );
    }
    #[test]
    fn panel_may_be_shared_between_workspaces_but_not_duplicated() {
        let mut registry = WorkspaceRegistry::new();
        registry.register("second", "Second").unwrap();
        let panel = PanelInstanceId::new("editor", "one");
        registry.register_panel(panel.clone());
        registry
            .place_panel(DEFAULT_WORKSPACE_ID, &panel, DEFAULT_CONTAINER_ID)
            .unwrap();
        assert!(matches!(
            registry.place_panel(DEFAULT_WORKSPACE_ID, &panel, DEFAULT_CONTAINER_ID),
            Err(WorkspaceError::PanelAlreadyPlaced(_))
        ));
        registry
            .place_panel("second", &panel, DEFAULT_CONTAINER_ID)
            .unwrap();
    }
    #[test]
    fn moving_between_containers_preserves_panel() {
        let mut registry = WorkspaceRegistry::new();
        registry
            .add_container(DEFAULT_WORKSPACE_ID, "other")
            .unwrap();
        let panel = PanelInstanceId::new("editor", "one");
        registry.register_panel(panel.clone());
        registry
            .place_panel(DEFAULT_WORKSPACE_ID, &panel, DEFAULT_CONTAINER_ID)
            .unwrap();
        registry
            .move_panel(DEFAULT_WORKSPACE_ID, &panel, "other")
            .unwrap();
        assert_eq!(
            registry.get(DEFAULT_WORKSPACE_ID).unwrap().containers["other"].panels,
            vec![panel]
        );
    }

    #[test]
    fn floating_container_rejects_other_panel_drop() {
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
        let floating = registry
            .float_panel(
                DEFAULT_WORKSPACE_ID,
                &a,
                FloatingGeometry {
                    position: [20.0, 30.0],
                    size: [400.0, 300.0],
                },
            )
            .unwrap();
        assert_eq!(
            registry.move_panel(DEFAULT_WORKSPACE_ID, &b, &floating),
            Err(WorkspaceError::FloatingContainerRejectsDrop)
        );
        assert_eq!(
            registry.get(DEFAULT_WORKSPACE_ID).unwrap().containers[&floating].panels,
            vec![a]
        );
    }
    #[test]
    fn hidden_floating_panel_restores_geometry() {
        let mut registry = WorkspaceRegistry::new();
        let panel = PanelInstanceId::new("editor", "one");
        registry.register_panel(panel.clone());
        registry
            .place_panel(DEFAULT_WORKSPACE_ID, &panel, DEFAULT_CONTAINER_ID)
            .unwrap();
        let geometry = FloatingGeometry {
            position: [20.0, 30.0],
            size: [400.0, 300.0],
        };
        let floating = registry
            .float_panel(DEFAULT_WORKSPACE_ID, &panel, geometry)
            .unwrap();
        registry.hide_panel(DEFAULT_WORKSPACE_ID, &panel).unwrap();
        assert!(
            !registry
                .get(DEFAULT_WORKSPACE_ID)
                .unwrap()
                .containers
                .contains_key(&floating)
        );
        registry
            .show_panel(DEFAULT_WORKSPACE_ID, &panel, None)
            .unwrap();
        let ws = registry.get(DEFAULT_WORKSPACE_ID).unwrap();
        assert_eq!(ws.floating_geometry.values().next(), Some(&geometry));
        assert_eq!(
            ws.containers
                .values()
                .filter(|c| c.kind == ContainerKind::Floating)
                .count(),
            1
        );
    }
    #[test]
    fn explicit_container_overrides_floating_history() {
        let mut registry = WorkspaceRegistry::new();
        let panel = PanelInstanceId::new("editor", "one");
        registry.register_panel(panel.clone());
        registry
            .place_panel(DEFAULT_WORKSPACE_ID, &panel, DEFAULT_CONTAINER_ID)
            .unwrap();
        registry
            .float_panel(
                DEFAULT_WORKSPACE_ID,
                &panel,
                FloatingGeometry {
                    position: [1.0, 2.0],
                    size: [50.0, 60.0],
                },
            )
            .unwrap();
        registry.hide_panel(DEFAULT_WORKSPACE_ID, &panel).unwrap();
        registry
            .show_panel(DEFAULT_WORKSPACE_ID, &panel, Some(DEFAULT_CONTAINER_ID))
            .unwrap();
        let ws = registry.get(DEFAULT_WORKSPACE_ID).unwrap();
        assert_eq!(ws.containers[DEFAULT_CONTAINER_ID].panels, vec![panel]);
        assert!(ws.floating_geometry.is_empty());
    }
}
