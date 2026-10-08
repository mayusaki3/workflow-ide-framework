use workflow_ide_framework::{
    file_resource_selector::FileSelectionResult,
    project_resource::{ResourceReference, ResourceScope},
    resource_registry::ResourceRegistry,
    resource_selection_registry::{FileSelectionRegistration, register_selection},
};

#[test]
fn selected_files_are_registered_and_existing_reference_is_reused() {
    let mut registry = ResourceRegistry::default();
    let reference = ResourceReference::new(ResourceScope::Project, "a.png").unwrap();
    registry.register("existing", reference.clone());
    let mut n = 0;
    let result = register_selection(
        &mut registry,
        FileSelectionResult::Selected(vec![
            reference,
            ResourceReference::new(ResourceScope::Project, "b.png").unwrap(),
        ]),
        || {
            n += 1;
            format!("new-{n}")
        },
    );
    match result {
        FileSelectionRegistration::Registered(batch) => {
            assert_eq!(batch.entries.len(), 2);
            assert_eq!(batch.entries[0].resource_id, "existing");
            assert_eq!(batch.added_count, 1);
        }
        other => panic!("unexpected: {other:?}"),
    }
    assert_eq!(registry.entries().len(), 2);
}

#[test]
fn cancelled_selection_does_not_modify_registry() {
    let mut registry = ResourceRegistry::default();
    let result = register_selection(&mut registry, FileSelectionResult::Cancelled, || {
        "unused".into()
    });
    assert_eq!(result, FileSelectionRegistration::Cancelled);
    assert!(registry.entries().is_empty());
}
