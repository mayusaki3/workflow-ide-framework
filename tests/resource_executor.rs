use std::{fs, path::PathBuf};
use workflow_ide_framework::{
    locate_replace::FrameworkResourceUse,
    project::ProjectContext,
    project_resource::{
        ApplicationJournalData, ResourceOperationDecision, ResourceReference, ResourceScope,
    },
    resource_executor::{execute_operation, execute_prepared_operation},
    resource_journal::{ResourceOperationJournal, journal_path},
    resource_operation::{
        ApplicationResourceOperationAdapter, PreparedResourceOperation, ResourceOperationItem,
        ResourceOperationKind, ResourceOperationPlan,
    },
    resource_registry::ResourceRegistry,
    resource_state::ResourceRoots,
};
fn setup(name: &str) -> (PathBuf, ProjectContext, ResourceRoots) {
    let base = std::env::temp_dir().join(format!("wfide-executor-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    let project = base.join("project");
    let resources = project.join("resources");
    let app = base.join("app");
    fs::create_dir_all(&resources).unwrap();
    fs::create_dir_all(&app).unwrap();
    (
        base,
        ProjectContext::new(project),
        ResourceRoots {
            application: app,
            project: resources,
        },
    )
}
fn p(s: &str) -> ResourceReference {
    ResourceReference::new(ResourceScope::Project, s).unwrap()
}
#[test]
fn move_updates_file_registry_and_framework_references_then_clears_journal() {
    let (base, context, roots) = setup("move");
    fs::write(roots.project.join("a.txt"), b"x").unwrap();
    let mut registry = ResourceRegistry::default();
    registry.register("stable", p("a.txt"));
    let mut uses = vec![FrameworkResourceUse {
        owner_id: "editor".into(),
        reference: p("a.txt"),
    }];
    let plan = ResourceOperationPlan {
        operation_id: "move-1".into(),
        kind: ResourceOperationKind::Move,
        items: vec![ResourceOperationItem {
            before: Some(p("a.txt")),
            after: Some(p("folder/a.txt")),
        }],
    };
    execute_operation(
        &context,
        &roots,
        &mut registry,
        &mut uses,
        &plan,
        None,
        || "unused".into(),
    )
    .unwrap();
    assert!(!roots.project.join("a.txt").exists());
    assert!(roots.project.join("folder/a.txt").is_file());
    assert_eq!(
        registry.find_by_id("stable").unwrap().reference,
        p("folder/a.txt")
    );
    assert_eq!(uses[0].reference, p("folder/a.txt"));
    assert!(!journal_path(&context).exists());
    let _ = fs::remove_dir_all(base);
}
#[test]
fn delete_removes_file_and_registry_but_keeps_framework_reference_missing() {
    let (base, context, roots) = setup("delete");
    fs::write(roots.project.join("a.txt"), b"x").unwrap();
    let mut registry = ResourceRegistry::default();
    registry.register("stable", p("a.txt"));
    let mut uses = vec![FrameworkResourceUse {
        owner_id: "editor".into(),
        reference: p("a.txt"),
    }];
    let plan = ResourceOperationPlan {
        operation_id: "delete-1".into(),
        kind: ResourceOperationKind::Delete,
        items: vec![ResourceOperationItem {
            before: Some(p("a.txt")),
            after: None,
        }],
    };
    execute_operation(
        &context,
        &roots,
        &mut registry,
        &mut uses,
        &plan,
        None,
        || "unused".into(),
    )
    .unwrap();
    assert!(!roots.project.join("a.txt").exists());
    assert!(registry.find_by_id("stable").is_none());
    assert_eq!(uses[0].reference, p("a.txt"));
    assert!(!journal_path(&context).exists());
    let _ = fs::remove_dir_all(base);
}
#[test]
fn import_copies_external_file_and_registers_project_copy() {
    let (base, context, roots) = setup("import");
    let external = base.join("source.txt");
    fs::write(&external, b"x").unwrap();
    let source = ResourceReference::new(ResourceScope::External, external.clone()).unwrap();
    let target = p("imported/source.txt");
    let mut registry = ResourceRegistry::default();
    registry.register("external", source.clone());
    let mut uses = vec![];
    let plan = ResourceOperationPlan {
        operation_id: "import-1".into(),
        kind: ResourceOperationKind::Import,
        items: vec![ResourceOperationItem {
            before: Some(source.clone()),
            after: Some(target.clone()),
        }],
    };
    execute_operation(
        &context,
        &roots,
        &mut registry,
        &mut uses,
        &plan,
        None,
        || "project-copy".into(),
    )
    .unwrap();
    assert!(external.is_file());
    assert!(roots.project.join("imported/source.txt").is_file());
    assert!(registry.find_by_reference(&source).is_some());
    assert_eq!(
        registry.find_by_reference(&target).unwrap().resource_id,
        "project-copy"
    );
    let _ = fs::remove_dir_all(base);
}
#[test]
fn filesystem_failure_leaves_journal_for_recovery() {
    let (base, context, roots) = setup("failure");
    let mut registry = ResourceRegistry::default();
    let mut uses = vec![];
    let plan = ResourceOperationPlan {
        operation_id: "failed".into(),
        kind: ResourceOperationKind::Move,
        items: vec![ResourceOperationItem {
            before: Some(p("missing.txt")),
            after: Some(p("new.txt")),
        }],
    };
    assert!(
        execute_operation(
            &context,
            &roots,
            &mut registry,
            &mut uses,
            &plan,
            None,
            || "x".into()
        )
        .is_err()
    );
    assert!(journal_path(&context).is_file());
    let _ = fs::remove_dir_all(base);
}

struct ConvertingApp {
    executed: bool,
    saw_journal: bool,
    fail: bool,
}
impl ApplicationResourceOperationAdapter for ConvertingApp {
    type Error = &'static str;
    fn prepare_resource_operation(
        &mut self,
        _: &ResourceOperationPlan,
    ) -> Result<ResourceOperationDecision, Self::Error> {
        Ok(ResourceOperationDecision::Handled {
            journal_data: Some(ApplicationJournalData {
                format: "test".into(),
                data: vec![1],
            }),
        })
    }
    fn execute_resource_operation(
        &mut self,
        plan: &ResourceOperationPlan,
        _: Option<&ApplicationJournalData>,
    ) -> Result<(), Self::Error> {
        self.saw_journal = plan.items.len() == 1;
        self.executed = true;
        if self.fail {
            Err("conversion failed")
        } else {
            Ok(())
        }
    }
}
#[test]
fn application_handled_export_runs_without_framework_copy_and_clears_journal() {
    let (base, context, roots) = setup("handled-export");
    let source = roots.application.join("source.dat");
    fs::write(&source, b"source").unwrap();
    let target = base.join("converted.out");
    let before = ResourceReference::new(ResourceScope::Application, "source.dat").unwrap();
    let after = ResourceReference::new(ResourceScope::External, target.clone()).unwrap();
    let plan = ResourceOperationPlan {
        operation_id: "export".into(),
        kind: ResourceOperationKind::Export,
        items: vec![ResourceOperationItem {
            before: Some(before),
            after: Some(after),
        }],
    };
    let mut app = ConvertingApp {
        executed: false,
        saw_journal: false,
        fail: false,
    };
    let mut registry = ResourceRegistry::default();
    let mut uses = vec![];
    execute_prepared_operation(
        &mut app,
        &context,
        &roots,
        &mut registry,
        &mut uses,
        &plan,
        PreparedResourceOperation::Handled {
            journal_data: Some(ApplicationJournalData {
                format: "test".into(),
                data: vec![1],
            }),
        },
        || "unused".into(),
    )
    .unwrap();
    assert!(app.executed && app.saw_journal);
    assert!(!target.exists());
    assert!(!journal_path(&context).exists());
    let _ = fs::remove_dir_all(base);
}
#[test]
fn failed_application_conversion_leaves_journal_for_recovery() {
    let (base, context, roots) = setup("handled-fail");
    let target = base.join("converted.out");
    let before = ResourceReference::new(ResourceScope::Application, "source.dat").unwrap();
    let after = ResourceReference::new(ResourceScope::External, target).unwrap();
    let plan = ResourceOperationPlan {
        operation_id: "export-fail".into(),
        kind: ResourceOperationKind::Export,
        items: vec![ResourceOperationItem {
            before: Some(before),
            after: Some(after),
        }],
    };
    let mut app = ConvertingApp {
        executed: false,
        saw_journal: false,
        fail: true,
    };
    let mut registry = ResourceRegistry::default();
    let mut uses = vec![];
    assert!(
        execute_prepared_operation(
            &mut app,
            &context,
            &roots,
            &mut registry,
            &mut uses,
            &plan,
            PreparedResourceOperation::Handled { journal_data: None },
            || "unused".into()
        )
        .is_err()
    );
    assert!(app.executed);
    assert!(journal_path(&context).is_file());
    let _ = fs::remove_dir_all(base);
}

#[test]
fn failed_import_journal_preserves_preallocated_target_resource_id() {
    let (base, context, roots) = setup("import-journal-id");
    let source = ResourceReference::new(ResourceScope::External, base.join("missing.txt")).unwrap();
    let target = p("imported/missing.txt");
    let plan = ResourceOperationPlan {
        operation_id: "import-fail".into(),
        kind: ResourceOperationKind::Import,
        items: vec![ResourceOperationItem {
            before: Some(source),
            after: Some(target),
        }],
    };
    let mut registry = ResourceRegistry::default();
    let mut uses = vec![];
    assert!(
        execute_operation(
            &context,
            &roots,
            &mut registry,
            &mut uses,
            &plan,
            None,
            || "preallocated-target".into()
        )
        .is_err()
    );
    let journal = ResourceOperationJournal::load(&context).unwrap().unwrap();
    assert_eq!(
        journal.items[0].after_resource_id.as_deref(),
        Some("preallocated-target")
    );
    let _ = fs::remove_dir_all(base);
}

#[test]
fn replace_registers_target_with_preallocated_resource_id() {
    let (base, context, roots) = setup("replace-target-id");
    let old = p("old.txt");
    let target = p("replacement.txt");
    let mut registry = ResourceRegistry::default();
    registry.register("old-id", old.clone());
    let mut uses = vec![FrameworkResourceUse {
        owner_id: "editor".into(),
        reference: old.clone(),
    }];
    let plan = ResourceOperationPlan {
        operation_id: "replace-1".into(),
        kind: ResourceOperationKind::Replace,
        items: vec![ResourceOperationItem {
            before: Some(old.clone()),
            after: Some(target.clone()),
        }],
    };
    execute_operation(
        &context,
        &roots,
        &mut registry,
        &mut uses,
        &plan,
        None,
        || "replacement-id".into(),
    )
    .unwrap();
    assert_eq!(
        registry.find_by_reference(&target).unwrap().resource_id,
        "replacement-id"
    );
    assert_eq!(
        registry.find_by_reference(&old).unwrap().resource_id,
        "old-id"
    );
    assert_eq!(uses[0].reference, target);
    assert!(!journal_path(&context).exists());
    let _ = fs::remove_dir_all(base);
}

struct JournalInspectingReplaceApp {
    context: ProjectContext,
    expected_id: String,
    saw_preallocated_id: bool,
    fail: bool,
}
impl ApplicationResourceOperationAdapter for JournalInspectingReplaceApp {
    type Error = &'static str;
    fn prepare_resource_operation(
        &mut self,
        _: &ResourceOperationPlan,
    ) -> Result<ResourceOperationDecision, Self::Error> {
        Ok(ResourceOperationDecision::Handled { journal_data: None })
    }
    fn execute_resource_operation(
        &mut self,
        _: &ResourceOperationPlan,
        _: Option<&ApplicationJournalData>,
    ) -> Result<(), Self::Error> {
        let journal = ResourceOperationJournal::load(&self.context)
            .unwrap()
            .expect("journal must exist before Application execution");
        self.saw_preallocated_id =
            journal.items[0].after_resource_id.as_deref() == Some(self.expected_id.as_str());
        if self.fail {
            Err("replace conversion failed")
        } else {
            Ok(())
        }
    }
}

#[test]
fn application_handled_replace_journals_target_id_before_application_execution() {
    let (base, context, roots) = setup("handled-replace-id");
    let old = p("old.txt");
    let target = p("converted.txt");
    let plan = ResourceOperationPlan {
        operation_id: "replace-handled".into(),
        kind: ResourceOperationKind::Replace,
        items: vec![ResourceOperationItem {
            before: Some(old.clone()),
            after: Some(target.clone()),
        }],
    };
    let mut registry = ResourceRegistry::default();
    registry.register("old-id", old.clone());
    let mut uses = vec![FrameworkResourceUse {
        owner_id: "editor".into(),
        reference: old,
    }];
    let mut app = JournalInspectingReplaceApp {
        context: context.clone(),
        expected_id: "converted-id".into(),
        saw_preallocated_id: false,
        fail: false,
    };
    execute_prepared_operation(
        &mut app,
        &context,
        &roots,
        &mut registry,
        &mut uses,
        &plan,
        PreparedResourceOperation::Handled { journal_data: None },
        || "converted-id".into(),
    )
    .unwrap();
    assert!(app.saw_preallocated_id);
    assert_eq!(
        registry.find_by_reference(&target).unwrap().resource_id,
        "converted-id"
    );
    assert_eq!(uses[0].reference, target);
    assert!(!journal_path(&context).exists());
    let _ = fs::remove_dir_all(base);
}

#[test]
fn failed_application_handled_replace_keeps_preallocated_target_id_in_journal() {
    let (base, context, roots) = setup("handled-replace-fail-id");
    let old = p("old.txt");
    let target = p("converted.txt");
    let plan = ResourceOperationPlan {
        operation_id: "replace-handled-fail".into(),
        kind: ResourceOperationKind::Replace,
        items: vec![ResourceOperationItem {
            before: Some(old),
            after: Some(target),
        }],
    };
    let mut registry = ResourceRegistry::default();
    let mut uses = vec![];
    let mut app = JournalInspectingReplaceApp {
        context: context.clone(),
        expected_id: "converted-id".into(),
        saw_preallocated_id: false,
        fail: true,
    };
    assert!(
        execute_prepared_operation(
            &mut app,
            &context,
            &roots,
            &mut registry,
            &mut uses,
            &plan,
            PreparedResourceOperation::Handled { journal_data: None },
            || "converted-id".into()
        )
        .is_err()
    );
    assert!(app.saw_preallocated_id);
    let journal = ResourceOperationJournal::load(&context).unwrap().unwrap();
    assert_eq!(
        journal.items[0].after_resource_id.as_deref(),
        Some("converted-id")
    );
    let _ = fs::remove_dir_all(base);
}
