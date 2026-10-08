//! Lightweight application-level status items.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusItem {
    pub id: String,
    pub label: String,
    pub value: String,
    pub visible: bool,
}

impl StatusItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            value: value.into(),
            visible: true,
        }
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusRegistryError {
    DuplicateId(String),
    NotFound(String),
    Unavailable,
}

#[derive(Debug, Default, Clone)]
pub struct StatusRegistry {
    items: HashMap<String, StatusItem>,
    order: Vec<String>,
}

impl StatusRegistry {
    pub fn register(&mut self, item: StatusItem) -> Result<(), StatusRegistryError> {
        if self.items.contains_key(&item.id) {
            return Err(StatusRegistryError::DuplicateId(item.id));
        }
        self.order.push(item.id.clone());
        self.items.insert(item.id.clone(), item);
        Ok(())
    }
    pub fn set_value(
        &mut self,
        id: &str,
        value: impl Into<String>,
    ) -> Result<(), StatusRegistryError> {
        self.items
            .get_mut(id)
            .ok_or_else(|| StatusRegistryError::NotFound(id.to_owned()))?
            .value = value.into();
        Ok(())
    }
    pub fn set_visible(&mut self, id: &str, visible: bool) -> Result<(), StatusRegistryError> {
        self.items
            .get_mut(id)
            .ok_or_else(|| StatusRegistryError::NotFound(id.to_owned()))?
            .visible = visible;
        Ok(())
    }
    pub fn get(&self, id: &str) -> Option<&StatusItem> {
        self.items.get(id)
    }
    pub fn iter(&self) -> impl Iterator<Item = &StatusItem> {
        self.order.iter().filter_map(|id| self.items.get(id))
    }
}

/// Cloneable Consumer handle. The Framework and Consumer share the same
/// registry so status can be updated after `Application::run()` starts.
#[derive(Debug, Clone, Default)]
pub struct StatusHandle {
    registry: Arc<Mutex<StatusRegistry>>,
}

impl StatusHandle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, item: StatusItem) -> Result<(), StatusRegistryError> {
        self.registry
            .lock()
            .map_err(|_| StatusRegistryError::Unavailable)?
            .register(item)
    }

    pub fn set_value(&self, id: &str, value: impl Into<String>) -> Result<(), StatusRegistryError> {
        self.registry
            .lock()
            .map_err(|_| StatusRegistryError::Unavailable)?
            .set_value(id, value)
    }

    pub fn set_visible(&self, id: &str, visible: bool) -> Result<(), StatusRegistryError> {
        self.registry
            .lock()
            .map_err(|_| StatusRegistryError::Unavailable)?
            .set_visible(id, visible)
    }

    pub fn snapshot(&self) -> Result<Vec<StatusItem>, StatusRegistryError> {
        let registry = self
            .registry
            .lock()
            .map_err(|_| StatusRegistryError::Unavailable)?;
        Ok(registry.iter().cloned().collect())
    }
}
