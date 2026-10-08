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
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PanelInstanceId {
    pub definition_id: String,
    pub instance_id: String,
}

impl PanelInstanceId {
    pub fn new(definition_id: impl Into<String>, instance_id: impl Into<String>) -> Self {
        Self { definition_id: definition_id.into(), instance_id: instance_id.into() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerKind { Normal, Floating }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    pub id: String,
    pub kind: ContainerKind,
    pub panels: Vec<PanelInstanceId>,
}

#[derive(Debug, Clone)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub containers: BTreeMap<String, Container>,
}

impl Workspace {
    fn new(id: String, name: String) -> Self {
        let mut containers = BTreeMap::new();
        containers.insert(DEFAULT_CONTAINER_ID.into(), Container {
            id: DEFAULT_CONTAINER_ID.into(), kind: ContainerKind::Normal, panels: Vec::new()
        });
        Self { id, name, containers }
    }
}

#[derive(Debug)]
pub struct WorkspaceRegistry {
    workspaces: BTreeMap<String, Workspace>,
    selected: String,
    registered_panels: BTreeSet<PanelInstanceId>,
}

impl Default for WorkspaceRegistry {
    fn default() -> Self { Self::new() }
}

impl WorkspaceRegistry {
    pub fn new() -> Self {
        let mut workspaces = BTreeMap::new();
        workspaces.insert(DEFAULT_WORKSPACE_ID.into(),
            Workspace::new(DEFAULT_WORKSPACE_ID.into(), "Default".into()));
        Self { workspaces, selected: DEFAULT_WORKSPACE_ID.into(), registered_panels: BTreeSet::new() }
    }

    pub fn selected_id(&self) -> &str { &self.selected }
    pub fn get(&self, id: &str) -> Option<&Workspace> { self.workspaces.get(id) }
    pub fn iter(&self) -> impl Iterator<Item = &Workspace> { self.workspaces.values() }

    pub fn register(&mut self, id: impl Into<String>, name: impl Into<String>) -> Result<(), WorkspaceError> {
        let id = id.into();
        if id == DEFAULT_WORKSPACE_ID { return Err(WorkspaceError::ReservedWorkspaceId); }
        if self.workspaces.contains_key(&id) { return Err(WorkspaceError::DuplicateWorkspace(id)); }
        self.workspaces.insert(id.clone(), Workspace::new(id, name.into()));
        Ok(())
    }

    pub fn unregister(&mut self, id: &str) -> Result<(), WorkspaceError> {
        if id == DEFAULT_WORKSPACE_ID { return Err(WorkspaceError::CannotRemoveDefaultWorkspace); }
        if self.workspaces.remove(id).is_none() { return Err(WorkspaceError::WorkspaceNotFound(id.into())); }
        if self.selected == id { self.selected = DEFAULT_WORKSPACE_ID.into(); }
        Ok(())
    }

    pub fn select(&mut self, id: &str) -> Result<(), WorkspaceError> {
        if !self.workspaces.contains_key(id) { return Err(WorkspaceError::WorkspaceNotFound(id.into())); }
        self.selected = id.into();
        Ok(())
    }

    pub fn register_panel(&mut self, panel: PanelInstanceId) -> bool {
        self.registered_panels.insert(panel)
    }

    pub fn add_container(&mut self, workspace: &str, id: impl Into<String>) -> Result<(), WorkspaceError> {
        let ws = self.workspaces.get_mut(workspace).ok_or_else(|| WorkspaceError::WorkspaceNotFound(workspace.into()))?;
        let id = id.into();
        if ws.containers.contains_key(&id) { return Err(WorkspaceError::DuplicateContainer(id)); }
        ws.containers.insert(id.clone(), Container { id, kind: ContainerKind::Normal, panels: Vec::new() });
        Ok(())
    }

    pub fn place_panel(&mut self, workspace: &str, panel: &PanelInstanceId, container: &str) -> Result<(), WorkspaceError> {
        if !self.registered_panels.contains(panel) {
            return Err(WorkspaceError::PanelNotRegistered(panel.definition_id.clone()));
        }
        let ws = self.workspaces.get_mut(workspace).ok_or_else(|| WorkspaceError::WorkspaceNotFound(workspace.into()))?;
        if ws.containers.values().any(|c| c.panels.contains(panel)) {
            return Err(WorkspaceError::PanelAlreadyPlaced(panel.instance_id.clone()));
        }
        let target = ws.containers.get_mut(container).ok_or_else(|| WorkspaceError::ContainerNotFound(container.into()))?;
        if target.kind == ContainerKind::Floating { return Err(WorkspaceError::FloatingContainerRejectsDrop); }
        target.panels.push(panel.clone());
        Ok(())
    }

    pub fn move_panel(&mut self, workspace: &str, panel: &PanelInstanceId, container: &str) -> Result<(), WorkspaceError> {
        let ws = self.workspaces.get_mut(workspace).ok_or_else(|| WorkspaceError::WorkspaceNotFound(workspace.into()))?;
        let target = ws.containers.get(container).ok_or_else(|| WorkspaceError::ContainerNotFound(container.into()))?;
        if target.kind == ContainerKind::Floating { return Err(WorkspaceError::FloatingContainerRejectsDrop); }
        let source = ws.containers.values().find(|c| c.panels.contains(panel)).map(|c| c.id.clone())
            .ok_or_else(|| WorkspaceError::PanelNotPlaced(panel.instance_id.clone()))?;
        if source == container { return Ok(()); }
        ws.containers.get_mut(&source).unwrap().panels.retain(|p| p != panel);
        ws.containers.get_mut(container).unwrap().panels.push(panel.clone());
        if ws.containers.get(&source).is_some_and(|c| c.kind == ContainerKind::Floating && c.panels.is_empty()) {
            ws.containers.remove(&source);
        }
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
        assert_eq!(registry.get(DEFAULT_WORKSPACE_ID).unwrap().containers.len(), 1);
    }
    #[test]
    fn default_workspace_cannot_be_removed() {
        let mut registry = WorkspaceRegistry::new();
        assert_eq!(registry.unregister(DEFAULT_WORKSPACE_ID), Err(WorkspaceError::CannotRemoveDefaultWorkspace));
    }
    #[test]
    fn panel_may_be_shared_between_workspaces_but_not_duplicated() {
        let mut registry = WorkspaceRegistry::new();
        registry.register("second", "Second").unwrap();
        let panel = PanelInstanceId::new("editor", "one");
        registry.register_panel(panel.clone());
        registry.place_panel(DEFAULT_WORKSPACE_ID, &panel, DEFAULT_CONTAINER_ID).unwrap();
        assert!(matches!(registry.place_panel(DEFAULT_WORKSPACE_ID, &panel, DEFAULT_CONTAINER_ID), Err(WorkspaceError::PanelAlreadyPlaced(_))));
        registry.place_panel("second", &panel, DEFAULT_CONTAINER_ID).unwrap();
    }
    #[test]
    fn moving_between_containers_preserves_panel() {
        let mut registry = WorkspaceRegistry::new();
        registry.add_container(DEFAULT_WORKSPACE_ID, "other").unwrap();
        let panel = PanelInstanceId::new("editor", "one");
        registry.register_panel(panel.clone());
        registry.place_panel(DEFAULT_WORKSPACE_ID, &panel, DEFAULT_CONTAINER_ID).unwrap();
        registry.move_panel(DEFAULT_WORKSPACE_ID, &panel, "other").unwrap();
        assert_eq!(registry.get(DEFAULT_WORKSPACE_ID).unwrap().containers["other"].panels, vec![panel]);
    }
}
