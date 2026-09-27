use std::{fs,path::PathBuf};
use workflow_ide_framework::{
 locate_replace::FrameworkResourceUse,
 project::ProjectContext,
 project_resource::{ResourceReference,ResourceScope},
 resource_executor::execute_operation,
 resource_journal::journal_path,
 resource_operation::{ResourceOperationItem,ResourceOperationKind,ResourceOperationPlan},
 resource_registry::ResourceRegistry,
 resource_state::ResourceRoots,
};
fn setup(name:&str)->(PathBuf,ProjectContext,ResourceRoots){
 let base=std::env::temp_dir().join(format!("wfide-executor-{name}-{}",std::process::id()));let _=fs::remove_dir_all(&base);
 let project=base.join("project");let resources=project.join("resources");let app=base.join("app");fs::create_dir_all(&resources).unwrap();fs::create_dir_all(&app).unwrap();
 (base,ProjectContext::new(project),ResourceRoots{application:app,project:resources})
}
fn p(s:&str)->ResourceReference{ResourceReference::new(ResourceScope::Project,s).unwrap()}
#[test]
fn move_updates_file_registry_and_framework_references_then_clears_journal(){
 let (base,context,roots)=setup("move");fs::write(roots.project.join("a.txt"),b"x").unwrap();
 let mut registry=ResourceRegistry::default();registry.register("stable",p("a.txt"));
 let mut uses=vec![FrameworkResourceUse{owner_id:"editor".into(),reference:p("a.txt")}];
 let plan=ResourceOperationPlan{operation_id:"move-1".into(),kind:ResourceOperationKind::Move,items:vec![ResourceOperationItem{before:Some(p("a.txt")),after:Some(p("folder/a.txt"))}]};
 execute_operation(&context,&roots,&mut registry,&mut uses,&plan,None,||"unused".into()).unwrap();
 assert!(!roots.project.join("a.txt").exists());assert!(roots.project.join("folder/a.txt").is_file());
 assert_eq!(registry.find_by_id("stable").unwrap().reference,p("folder/a.txt"));assert_eq!(uses[0].reference,p("folder/a.txt"));
 assert!(!journal_path(&context).exists());let _=fs::remove_dir_all(base);
}
#[test]
fn delete_removes_file_and_registry_but_keeps_framework_reference_missing(){
 let (base,context,roots)=setup("delete");fs::write(roots.project.join("a.txt"),b"x").unwrap();
 let mut registry=ResourceRegistry::default();registry.register("stable",p("a.txt"));
 let mut uses=vec![FrameworkResourceUse{owner_id:"editor".into(),reference:p("a.txt")}];
 let plan=ResourceOperationPlan{operation_id:"delete-1".into(),kind:ResourceOperationKind::Delete,items:vec![ResourceOperationItem{before:Some(p("a.txt")),after:None}]};
 execute_operation(&context,&roots,&mut registry,&mut uses,&plan,None,||"unused".into()).unwrap();
 assert!(!roots.project.join("a.txt").exists());assert!(registry.find_by_id("stable").is_none());assert_eq!(uses[0].reference,p("a.txt"));
 assert!(!journal_path(&context).exists());let _=fs::remove_dir_all(base);
}
#[test]
fn import_copies_external_file_and_registers_project_copy(){
 let (base,context,roots)=setup("import");let external=base.join("source.txt");fs::write(&external,b"x").unwrap();
 let source=ResourceReference::new(ResourceScope::External,external.clone()).unwrap();let target=p("imported/source.txt");
 let mut registry=ResourceRegistry::default();registry.register("external",source.clone());let mut uses=vec![];
 let plan=ResourceOperationPlan{operation_id:"import-1".into(),kind:ResourceOperationKind::Import,items:vec![ResourceOperationItem{before:Some(source.clone()),after:Some(target.clone())}]};
 execute_operation(&context,&roots,&mut registry,&mut uses,&plan,None,||"project-copy".into()).unwrap();
 assert!(external.is_file());assert!(roots.project.join("imported/source.txt").is_file());assert!(registry.find_by_reference(&source).is_some());assert_eq!(registry.find_by_reference(&target).unwrap().resource_id,"project-copy");
 let _=fs::remove_dir_all(base);
}
#[test]
fn filesystem_failure_leaves_journal_for_recovery(){
 let (base,context,roots)=setup("failure");
 let mut registry=ResourceRegistry::default();let mut uses=vec![];
 let plan=ResourceOperationPlan{operation_id:"failed".into(),kind:ResourceOperationKind::Move,items:vec![ResourceOperationItem{before:Some(p("missing.txt")),after:Some(p("new.txt"))}]};
 assert!(execute_operation(&context,&roots,&mut registry,&mut uses,&plan,None,||"x".into()).is_err());
 assert!(journal_path(&context).is_file());let _=fs::remove_dir_all(base);
}
