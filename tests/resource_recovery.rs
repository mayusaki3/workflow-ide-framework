use std::{fs,path::PathBuf};
use workflow_ide_framework::{
 project::ProjectContext,
 locate_replace::FrameworkResourceUse,
 project_resource::{ResourceReference,ResourceScope},
 resource_journal::ResourceOperationJournal,
 resource_operation::{ResourceOperationItem,ResourceOperationKind,ResourceOperationPlan},
 resource_recovery::{assess_pending_journal,assess_pending_journal_with_application,complete_framework_recovery,ApplicationRecoveryState,ApplicationResourceRecoveryAdapter,CombinedRecoveryState,FrameworkRecoveryResult,JournalItemRecoveryState},
 resource_registry::ResourceRegistry,
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

struct RecoveryApp(ApplicationRecoveryState);
impl ApplicationResourceRecoveryAdapter for RecoveryApp { type Error=&'static str; fn assess_resource_recovery(&mut self,_:&workflow_ide_framework::resource_journal::ResourceOperationJournal)->Result<ApplicationRecoveryState,Self::Error>{Ok(self.0)} }
#[test]
fn combined_recovery_reports_fully_applied(){ let (base,c,r)=setup("combined");fs::write(r.project.join("b"),b"x").unwrap();save(&c,ResourceOperationKind::Move,Some(p("a")),Some(p("b")));let a=assess_pending_journal_with_application(&mut RecoveryApp(ApplicationRecoveryState::Applied),&c,&r).unwrap().unwrap();assert_eq!(a.state,CombinedRecoveryState::Applied);let _=fs::remove_dir_all(base);}
#[test]
fn combined_recovery_reports_partial_filesystem_state(){ let (base,c,r)=setup("partial");fs::write(r.project.join("b"),b"x").unwrap();save(&c,ResourceOperationKind::Move,Some(p("a")),Some(p("b")));let a=assess_pending_journal_with_application(&mut RecoveryApp(ApplicationRecoveryState::NotStarted),&c,&r).unwrap().unwrap();assert_eq!(a.state,CombinedRecoveryState::FilesystemApplied);let _=fs::remove_dir_all(base);}

#[test]
fn recovery_completes_framework_state_after_move_filesystem_applied(){
 let (base,c,r)=setup("complete-move");fs::write(r.project.join("b"),b"x").unwrap();save(&c,ResourceOperationKind::Move,Some(p("a")),Some(p("b")));
 let a=assess_pending_journal_with_application(&mut RecoveryApp(ApplicationRecoveryState::NotStarted),&c,&r).unwrap().unwrap();
 let mut registry=ResourceRegistry::default();registry.register("stable",p("a"));let mut uses=vec![FrameworkResourceUse{owner_id:"editor".into(),reference:p("a")}];
 assert_eq!(complete_framework_recovery(&a,&mut registry,&mut uses,||"unused".into()).unwrap(),FrameworkRecoveryResult::Completed);
 assert_eq!(registry.find_by_id("stable").unwrap().reference,p("b"));assert_eq!(uses[0].reference,p("b"));let _=fs::remove_dir_all(base);
}
#[test]
fn conflict_cannot_mutate_framework_state(){
 let (base,c,r)=setup("no-complete-conflict");fs::write(r.project.join("a"),b"x").unwrap();fs::write(r.project.join("b"),b"x").unwrap();save(&c,ResourceOperationKind::Move,Some(p("a")),Some(p("b")));
 let a=assess_pending_journal_with_application(&mut RecoveryApp(ApplicationRecoveryState::NotStarted),&c,&r).unwrap().unwrap();
 let mut registry=ResourceRegistry::default();registry.register("stable",p("a"));let mut uses=vec![FrameworkResourceUse{owner_id:"editor".into(),reference:p("a")}];
 assert!(complete_framework_recovery(&a,&mut registry,&mut uses,||"unused".into()).is_err());assert_eq!(registry.find_by_id("stable").unwrap().reference,p("a"));let _=fs::remove_dir_all(base);
}
