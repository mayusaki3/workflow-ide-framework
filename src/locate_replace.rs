use crate::{
    project_resource::{ResourceEntry, ResourceReference},
    resource_registry::{RegisterResult, ResourceRegistry},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameworkResourceUse {
    pub owner_id: String,
    pub reference: ResourceReference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceChange {
    pub before: ResourceReference,
    pub after: ResourceReference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocateReplacePlan {
    pub change: ResourceChange,
    pub affected_owner_ids: Vec<String>,
    pub new_registry_entry: ResourceEntry,
    pub new_registry_entry_added: bool,
}

pub fn locate_replace<F>(
    registry: &mut ResourceRegistry,
    framework_uses: &mut [FrameworkResourceUse],
    old: &ResourceReference,
    new: ResourceReference,
    new_id: F,
) -> LocateReplacePlan
where
    F: FnOnce() -> String,
{
    let (new_registry_entry, new_registry_entry_added) = match registry.find_by_reference(&new).cloned() {
        Some(entry) => (entry, false),
        None => match registry.register(new_id(), new.clone()) {
            RegisterResult::Added(entry) => (entry, true),
            RegisterResult::Existing(entry) => (entry, false),
        },
    };

    let mut affected_owner_ids = Vec::new();
    for usage in framework_uses {
        if usage.reference == *old {
            usage.reference = new.clone();
            affected_owner_ids.push(usage.owner_id.clone());
        }
    }

    LocateReplacePlan {
        change: ResourceChange { before: old.clone(), after: new },
        affected_owner_ids,
        new_registry_entry,
        new_registry_entry_added,
    }
}


pub trait ApplicationResourceChangeAdapter {
    type Error: std::fmt::Display;
    fn prepare_resource_change(
        &mut self,
        change: &ResourceChange,
    ) -> Result<crate::project_resource::ResourceOperationDecision, Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocateReplaceDecision {
    Accepted { journal_data: Option<crate::project_resource::ApplicationJournalData> },
    Rejected { reason: Option<String>, handled: bool },
}

pub fn prepare_locate_replace<A: ApplicationResourceChangeAdapter>(
    application: &mut A,
    old: &ResourceReference,
    new: &ResourceReference,
) -> Result<LocateReplaceDecision, String> {
    let change = ResourceChange { before: old.clone(), after: new.clone() };
    match application.prepare_resource_change(&change).map_err(|error| error.to_string())? {
        crate::project_resource::ResourceOperationDecision::Accept { journal_data } => {
            Ok(LocateReplaceDecision::Accepted { journal_data })
        }
        crate::project_resource::ResourceOperationDecision::Handled { journal_data } => {
            Ok(LocateReplaceDecision::Accepted { journal_data })
        }
        crate::project_resource::ResourceOperationDecision::Reject { reason, handled } => {
            Ok(LocateReplaceDecision::Rejected { reason, handled })
        }
    }
}
