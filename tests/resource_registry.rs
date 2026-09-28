use std::path::PathBuf;
use workflow_ide_framework::{
    framework_settings::{FrameworkSettings, StoredResourceEntry, StoredResourceScope},
    project_resource::{ResourceReference, ResourceScope},
    resource_registry::{RegisterResult, ResourceRegistry},
};

fn project(path: &str) -> ResourceReference {
    ResourceReference::new(ResourceScope::Project, path).unwrap()
}

#[test]
fn duplicate_scope_and_normalized_path_returns_existing_entry() {
    let mut registry = ResourceRegistry::default();
    let first = registry.register("stable-1", project("textures/./robot.png"));
    assert!(matches!(first, RegisterResult::Added(_)));
    let duplicate = registry.register("stable-2", project("textures/robot.png"));
    assert!(matches!(duplicate, RegisterResult::Existing(ref e) if e.resource_id == "stable-1"));
    assert_eq!(registry.entries().len(), 1);
}

#[test]
fn same_path_in_different_scopes_is_not_duplicate() {
    let mut registry = ResourceRegistry::default();
    registry.register("project", project("robot.png"));
    registry.register("application", ResourceReference::new(ResourceScope::Application, "robot.png").unwrap());
    assert_eq!(registry.entries().len(), 2);
}

#[test]
fn remove_is_registry_only_model_operation() {
    let mut registry = ResourceRegistry::default();
    registry.register("r1", project("robot.png"));
    let removed = registry.remove("r1").unwrap();
    assert_eq!(removed.resource_id, "r1");
    assert!(registry.entries().is_empty());
}

#[test]
fn replace_reference_preserves_resource_id() {
    let mut registry = ResourceRegistry::default();
    registry.register("stable-id", project("old.png"));
    assert_eq!(registry.replace_reference(&project("old.png"), project("new.png")), 1);
    let entry = registry.find_by_id("stable-id").unwrap();
    assert_eq!(entry.reference, project("new.png"));
}


#[test]
fn registry_round_trips_through_framework_settings() {
    let mut registry = ResourceRegistry::default();
    registry.register("r1", project("textures/robot.png"));
    let settings = FrameworkSettings::from_registry(&registry);
    let restored = settings.to_registry().unwrap();
    assert_eq!(restored.entries(), registry.entries());
}

#[test]
fn invalid_stored_reference_is_not_silently_dropped() {
    let settings = FrameworkSettings {
        format_version: 1,
        save_id: None,
        resources: vec![StoredResourceEntry {
            resource_id: "bad".into(),
            scope: StoredResourceScope::Project,
            path: PathBuf::from("../escape.png"),
        }],
    };
    assert!(settings.to_registry().is_err());
}
