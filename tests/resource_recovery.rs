use std::{fs,path::PathBuf};
use workflow_ide_framework::{
 project::ProjectContext,
 locate_replace::FrameworkResourceUse,
 project_resource::{ResourceReference,ResourceScope},
 resource_journal::{ResourceOperationJournal,StoredExecutionRoute},
 resource_operation::{ResourceOperationItem,ResourceOperationKind,ResourceOperationPlan},
 resource_recovery::{assess_pending_journal,assess_pending_journal_full,assess_pending_journal_with_application,complete_framework_recovery,ApplicationRecoveryState,ApplicationResourceRecoveryAdapter,CombinedRecoveryState,FrameworkPersistentRecoveryState,FrameworkRecoveryResult,JournalItemRecoveryState},
 resource_registry::ResourceRegistry,
 resource_state::ResourceRoots,
};
fn p(s:&str)->ResourceReference{ResourceReference::new(ResourceScope::Project,s).unwrap()}
fn setup(name:&str)->(PathBuf,ProjectContext,ResourceRoots){
 let base=std::env::temp_dir().join(format!("wfide-recovery-{name}-{}",std::process::id()));let _=fs::remove_dir_all(&base);
 let project=base.join("project");let resources=project.join("resources");fs::create_dir_all(&resources).unwrap();
 (base.clone(),ProjectContext::new(project),ResourceRoots{application:base.join("app"),project:resources})
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
struct PanicRecoveryApp;
impl ApplicationResourceRecoveryAdapter for PanicRecoveryApp { type Error=&'static str; fn assess_resource_recovery(&mut self,_:&workflow_ide_framework::resource_journal::ResourceOperationJournal)->Result<ApplicationRecoveryState,Self::Error>{panic!("Application recovery must not be called for Framework route")} }
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

#[test]
fn full_recovery_detects_framework_before_after_filesystem_move(){
 let (base,c,r)=setup("framework-before");fs::write(r.project.join("b"),b"x").unwrap();save(&c,ResourceOperationKind::Move,Some(p("a")),Some(p("b")));
 let mut registry=ResourceRegistry::default();registry.register("stable",p("a"));let uses=vec![FrameworkResourceUse{owner_id:"editor".into(),reference:p("a")}];
 let a=assess_pending_journal_full(&mut RecoveryApp(ApplicationRecoveryState::Applied),&c,&r,&registry,&uses).unwrap().unwrap();
 assert_eq!(a.framework,FrameworkPersistentRecoveryState::Before);assert_eq!(a.state,CombinedRecoveryState::FilesystemApplied);let _=fs::remove_dir_all(base);
}
#[test]
fn full_recovery_detects_fully_applied_move(){
 let (base,c,r)=setup("framework-after");fs::write(r.project.join("b"),b"x").unwrap();save(&c,ResourceOperationKind::Move,Some(p("a")),Some(p("b")));
 let mut registry=ResourceRegistry::default();registry.register("stable",p("b"));let uses=vec![FrameworkResourceUse{owner_id:"editor".into(),reference:p("b")}];
 let a=assess_pending_journal_full(&mut RecoveryApp(ApplicationRecoveryState::Applied),&c,&r,&registry,&uses).unwrap().unwrap();
 assert_eq!(a.framework,FrameworkPersistentRecoveryState::Applied);assert_eq!(a.state,CombinedRecoveryState::Applied);let _=fs::remove_dir_all(base);
}
#[test]
fn full_recovery_detects_mixed_framework_state_as_conflict(){
 let (base,c,r)=setup("framework-mixed");fs::write(r.project.join("b"),b"x").unwrap();save(&c,ResourceOperationKind::Move,Some(p("a")),Some(p("b")));
 let mut registry=ResourceRegistry::default();registry.register("old",p("a"));registry.register("new",p("b"));let uses=vec![];
 let a=assess_pending_journal_full(&mut RecoveryApp(ApplicationRecoveryState::Applied),&c,&r,&registry,&uses).unwrap().unwrap();
 assert_eq!(a.framework,FrameworkPersistentRecoveryState::Mixed);assert_eq!(a.state,CombinedRecoveryState::Conflict);let _=fs::remove_dir_all(base);
}

#[test]
fn move_recovery_restores_stable_id_from_journal_when_old_registry_entry_is_gone(){
 let (base,c,r)=setup("stable-id");fs::write(r.project.join("b"),b"x").unwrap();
 let plan=ResourceOperationPlan{operation_id:"op".into(),kind:ResourceOperationKind::Move,items:vec![ResourceOperationItem{before:Some(p("a")),after:Some(p("b"))}]};
 let mut original=ResourceRegistry::default();original.register("stable-id",p("a"));
 ResourceOperationJournal::from_plan_with_registry(&plan,None,&original).save(&c).unwrap();
 let mut registry=ResourceRegistry::default();let mut uses=vec![FrameworkResourceUse{owner_id:"editor".into(),reference:p("a")}];
 let a=assess_pending_journal_full(&mut RecoveryApp(ApplicationRecoveryState::Applied),&c,&r,&registry,&uses).unwrap().unwrap();
 assert_eq!(complete_framework_recovery(&a,&mut registry,&mut uses,||"wrong-new-id".into()).unwrap(),FrameworkRecoveryResult::Completed);
 assert_eq!(registry.find_by_reference(&p("b")).unwrap().resource_id,"stable-id");assert_eq!(uses[0].reference,p("b"));let _=fs::remove_dir_all(base);
}

#[test]
fn full_recovery_accepts_move_destination_with_matching_journal_id(){
 let (base,c,r)=setup("matching-id");fs::write(r.project.join("b"),b"x").unwrap();
 let plan=ResourceOperationPlan{operation_id:"op".into(),kind:ResourceOperationKind::Move,items:vec![ResourceOperationItem{before:Some(p("a")),after:Some(p("b"))}]};
 let mut original=ResourceRegistry::default();original.register("stable-id",p("a"));ResourceOperationJournal::from_plan_with_registry(&plan,None,&original).save(&c).unwrap();
 let mut registry=ResourceRegistry::default();registry.register("stable-id",p("b"));let uses=vec![FrameworkResourceUse{owner_id:"editor".into(),reference:p("b")}];
 let a=assess_pending_journal_full(&mut RecoveryApp(ApplicationRecoveryState::Applied),&c,&r,&registry,&uses).unwrap().unwrap();
 assert_eq!(a.framework,FrameworkPersistentRecoveryState::Applied);assert_eq!(a.state,CombinedRecoveryState::Applied);let _=fs::remove_dir_all(base);
}
#[test]
fn full_recovery_rejects_move_destination_with_different_journal_id(){
 let (base,c,r)=setup("wrong-id");fs::write(r.project.join("b"),b"x").unwrap();
 let plan=ResourceOperationPlan{operation_id:"op".into(),kind:ResourceOperationKind::Move,items:vec![ResourceOperationItem{before:Some(p("a")),after:Some(p("b"))}]};
 let mut original=ResourceRegistry::default();original.register("stable-id",p("a"));ResourceOperationJournal::from_plan_with_registry(&plan,None,&original).save(&c).unwrap();
 let mut registry=ResourceRegistry::default();registry.register("other-id",p("b"));let uses=vec![FrameworkResourceUse{owner_id:"editor".into(),reference:p("b")}];
 let a=assess_pending_journal_full(&mut RecoveryApp(ApplicationRecoveryState::Applied),&c,&r,&registry,&uses).unwrap().unwrap();
 assert_eq!(a.framework,FrameworkPersistentRecoveryState::Mixed);assert_eq!(a.state,CombinedRecoveryState::Conflict);let _=fs::remove_dir_all(base);
}
#[test]
fn move_recovery_does_not_overwrite_destination_if_id_changes_after_assessment(){
 let (base,c,r)=setup("id-race");fs::write(r.project.join("b"),b"x").unwrap();
 let plan=ResourceOperationPlan{operation_id:"op".into(),kind:ResourceOperationKind::Move,items:vec![ResourceOperationItem{before:Some(p("a")),after:Some(p("b"))}]};
 let mut original=ResourceRegistry::default();original.register("stable-id",p("a"));ResourceOperationJournal::from_plan_with_registry(&plan,None,&original).save(&c).unwrap();
 let registry=ResourceRegistry::default();let uses=vec![FrameworkResourceUse{owner_id:"editor".into(),reference:p("a")}];
 let a=assess_pending_journal_full(&mut RecoveryApp(ApplicationRecoveryState::Applied),&c,&r,&registry,&uses).unwrap().unwrap();
 let mut changed_registry=ResourceRegistry::default();changed_registry.register("other-id",p("b"));let mut changed_uses=uses;
 assert!(complete_framework_recovery(&a,&mut changed_registry,&mut changed_uses,||"unused".into()).is_err());
 assert_eq!(changed_registry.find_by_reference(&p("b")).unwrap().resource_id,"other-id");assert_eq!(changed_uses[0].reference,p("a"));let _=fs::remove_dir_all(base);
}

#[test]
fn import_recovery_restores_preallocated_target_resource_id(){
 let (base,c,r)=setup("import-target-id");let external=base.join("source.txt");fs::write(&external,b"x").unwrap();fs::write(r.project.join("imported.txt"),b"x").unwrap();
 let source=ResourceReference::new(ResourceScope::External,external).unwrap();let plan=ResourceOperationPlan{operation_id:"op".into(),kind:ResourceOperationKind::Import,items:vec![ResourceOperationItem{before:Some(source.clone()),after:Some(p("imported.txt"))}]};
 let original=ResourceRegistry::default();let mut journal=ResourceOperationJournal::from_plan_with_registry(&plan,None,&original);journal.set_after_resource_ids(&[Some("target-id".into())]);journal.save(&c).unwrap();
 let mut registry=ResourceRegistry::default();let mut uses=vec![FrameworkResourceUse{owner_id:"editor".into(),reference:source}];
 let a=assess_pending_journal_full(&mut RecoveryApp(ApplicationRecoveryState::Applied),&c,&r,&registry,&uses).unwrap().unwrap();
 assert_eq!(complete_framework_recovery(&a,&mut registry,&mut uses,||"wrong-id".into()).unwrap(),FrameworkRecoveryResult::Completed);
 assert_eq!(registry.find_by_reference(&p("imported.txt")).unwrap().resource_id,"target-id");assert_eq!(uses[0].reference,p("imported.txt"));let _=fs::remove_dir_all(base);
}

#[test]
fn framework_route_recovery_does_not_call_application_assessment(){
 let (base,c,r)=setup("framework-route");fs::write(r.project.join("b"),b"x").unwrap();
 let plan=ResourceOperationPlan{operation_id:"op".into(),kind:ResourceOperationKind::Move,items:vec![ResourceOperationItem{before:Some(p("a")),after:Some(p("b"))}]};
 let mut journal=ResourceOperationJournal::from_plan(&plan,None);journal.set_execution_route(StoredExecutionRoute::Framework);journal.save(&c).unwrap();
 let mut registry=ResourceRegistry::default();registry.register("stable",p("b"));let uses=vec![FrameworkResourceUse{owner_id:"editor".into(),reference:p("b")}];
 let a=assess_pending_journal_full(&mut PanicRecoveryApp,&c,&r,&registry,&uses).unwrap().unwrap();
 assert_eq!(a.application,ApplicationRecoveryState::NotApplicable);assert_eq!(a.state,CombinedRecoveryState::Applied);let _=fs::remove_dir_all(base);
}
#[test]
fn legacy_journal_without_route_defaults_to_application_recovery(){
 let (base,c,r)=setup("legacy-route");fs::write(r.project.join("b"),b"x").unwrap();
 let plan=ResourceOperationPlan{operation_id:"op".into(),kind:ResourceOperationKind::Move,items:vec![ResourceOperationItem{before:Some(p("a")),after:Some(p("b"))}]};
 ResourceOperationJournal::from_plan(&plan,None).save(&c).unwrap();
 let a=assess_pending_journal_with_application(&mut RecoveryApp(ApplicationRecoveryState::Applied),&c,&r).unwrap().unwrap();
 assert_eq!(a.application,ApplicationRecoveryState::Applied);assert_eq!(a.state,CombinedRecoveryState::Applied);let _=fs::remove_dir_all(base);
}
