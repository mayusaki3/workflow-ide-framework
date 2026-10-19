//! Preflight for physical file deletion; confirmation is always required.
use crate::project_resource::ResourceReference;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeleteReferences {
    Known(usize),
    Unknown,
}

pub trait ApplicationDeleteReferenceInspector {
    type Error;
    fn count_resource_references(
        &mut self,
        resource_id: &str,
    ) -> Result<Option<usize>, Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteConfirmation {
    pub resource_id: String,
    pub reference: ResourceReference,
    pub references: DeleteReferences,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteDecision {
    Confirm,
    Cancel,
}

/// Read-only preflight. Missing hooks, unavailable counts and errors are Unknown.
pub fn inspect_delete<I: ApplicationDeleteReferenceInspector>(
    inspector: Option<&mut I>,
    resource_id: &str,
    reference: &ResourceReference,
) -> DeleteConfirmation {
    let references = match inspector.and_then(|app| app.count_resource_references(resource_id).ok())
    {
        Some(Some(count)) => DeleteReferences::Known(count),
        _ => DeleteReferences::Unknown,
    };
    DeleteConfirmation {
        resource_id: resource_id.to_owned(),
        reference: reference.clone(),
        references,
    }
}

/// Only an explicit user confirmation authorizes the existing operation pipeline.
pub fn authorize_delete(_request: &DeleteConfirmation, decision: DeleteDecision) -> bool {
    matches!(decision, DeleteDecision::Confirm)
}
