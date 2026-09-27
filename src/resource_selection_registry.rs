use crate::{
    file_resource_selector::FileSelectionResult,
    project_resource::ResourceEntry,
    resource_registry::{RegisterResult, ResourceRegistry},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationBatch {
    pub entries: Vec<ResourceEntry>,
    pub added_count: usize,
}

pub fn register_selection<F>(
    registry: &mut ResourceRegistry,
    selection: FileSelectionResult,
    mut new_id: F,
) -> FileSelectionRegistration
where
    F: FnMut() -> String,
{
    let references = match selection {
        FileSelectionResult::Selected(references) => references,
        FileSelectionResult::Cancelled => return FileSelectionRegistration::Cancelled,
        FileSelectionResult::Error(error) => return FileSelectionRegistration::Error(error),
    };

    let mut entries = Vec::with_capacity(references.len());
    let mut added_count = 0;
    for reference in references {
        let result = registry.register(new_id(), reference);
        match result {
            RegisterResult::Added(entry) => {
                added_count += 1;
                entries.push(entry);
            }
            RegisterResult::Existing(entry) => entries.push(entry),
        }
    }
    FileSelectionRegistration::Registered(RegistrationBatch { entries, added_count })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileSelectionRegistration {
    Registered(RegistrationBatch),
    Cancelled,
    Error(crate::file_resource_selector::FileSelectionError),
}
