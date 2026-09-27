use workflow_ide_framework::{
    locate_replace::{locate_replace, FrameworkResourceUse},
    project_resource::{ResourceReference, ResourceScope},
    resource_registry::ResourceRegistry,
};

fn p(path:&str)->ResourceReference { ResourceReference::new(ResourceScope::Project,path).unwrap() }
fn a(path:&str)->ResourceReference { ResourceReference::new(ResourceScope::Application,path).unwrap() }

#[test]
fn replaces_all_identical_framework_references() {
    let old=p("missing.png"); let new=a("replacement.png");
    let mut registry=ResourceRegistry::default(); registry.register("old-id",old.clone());
    let mut uses=vec![
        FrameworkResourceUse{owner_id:"button".into(),reference:old.clone()},
        FrameworkResourceUse{owner_id:"image".into(),reference:old.clone()},
        FrameworkResourceUse{owner_id:"other".into(),reference:p("other.png")},
    ];
    let plan=locate_replace(&mut registry,&mut uses,&old,new.clone(),||"new-id".into());
    assert_eq!(plan.affected_owner_ids,vec!["button","image"]);
    assert_eq!(uses[0].reference,new); assert_eq!(uses[1].reference,new);
    assert_eq!(uses[2].reference,p("other.png"));
}

#[test]
fn old_registry_entry_is_preserved_and_new_one_is_added() {
    let old=p("missing.png"); let new=p("replacement.png");
    let mut registry=ResourceRegistry::default(); registry.register("old-id",old.clone());
    let mut uses=vec![];
    let plan=locate_replace(&mut registry,&mut uses,&old,new.clone(),||"new-id".into());
    assert!(plan.new_registry_entry_added);
    assert_eq!(registry.find_by_reference(&old).unwrap().resource_id,"old-id");
    assert_eq!(registry.find_by_reference(&new).unwrap().resource_id,"new-id");
    assert_eq!(registry.entries().len(),2);
}

#[test]
fn existing_new_registry_entry_is_reused() {
    let old=p("missing.png"); let new=a("replacement.png");
    let mut registry=ResourceRegistry::default();
    registry.register("old-id",old.clone()); registry.register("existing-new",new.clone());
    let mut uses=vec![FrameworkResourceUse{owner_id:"x".into(),reference:old.clone()}];
    let plan=locate_replace(&mut registry,&mut uses,&old,new,||panic!("must not allocate id"));
    assert!(!plan.new_registry_entry_added);
    assert_eq!(plan.new_registry_entry.resource_id,"existing-new");
    assert_eq!(registry.entries().len(),2);
}

#[test]
fn normalized_old_reference_matches_framework_uses() {
    let old=p("textures/robot.png"); let mut registry=ResourceRegistry::default();
    registry.register("old",old.clone());
    let mut uses=vec![FrameworkResourceUse{owner_id:"x".into(),reference:p("textures/./robot.png")}];
    let plan=locate_replace(&mut registry,&mut uses,&old,p("textures/new.png"),||"new".into());
    assert_eq!(plan.affected_owner_ids,vec!["x"]);
}
