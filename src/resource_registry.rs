use crate::project_resource::{ResourceEntry, ResourceReference};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResourceRegistry {
    entries: Vec<ResourceEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisterResult {
    Added(ResourceEntry),
    Existing(ResourceEntry),
}

impl ResourceRegistry {
    pub fn new(entries: Vec<ResourceEntry>) -> Self {
        let mut registry = Self::default();
        for entry in entries {
            if registry.find_by_reference(&entry.reference).is_none() {
                registry.entries.push(entry);
            }
        }
        registry
    }

    pub fn entries(&self) -> &[ResourceEntry] {
        &self.entries
    }

    pub fn register(
        &mut self,
        resource_id: impl Into<String>,
        reference: ResourceReference,
    ) -> RegisterResult {
        if let Some(existing) = self.find_by_reference(&reference) {
            return RegisterResult::Existing(existing.clone());
        }
        let entry = ResourceEntry {
            resource_id: resource_id.into(),
            reference,
        };
        self.entries.push(entry.clone());
        RegisterResult::Added(entry)
    }

    pub fn find_by_reference(&self, reference: &ResourceReference) -> Option<&ResourceEntry> {
        self.entries
            .iter()
            .find(|entry| entry.reference == *reference)
    }

    pub fn find_by_id(&self, resource_id: &str) -> Option<&ResourceEntry> {
        self.entries
            .iter()
            .find(|entry| entry.resource_id == resource_id)
    }

    pub fn remove(&mut self, resource_id: &str) -> Option<ResourceEntry> {
        let index = self
            .entries
            .iter()
            .position(|entry| entry.resource_id == resource_id)?;
        Some(self.entries.remove(index))
    }

    pub fn replace_reference(&mut self, old: &ResourceReference, new: ResourceReference) -> usize {
        let mut replaced = 0;
        for entry in &mut self.entries {
            if entry.reference == *old {
                entry.reference = new.clone();
                replaced += 1;
            }
        }
        replaced
    }
}
