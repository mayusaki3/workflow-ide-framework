use std::{fs, path::PathBuf};
use workflow_ide_framework::{
    project_resource::{ResourceReference, ResourceRootStatus, ResourceScope, ResourceState},
    resource_state::ResourceRoots,
};

fn root(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!(
        "wfide-resource-state-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn project_and_application_references_resolve_against_their_roots() {
    let base = root("resolve");
    let roots = ResourceRoots {
        application: base.join("app"),
        project: base.join("project"),
    };
    assert_eq!(
        roots.resolve(&ResourceReference::new(ResourceScope::Project, "a.png").unwrap()),
        base.join("project/a.png")
    );
    assert_eq!(
        roots.resolve(&ResourceReference::new(ResourceScope::Application, "a.png").unwrap()),
        base.join("app/a.png")
    );
    let _ = fs::remove_dir_all(base);
}

#[test]
fn unavailable_project_root_is_not_reported_as_individual_missing() {
    let base = root("root-missing");
    let roots = ResourceRoots {
        application: base.join("app"),
        project: base.join("missing"),
    };
    let reference = ResourceReference::new(ResourceScope::Project, "robot.png").unwrap();
    assert_eq!(
        roots.root_status(ResourceScope::Project),
        Some(ResourceRootStatus::Missing)
    );
    assert_eq!(roots.state(&reference), ResourceState::RootUnavailable);
    let _ = fs::remove_dir_all(base);
}

#[test]
fn missing_file_under_available_root_is_missing() {
    let base = root("file-missing");
    let project = base.join("project");
    fs::create_dir_all(&project).unwrap();
    let roots = ResourceRoots {
        application: base.join("app"),
        project,
    };
    assert_eq!(
        roots.state(&ResourceReference::new(ResourceScope::Project, "robot.png").unwrap()),
        ResourceState::Missing
    );
    let _ = fs::remove_dir_all(base);
}

#[test]
fn only_regular_files_are_available_resources() {
    let base = root("regular");
    let project = base.join("project");
    fs::create_dir_all(project.join("folder")).unwrap();
    fs::write(project.join("robot.png"), b"x").unwrap();
    let roots = ResourceRoots {
        application: base.join("app"),
        project,
    };
    assert_eq!(
        roots.state(&ResourceReference::new(ResourceScope::Project, "robot.png").unwrap()),
        ResourceState::Available
    );
    assert_eq!(
        roots.state(&ResourceReference::new(ResourceScope::Project, "folder").unwrap()),
        ResourceState::Missing
    );
    let _ = fs::remove_dir_all(base);
}

#[test]
fn external_reference_has_no_shared_root_state() {
    let base = root("external");
    let file = base.join("outside.txt");
    fs::write(&file, b"x").unwrap();
    let roots = ResourceRoots {
        application: base.join("app"),
        project: base.join("project"),
    };
    let reference = ResourceReference::new(ResourceScope::External, file.clone()).unwrap();
    assert_eq!(roots.root_status(ResourceScope::External), None);
    assert_eq!(roots.state(&reference), ResourceState::Available);
    fs::remove_file(file).unwrap();
    assert_eq!(roots.state(&reference), ResourceState::Missing);
    let _ = fs::remove_dir_all(base);
}
