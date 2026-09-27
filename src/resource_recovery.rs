use crate::{
    project::ProjectContext,
    locate_replace::FrameworkResourceUse,
    project_resource::{ResourceReference,ResourceScope},
    resource_journal::{ResourceOperationJournal,StoredJournalItem,StoredOperationKind,StoredReference},
    resource_registry::ResourceRegistry,
    resource_state::ResourceRoots,
};
use std::{fs,io};

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum ApplicationRecoveryState { NotStarted, Applied, Conflict, Indeterminate }

pub trait ApplicationResourceRecoveryAdapter {
    type Error: std::fmt::Display;
    fn assess_resource_recovery(&mut self, journal:&ResourceOperationJournal)->Result<ApplicationRecoveryState,Self::Error>;
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum FrameworkPersistentRecoveryState { Before, Applied, Mixed, NotApplicable, Indeterminate }

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum CombinedRecoveryState { NotStarted, FilesystemApplied, ApplicationApplied, Applied, Conflict, Indeterminate }


#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum JournalItemRecoveryState { NotStarted, FilesystemApplied, AlreadyComplete, Conflict, Indeterminate }

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct JournalRecoveryAssessment {
    pub journal:ResourceOperationJournal,
    pub item_states:Vec<JournalItemRecoveryState>,
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct CombinedRecoveryAssessment {
    pub filesystem:JournalRecoveryAssessment,
    pub framework:FrameworkPersistentRecoveryState,
    pub application:ApplicationRecoveryState,
    pub state:CombinedRecoveryState,
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


pub fn assess_pending_journal_with_application<A:ApplicationResourceRecoveryAdapter>(
    application:&mut A,context:&ProjectContext,roots:&ResourceRoots,
)->Result<Option<CombinedRecoveryAssessment>,String>{
    let empty_registry=ResourceRegistry::default();
    assess_pending_journal_full(application,context,roots,&empty_registry,&[])
}

pub fn assess_pending_journal_full<A:ApplicationResourceRecoveryAdapter>(
    application:&mut A,context:&ProjectContext,roots:&ResourceRoots,registry:&ResourceRegistry,framework_uses:&[FrameworkResourceUse],
)->Result<Option<CombinedRecoveryAssessment>,String>{
    let Some(filesystem)=assess_pending_journal(context,roots).map_err(|e|e.to_string())? else{return Ok(None)};
    let framework=assess_framework_state(&filesystem.journal,registry,framework_uses);
    let application_state=application.assess_resource_recovery(&filesystem.journal).map_err(|e|e.to_string())?;
    let state=combine_recovery_full(&filesystem,framework,application_state);
    Ok(Some(CombinedRecoveryAssessment{filesystem,framework,application:application_state,state}))
}

fn combine_recovery(fs:&JournalRecoveryAssessment,app:ApplicationRecoveryState)->CombinedRecoveryState{
    if fs.item_states.iter().any(|s|*s==JournalItemRecoveryState::Conflict) || app==ApplicationRecoveryState::Conflict{return CombinedRecoveryState::Conflict;}
    if fs.item_states.iter().any(|s|*s==JournalItemRecoveryState::Indeterminate) || app==ApplicationRecoveryState::Indeterminate{return CombinedRecoveryState::Indeterminate;}
    let fs_not_started=fs.item_states.iter().all(|s|*s==JournalItemRecoveryState::NotStarted);
    let fs_applied=fs.item_states.iter().all(|s|matches!(s,JournalItemRecoveryState::FilesystemApplied|JournalItemRecoveryState::AlreadyComplete));
    match (fs_not_started,fs_applied,app) {
        (true,_,ApplicationRecoveryState::NotStarted)=>CombinedRecoveryState::NotStarted,
        (true,_,ApplicationRecoveryState::Applied)=>CombinedRecoveryState::ApplicationApplied,
        (_,true,ApplicationRecoveryState::NotStarted)=>CombinedRecoveryState::FilesystemApplied,
        (_,true,ApplicationRecoveryState::Applied)=>CombinedRecoveryState::Applied,
        _=>CombinedRecoveryState::Indeterminate,
    }
}

fn assess_framework_state(journal:&ResourceOperationJournal,registry:&ResourceRegistry,uses:&[FrameworkResourceUse])->FrameworkPersistentRecoveryState{
    if journal.kind==StoredOperationKind::Export{return FrameworkPersistentRecoveryState::NotApplicable;}
    let mut before_count=0usize;let mut after_count=0usize;let mut mixed=false;
    for item in &journal.items {
        let before=item.before.as_ref().and_then(to_reference);let after=item.after.as_ref().and_then(to_reference);
        match journal.kind {
            StoredOperationKind::Import|StoredOperationKind::Replace=>{let (Some(b),Some(a))=(before,after) else{return FrameworkPersistentRecoveryState::Indeterminate};let a_reg=registry.find_by_reference(&a).is_some();let b_use=uses.iter().any(|u|u.reference==b);let a_use=uses.iter().any(|u|u.reference==a);if a_reg||a_use{after_count+=1;}if b_use&&!a_use{before_count+=1;}if b_use&&a_use{mixed=true;}},
            StoredOperationKind::Rename|StoredOperationKind::Move=>{let (Some(b),Some(a))=(before,after) else{return FrameworkPersistentRecoveryState::Indeterminate};let b_reg=registry.find_by_reference(&b).is_some();let a_reg=registry.find_by_reference(&a).is_some();let b_use=uses.iter().any(|u|u.reference==b);let a_use=uses.iter().any(|u|u.reference==a);if b_reg||b_use{before_count+=1;}if a_reg||a_use{after_count+=1;}if (b_reg&&a_reg)||(b_use&&a_use){mixed=true;}},
            StoredOperationKind::Delete=>{let Some(b)=before else{return FrameworkPersistentRecoveryState::Indeterminate};if registry.find_by_reference(&b).is_some(){before_count+=1}else{after_count+=1;}},
            StoredOperationKind::Export=>{},
        }
    }
    if mixed||(before_count>0&&after_count>0){FrameworkPersistentRecoveryState::Mixed}else if after_count>0{FrameworkPersistentRecoveryState::Applied}else{FrameworkPersistentRecoveryState::Before}
}

fn combine_recovery_full(fs:&JournalRecoveryAssessment,framework:FrameworkPersistentRecoveryState,app:ApplicationRecoveryState)->CombinedRecoveryState{
    if matches!(framework,FrameworkPersistentRecoveryState::Mixed|FrameworkPersistentRecoveryState::Indeterminate){return CombinedRecoveryState::Conflict;}
    let base=combine_recovery(fs,app);match (base,framework){
        (CombinedRecoveryState::Applied,FrameworkPersistentRecoveryState::Applied|FrameworkPersistentRecoveryState::NotApplicable)=>CombinedRecoveryState::Applied,
        (CombinedRecoveryState::Applied,FrameworkPersistentRecoveryState::Before)=>CombinedRecoveryState::FilesystemApplied,
        (other,_)=>other,
    }
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum FrameworkRecoveryResult { Completed, NoChange }

pub fn complete_framework_recovery<F:FnMut()->String>(
    assessment:&CombinedRecoveryAssessment,
    registry:&mut ResourceRegistry,
    framework_uses:&mut [FrameworkResourceUse],
    mut new_id:F,
)->Result<FrameworkRecoveryResult,String>{
    if matches!(assessment.state,CombinedRecoveryState::Conflict|CombinedRecoveryState::Indeterminate|CombinedRecoveryState::NotStarted|CombinedRecoveryState::ApplicationApplied){
        return Err(format!("recovery state {:?} is not safe for framework completion",assessment.state));
    }
    let mut changed=false;
    for item in &assessment.filesystem.journal.items {
        let before=item.before.as_ref().and_then(to_reference);
        let after=item.after.as_ref().and_then(to_reference);
        match assessment.filesystem.journal.kind {
            StoredOperationKind::Import=>{
                let before=before.ok_or("invalid import before reference")?;let after=after.ok_or("invalid import after reference")?;
                if registry.find_by_reference(&after).is_none(){let _=registry.register(new_id(),after.clone());changed=true;}
                changed|=replace_framework_uses(framework_uses,&before,&after)>0;
            }
            StoredOperationKind::Export=>{}
            StoredOperationKind::Rename|StoredOperationKind::Move=>{
                let before=before.ok_or("invalid move before reference")?;let after=after.ok_or("invalid move after reference")?;
                if let Some(entry)=registry.find_by_reference(&before).cloned(){let _=registry.remove(&entry.resource_id);let _=registry.register(entry.resource_id,after.clone());changed=true;}
                changed|=replace_framework_uses(framework_uses,&before,&after)>0;
            }
            StoredOperationKind::Delete=>{
                let before=before.ok_or("invalid delete before reference")?;
                if let Some(entry)=registry.find_by_reference(&before).cloned(){let _=registry.remove(&entry.resource_id);changed=true;}
            }
            StoredOperationKind::Replace=>{
                let before=before.ok_or("invalid replace before reference")?;let after=after.ok_or("invalid replace after reference")?;
                if registry.find_by_reference(&after).is_none(){let _=registry.register(new_id(),after.clone());changed=true;}
                changed|=replace_framework_uses(framework_uses,&before,&after)>0;
            }
        }
    }
    Ok(if changed{FrameworkRecoveryResult::Completed}else{FrameworkRecoveryResult::NoChange})
}

pub fn finalize_recovered_journal(context:&ProjectContext,assessment:&CombinedRecoveryAssessment)->Result<(),String>{
    if assessment.state!=CombinedRecoveryState::Applied{return Err("journal can only be finalized when recovery is fully applied".into());}
    ResourceOperationJournal::remove(context).map_err(|e|e.to_string())
}

fn replace_framework_uses(uses:&mut [FrameworkResourceUse],before:&ResourceReference,after:&ResourceReference)->usize{
    let mut count=0;for usage in uses{if usage.reference==*before{usage.reference=after.clone();count+=1;}}count
}
