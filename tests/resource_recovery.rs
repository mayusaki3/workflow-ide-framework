use std::{fs,path::PathBuf};
use workflow_ide_framework::{
 project::ProjectContext,
 project_resource::{ResourceReference,ResourceScope},
 resource_journal::ResourceOperationJournal,
 resource_operation::{ResourceOperationItem,ResourceOperationKind,ResourceOperationPlan},
 resource_recovery::{assess_pending_journal,JournalItemRecoveryState},
 resource_state::ResourceRoots,
};
fn p(s:&str)->ResourceReference{ResourceReference::new(ResourceScope::Project,s).unwrap()}
fn setup(name:&str)->(PathBuf,ProjectContext,ResourceRoots){
 let base=std::env::temp_dir().join(format!("wfide-recovery-{name}-{}",std::process::id()));let _=fs::remove_dir_all(&base);
 let project=base.join("project");let resources=project.join("resources");fs::create_dir_all(&resources).unwrap();
 (base,ProjectContext::new(project),ResourceRoots{application:base.join("app"),project:resources})
}
fn save(context:&ProjectContext,kind:ResourceOperationKind,before:Option<ResourceReference>,after:Option<ResourceReference>){
 let plan=ResourceOperationPlan{operation_id:"op".into(),kind,items:vec![ResourceOperationItem{before,after}]};
 ResourceOperationJournal::from_plan(&plan,None).save(context).unwrap();
}
#[test]
fn move_before_only_is_not_started(){
 let (base,c,r)=setup("not-started");fs::write(r.project.join("a"),b"x").unwrap();save(&c,ResourceOperationKind::Move,Some(p("a")),Some(p("b")));
 let a=assess_pending_journal(&c,&r).unwrap().unwrap();assert_eq!(a.item_states,vec![JournalItemRecoveryState::NotStarted]);let _=fs::remove_dir_all(base);
}
#[test]
fn move_after_only_means_filesystem_applied(){
 let (base,c,r)=setup("applied");fs::write(r.project.join("b"),b"x").unwrap();save(&c,ResourceOperationKind::Move,Some(p("a")),Some(p("b")));
 let a=assess_pending_journal(&c,&r).unwrap().unwrap();assert_eq!(a.item_states,vec![JournalItemRecoveryState::FilesystemApplied]);let _=fs::remove_dir_all(base);
}
#[test]
fn move_both_existing_is_conflict(){
 let (base,c,r)=setup("conflict");fs::write(r.project.join("a"),b"x").unwrap();fs::write(r.project.join("b"),b"x").unwrap();save(&c,ResourceOperationKind::Move,Some(p("a")),Some(p("b")));
 let a=assess_pending_journal(&c,&r).unwrap().unwrap();assert!(a.has_conflict());assert_eq!(a.item_states,vec![JournalItemRecoveryState::Conflict]);let _=fs::remove_dir_all(base);
}
#[test]
fn delete_missing_source_means_filesystem_applied(){
 let (base,c,r)=setup("delete");save(&c,ResourceOperationKind::Delete,Some(p("a")),None);
 let a=assess_pending_journal(&c,&r).unwrap().unwrap();assert_eq!(a.item_states,vec![JournalItemRecoveryState::FilesystemApplied]);let _=fs::remove_dir_all(base);
}
#[test]
fn no_journal_returns_none(){
 let (base,c,r)=setup("none");assert!(assess_pending_journal(&c,&r).unwrap().is_none());let _=fs::remove_dir_all(base);
}
