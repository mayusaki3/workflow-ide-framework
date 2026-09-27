use crate::{
    locate_replace::FrameworkResourceUse,
    project::ProjectContext,
    project_resource::{ApplicationJournalData, ResourceReference, ResourceScope},
    resource_journal::ResourceOperationJournal,
    resource_operation::{ResourceOperationKind, ResourceOperationPlan},
    resource_registry::{RegisterResult, ResourceRegistry},
    resource_state::ResourceRoots,
};
use std::{fs, io, path::{Path, PathBuf}};

#[derive(Debug)]
pub enum ResourceExecutionError {
    Validation(String),
    JournalWrite(io::Error),
    Filesystem(io::Error),
    JournalRemove(io::Error),
    MissingSource(PathBuf),
    DestinationExists(PathBuf),
}

pub fn execute_operation<F>(
    context: &ProjectContext,
    roots: &ResourceRoots,
    registry: &mut ResourceRegistry,
    framework_uses: &mut [FrameworkResourceUse],
    plan: &ResourceOperationPlan,
    application_journal_data: Option<ApplicationJournalData>,
    mut new_id: F,
) -> Result<(), ResourceExecutionError>
where
    F: FnMut() -> String,
{
    plan.validate().map_err(|error| ResourceExecutionError::Validation(format!("{error:?}")))?;

    let journal = ResourceOperationJournal::from_plan(plan, application_journal_data);
    journal.save(context).map_err(ResourceExecutionError::JournalWrite)?;

    for item in &plan.items {
        let result = match plan.kind {
            ResourceOperationKind::Import => {
                let before=item.before.as_ref().expect("validated");
                let after=item.after.as_ref().expect("validated");
                copy_file(roots,before,after)
            }
            ResourceOperationKind::Rename|ResourceOperationKind::Move => {
                let before=item.before.as_ref().expect("validated");
                let after=item.after.as_ref().expect("validated");
                move_file(roots,before,after)
            }
            ResourceOperationKind::Delete => {
                let before=item.before.as_ref().expect("validated");
                delete_file(roots,before)
            }
            ResourceOperationKind::Replace => Ok(()),
        };
        if let Err(error)=result { return Err(error); }
    }

    apply_framework_state(registry,framework_uses,plan,&mut new_id);

    ResourceOperationJournal::remove(context).map_err(ResourceExecutionError::JournalRemove)?;
    Ok(())
}

fn copy_file(roots:&ResourceRoots,before:&ResourceReference,after:&ResourceReference)->Result<(),ResourceExecutionError>{
    let source=roots.resolve(before); let destination=roots.resolve(after);
    require_source_file(&source)?; require_free_destination(&destination)?;
    if let Some(parent)=destination.parent(){fs::create_dir_all(parent).map_err(ResourceExecutionError::Filesystem)?;}
    fs::copy(source,destination).map_err(ResourceExecutionError::Filesystem)?;
    Ok(())
}

fn move_file(roots:&ResourceRoots,before:&ResourceReference,after:&ResourceReference)->Result<(),ResourceExecutionError>{
    let source=roots.resolve(before); let destination=roots.resolve(after);
    require_source_file(&source)?; require_free_destination(&destination)?;
    if let Some(parent)=destination.parent(){fs::create_dir_all(parent).map_err(ResourceExecutionError::Filesystem)?;}
    fs::rename(source,destination).map_err(ResourceExecutionError::Filesystem)
}

fn delete_file(roots:&ResourceRoots,before:&ResourceReference)->Result<(),ResourceExecutionError>{
    let source=roots.resolve(before); require_source_file(&source)?;
    fs::remove_file(source).map_err(ResourceExecutionError::Filesystem)
}

fn require_source_file(path:&Path)->Result<(),ResourceExecutionError>{
    match fs::metadata(path){
        Ok(metadata) if metadata.is_file()=>Ok(()),
        _=>Err(ResourceExecutionError::MissingSource(path.to_path_buf())),
    }
}
fn require_free_destination(path:&Path)->Result<(),ResourceExecutionError>{
    if path.exists(){Err(ResourceExecutionError::DestinationExists(path.to_path_buf()))}else{Ok(())}
}

fn apply_framework_state<F:FnMut()->String>(
    registry:&mut ResourceRegistry,
    framework_uses:&mut [FrameworkResourceUse],
    plan:&ResourceOperationPlan,
    new_id:&mut F,
){
    for item in &plan.items {
        match plan.kind {
            ResourceOperationKind::Import=>{
                let after=item.after.as_ref().expect("validated");
                if registry.find_by_reference(after).is_none(){
                    let _=registry.register(new_id(),after.clone());
                }
            }
            ResourceOperationKind::Rename|ResourceOperationKind::Move=>{
                let before=item.before.as_ref().expect("validated");
                let after=item.after.as_ref().expect("validated");
                if let Some(entry)=registry.find_by_reference(before).cloned(){
                    let _=registry.remove(&entry.resource_id);
                    match registry.register(entry.resource_id,after.clone()){RegisterResult::Added(_)|RegisterResult::Existing(_)=>{}}
                }
                replace_uses(framework_uses,before,after);
            }
            ResourceOperationKind::Delete=>{
                let before=item.before.as_ref().expect("validated");
                if let Some(entry)=registry.find_by_reference(before).cloned(){let _=registry.remove(&entry.resource_id);}
                // Framework references intentionally remain and therefore become Missing.
            }
            ResourceOperationKind::Replace=>{
                let before=item.before.as_ref().expect("validated");
                let after=item.after.as_ref().expect("validated");
                if registry.find_by_reference(after).is_none(){let _=registry.register(new_id(),after.clone());}
                replace_uses(framework_uses,before,after);
            }
        }
    }
}
fn replace_uses(uses:&mut [FrameworkResourceUse],before:&ResourceReference,after:&ResourceReference){
    for usage in uses {if usage.reference==*before{usage.reference=after.clone();}}
}
