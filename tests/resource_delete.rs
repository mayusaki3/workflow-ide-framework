use workflow_ide_framework::{
    project_resource::{ResourceReference, ResourceScope},
    resource_delete::{
        ApplicationDeleteReferenceInspector, DeleteDecision, DeleteReferences, authorize_delete,
        inspect_delete,
    },
};

struct Inspector(Result<Option<usize>, &'static str>);

impl ApplicationDeleteReferenceInspector for Inspector {
    type Error = &'static str;

    fn count_resource_references(&mut self, _: &str) -> Result<Option<usize>, Self::Error> {
        self.0
    }
}

fn target() -> ResourceReference {
    ResourceReference::new(ResourceScope::Project, "models/shared.urdf").unwrap()
}

#[test]
fn reg_del_002_referenced_file_requires_explicit_confirmation() {
    let request = inspect_delete(Some(&mut Inspector(Ok(Some(3)))), "id-1", &target());
    assert_eq!(request.references, DeleteReferences::Known(3));
    assert_eq!(request.resource_id, "id-1");
    assert_eq!(request.reference, target());
    assert!(!authorize_delete(&request, DeleteDecision::Cancel));
    assert!(authorize_delete(&request, DeleteDecision::Confirm));
}

#[test]
fn reg_del_003_missing_or_failed_inspection_is_unknown_not_zero() {
    let no_inspector: Option<&mut Inspector> = None;
    assert_eq!(
        inspect_delete(no_inspector, "id-1", &target()).references,
        DeleteReferences::Unknown
    );
    assert_eq!(
        inspect_delete(Some(&mut Inspector(Ok(None))), "id-1", &target()).references,
        DeleteReferences::Unknown
    );
    assert_eq!(
        inspect_delete(Some(&mut Inspector(Err("offline"))), "id-1", &target()).references,
        DeleteReferences::Unknown
    );
}

#[test]
fn reg_del_004_zero_references_still_requires_confirmation() {
    let request = inspect_delete(Some(&mut Inspector(Ok(Some(0)))), "id-1", &target());
    assert_eq!(request.references, DeleteReferences::Known(0));
    assert!(!authorize_delete(&request, DeleteDecision::Cancel));
}

#[test]
fn reg_del_004_cancelled_execution_does_not_touch_file_registry_or_journal() {
    use std::fs;
    use workflow_ide_framework::{
        resource_executor::{ConfirmedDeleteError, execute_confirmed_delete},
        resource_journal::journal_path,
        resource_operation::{ResourceOperationItem, ResourceOperationKind, ResourceOperationPlan},
        resource_registry::ResourceRegistry,
        resource_state::ResourceRoots,
        project::ProjectContext,
    };
    let base = std::env::temp_dir().join(format!("wfide-delete-guard-{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    let project = base.join("project");
    let roots = ResourceRoots {
        application: base.join("app"),
        project: project.join("resources"),
    };
    fs::create_dir_all(&roots.project).unwrap();
    fs::write(roots.project.join("models.txt"), b"content").unwrap();
    let context = ProjectContext::new(project);
    let reference = ResourceReference::new(ResourceScope::Project, "models.txt").unwrap();
    let mut registry = ResourceRegistry::default();
    registry.register("stable", reference.clone());
    let mut uses = vec![];
    let plan = ResourceOperationPlan {
        operation_id: "cancelled-delete".into(),
        kind: ResourceOperationKind::Delete,
        items: vec![ResourceOperationItem { before: Some(reference.clone()), after: None }],
    };
    let confirmation = inspect_delete(Some(&mut Inspector(Ok(Some(3)))), "stable", &reference);
    let result = execute_confirmed_delete(
        &context, &roots, &mut registry, &mut uses, &plan,
        &confirmation, DeleteDecision::Cancel, None, || "unused".into(),
    );
    assert!(matches!(result, Err(ConfirmedDeleteError::Cancelled)));
    assert_eq!(fs::read(roots.project.join("models.txt")).unwrap(), b"content");
    assert_eq!(registry.find_by_id("stable").unwrap().reference, reference);
    assert!(!journal_path(&context).exists());
    let _ = fs::remove_dir_all(base);
}
