use std::fs;
use workflow_ide_framework::{
    project::ProjectContext,
    project_resource::{ResourceOperationDecision, ResourceReference, ResourceScope},
    resource_journal::{ResourceOperationJournal, journal_path},
    resource_operation::{
        ApplicationResourceOperationAdapter, PreparedResourceOperation, ResourceOperationItem,
        ResourceOperationKind, ResourceOperationPlan, ResourceOperationValidationError,
        prepare_operation,
    },
};
fn p(s: &str) -> ResourceReference {
    ResourceReference::new(ResourceScope::Project, s).unwrap()
}
#[test]
fn rename_is_single_project_resource_only() {
    let plan = ResourceOperationPlan {
        operation_id: "1".into(),
        kind: ResourceOperationKind::Rename,
        items: vec![
            ResourceOperationItem {
                before: Some(p("a")),
                after: Some(p("b")),
            },
            ResourceOperationItem {
                before: Some(p("c")),
                after: Some(p("d")),
            },
        ],
    };
    assert_eq!(
        plan.validate(),
        Err(ResourceOperationValidationError::RenameRequiresOneItem)
    );
}
#[test]
fn import_destination_must_be_project_scope() {
    let plan = ResourceOperationPlan {
        operation_id: "1".into(),
        kind: ResourceOperationKind::Import,
        items: vec![ResourceOperationItem {
            before: Some(
                ResourceReference::new(ResourceScope::External, std::env::temp_dir().join("a"))
                    .unwrap(),
            ),
            after: Some(ResourceReference::new(ResourceScope::Application, "a").unwrap()),
        }],
    };
    assert_eq!(
        plan.validate(),
        Err(ResourceOperationValidationError::ProjectScopeRequired)
    );
}
struct Reject;
impl ApplicationResourceOperationAdapter for Reject {
    type Error = &'static str;
    fn prepare_resource_operation(
        &mut self,
        _: &ResourceOperationPlan,
    ) -> Result<ResourceOperationDecision, Self::Error> {
        Ok(ResourceOperationDecision::Reject {
            reason: Some("busy".into()),
            handled: true,
        })
    }
    fn execute_resource_operation(
        &mut self,
        _: &ResourceOperationPlan,
        _: Option<&workflow_ide_framework::project_resource::ApplicationJournalData>,
    ) -> Result<(), Self::Error> {
        panic!("rejected operation must not execute")
    }
}
#[test]
fn application_can_reject_before_journal_or_filesystem_stage() {
    let plan = ResourceOperationPlan {
        operation_id: "1".into(),
        kind: ResourceOperationKind::Delete,
        items: vec![ResourceOperationItem {
            before: Some(p("a")),
            after: None,
        }],
    };
    assert_eq!(
        prepare_operation(&mut Reject, &plan).unwrap(),
        PreparedResourceOperation::Rejected {
            reason: Some("busy".into()),
            handled: true
        }
    );
}
#[test]
fn journal_is_persisted_before_operation_can_continue() {
    let root = std::env::temp_dir().join(format!("wfide-journal-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let context = ProjectContext::new(&root);
    let plan = ResourceOperationPlan {
        operation_id: "op-1".into(),
        kind: ResourceOperationKind::Move,
        items: vec![ResourceOperationItem {
            before: Some(p("a")),
            after: Some(p("folder/a")),
        }],
    };
    let journal = ResourceOperationJournal::from_plan(&plan, None);
    journal.save(&context).unwrap();
    assert!(journal_path(&context).is_file());
    let text = fs::read_to_string(journal_path(&context)).unwrap();
    assert!(text.contains("op-1"));
    assert!(text.contains("move"));
    ResourceOperationJournal::remove(&context).unwrap();
    assert!(!journal_path(&context).exists());
    let _ = fs::remove_dir_all(root);
}
