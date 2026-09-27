use crate::{
    project::ProjectContext,
    project_resource::{ResourceReference,ResourceScope},
    resource_journal::{ResourceOperationJournal,StoredJournalItem,StoredOperationKind,StoredReference},
    resource_state::ResourceRoots,
};
use std::{fs,io};

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum JournalItemRecoveryState { NotStarted, FilesystemApplied, AlreadyComplete, Conflict, Indeterminate }

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct JournalRecoveryAssessment {
    pub journal:ResourceOperationJournal,
    pub item_states:Vec<JournalItemRecoveryState>,
}
impl JournalRecoveryAssessment {
    pub fn has_conflict(&self)->bool{self.item_states.iter().any(|s|matches!(s,JournalItemRecoveryState::Conflict|JournalItemRecoveryState::Indeterminate))}
    pub fn all_not_started(&self)->bool{self.item_states.iter().all(|s|*s==JournalItemRecoveryState::NotStarted)}
}

pub fn assess_pending_journal(context:&ProjectContext,roots:&ResourceRoots)->io::Result<Option<JournalRecoveryAssessment>>{
    let Some(journal)=ResourceOperationJournal::load(context)? else{return Ok(None)};
    let states=journal.items.iter().map(|item|assess_item(journal.kind,item,roots)).collect();
    Ok(Some(JournalRecoveryAssessment{journal,item_states:states}))
}

fn assess_item(kind:StoredOperationKind,item:&StoredJournalItem,roots:&ResourceRoots)->JournalItemRecoveryState{
    let before=item.before.as_ref().and_then(to_reference).map(|r|exists_file(roots,&r));
    let after=item.after.as_ref().and_then(to_reference).map(|r|exists_file(roots,&r));
    match kind {
        StoredOperationKind::Rename|StoredOperationKind::Move => match (before,after) {
            (Some(true),Some(false))=>JournalItemRecoveryState::NotStarted,
            (Some(false),Some(true))=>JournalItemRecoveryState::FilesystemApplied,
            (Some(true),Some(true))=>JournalItemRecoveryState::Conflict,
            (Some(false),Some(false))=>JournalItemRecoveryState::Indeterminate,
            _=>JournalItemRecoveryState::Indeterminate,
        },
        StoredOperationKind::Import|StoredOperationKind::Export => match (before,after) {
            (Some(true),Some(false))=>JournalItemRecoveryState::NotStarted,
            (Some(true),Some(true))=>JournalItemRecoveryState::FilesystemApplied,
            (Some(false),Some(true))=>JournalItemRecoveryState::FilesystemApplied,
            (Some(false),Some(false))=>JournalItemRecoveryState::Indeterminate,
            _=>JournalItemRecoveryState::Indeterminate,
        },
        StoredOperationKind::Delete => match before {
            Some(true)=>JournalItemRecoveryState::NotStarted,
            Some(false)=>JournalItemRecoveryState::FilesystemApplied,
            None=>JournalItemRecoveryState::Indeterminate,
        },
        StoredOperationKind::Replace => JournalItemRecoveryState::AlreadyComplete,
    }
}
fn exists_file(roots:&ResourceRoots,r:&ResourceReference)->bool{fs::metadata(roots.resolve(r)).map(|m|m.is_file()).unwrap_or(false)}
fn to_reference(stored:&StoredReference)->Option<ResourceReference>{
    let scope=match stored.scope.as_str(){"application"=>ResourceScope::Application,"project"=>ResourceScope::Project,"external"=>ResourceScope::External,_=>return None};
    ResourceReference::new(scope,stored.path.clone()).ok()
}
