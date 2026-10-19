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
