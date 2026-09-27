use crate::{
    locate_replace::ResourceChange,
    project_resource::{ApplicationJournalData, ResourceOperationDecision, ResourceReference, ResourceScope},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceOperationKind { Import, Export, Rename, Move, Delete, Replace }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceOperationItem {
    pub before: Option<ResourceReference>,
    pub after: Option<ResourceReference>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceOperationPlan {
    pub operation_id: String,
    pub kind: ResourceOperationKind,
    pub items: Vec<ResourceOperationItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceOperationValidationError {
    Empty,
    RenameRequiresOneItem,
    DeleteRequiresOneItem,
    ProjectScopeRequired,
    ExportDestinationMustBeExternal,
    MissingBefore,
    MissingAfter,
    SameSourceAndDestination,
}

impl ResourceOperationPlan {
    pub fn validate(&self) -> Result<(), ResourceOperationValidationError> {
        if self.items.is_empty() { return Err(ResourceOperationValidationError::Empty); }
        if self.kind == ResourceOperationKind::Rename && self.items.len()!=1 { return Err(ResourceOperationValidationError::RenameRequiresOneItem); }
        if self.kind == ResourceOperationKind::Delete && self.items.len()!=1 { return Err(ResourceOperationValidationError::DeleteRequiresOneItem); }
        for item in &self.items {
            match self.kind {
                ResourceOperationKind::Import => {
                    let before=item.before.as_ref().ok_or(ResourceOperationValidationError::MissingBefore)?;
                    let after=item.after.as_ref().ok_or(ResourceOperationValidationError::MissingAfter)?;
                    if after.scope!=ResourceScope::Project { return Err(ResourceOperationValidationError::ProjectScopeRequired); }
                    if before==after { return Err(ResourceOperationValidationError::SameSourceAndDestination); }
                }
                ResourceOperationKind::Export => {
                    let before=item.before.as_ref().ok_or(ResourceOperationValidationError::MissingBefore)?;
                    let after=item.after.as_ref().ok_or(ResourceOperationValidationError::MissingAfter)?;
                    if after.scope!=ResourceScope::External { return Err(ResourceOperationValidationError::ExportDestinationMustBeExternal); }
                    if before==after { return Err(ResourceOperationValidationError::SameSourceAndDestination); }
                }
                ResourceOperationKind::Rename|ResourceOperationKind::Move => {
                    let before=item.before.as_ref().ok_or(ResourceOperationValidationError::MissingBefore)?;
                    let after=item.after.as_ref().ok_or(ResourceOperationValidationError::MissingAfter)?;
                    if before.scope!=ResourceScope::Project || after.scope!=ResourceScope::Project { return Err(ResourceOperationValidationError::ProjectScopeRequired); }
                    if before==after { return Err(ResourceOperationValidationError::SameSourceAndDestination); }
                }
                ResourceOperationKind::Delete => {
                    let before=item.before.as_ref().ok_or(ResourceOperationValidationError::MissingBefore)?;
                    if before.scope!=ResourceScope::Project { return Err(ResourceOperationValidationError::ProjectScopeRequired); }
                }
                ResourceOperationKind::Replace => {
                    item.before.as_ref().ok_or(ResourceOperationValidationError::MissingBefore)?;
                    item.after.as_ref().ok_or(ResourceOperationValidationError::MissingAfter)?;
                }
            }
        }
        Ok(())
    }

    pub fn changes(&self) -> Vec<ResourceChange> {
        self.items.iter().filter_map(|item| Some(ResourceChange {
            before:item.before.clone()?,
            after:item.after.clone()?,
        })).collect()
    }
}

pub trait ApplicationResourceOperationAdapter {
    type Error: std::fmt::Display;
    fn prepare_resource_operation(&mut self, plan: &ResourceOperationPlan) -> Result<ResourceOperationDecision,Self::Error>;
    fn execute_resource_operation(&mut self, _plan: &ResourceOperationPlan, _journal_data: Option<&ApplicationJournalData>) -> Result<(),Self::Error> { Ok(()) }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreparedResourceOperation {
    Accepted { journal_data: Option<ApplicationJournalData> },
    Handled { journal_data: Option<ApplicationJournalData> },
    Rejected { reason: Option<String>, handled: bool },
}

pub fn prepare_operation<A:ApplicationResourceOperationAdapter>(application:&mut A,plan:&ResourceOperationPlan)->Result<PreparedResourceOperation,String>{
    plan.validate().map_err(|e|format!("{e:?}"))?;
    match application.prepare_resource_operation(plan).map_err(|e|e.to_string())? {
        ResourceOperationDecision::Accept{journal_data}=>Ok(PreparedResourceOperation::Accepted{journal_data}),
        ResourceOperationDecision::Handled{journal_data}=>Ok(PreparedResourceOperation::Handled{journal_data}),
        ResourceOperationDecision::Reject{reason,handled}=>Ok(PreparedResourceOperation::Rejected{reason,handled}),
    }
}
