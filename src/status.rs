//! Lightweight application-level status items.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusItem {
    pub id: String,
    pub label: String,
    pub value: String,
    pub visible: bool,
}

impl StatusItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>, value: impl Into<String>) -> Self {
        Self { id: id.into(), label: label.into(), value: value.into(), visible: true }
    }

    pub fn visible(mut self, visible: bool) -> Self { self.visible = visible; self }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusRegistryError {
    DuplicateId(String),
    NotFound(String),
}

#[derive(Debug, Default, Clone)]
pub struct StatusRegistry {
    items: HashMap<String, StatusItem>,
    order: Vec<String>,
}

impl StatusRegistry {
    pub fn register(&mut self, item: StatusItem) -> Result<(), StatusRegistryError> {
        if self.items.contains_key(&item.id) { return Err(StatusRegistryError::DuplicateId(item.id)); }
        self.order.push(item.id.clone());
        self.items.insert(item.id.clone(), item);
        Ok(())
    }
    pub fn set_value(&mut self, id: &str, value: impl Into<String>) -> Result<(), StatusRegistryError> {
        self.items.get_mut(id).ok_or_else(|| StatusRegistryError::NotFound(id.to_owned()))?.value = value.into();
        Ok(())
    }
    pub fn set_visible(&mut self, id: &str, visible: bool) -> Result<(), StatusRegistryError> {
        self.items.get_mut(id).ok_or_else(|| StatusRegistryError::NotFound(id.to_owned()))?.visible = visible;
        Ok(())
    }
    pub fn get(&self, id: &str) -> Option<&StatusItem> { self.items.get(id) }
    pub fn iter(&self) -> impl Iterator<Item=&StatusItem> {
        self.order.iter().filter_map(|id| self.items.get(id))
    }
}
