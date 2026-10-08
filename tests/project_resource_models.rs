use std::path::PathBuf;

use workflow_ide_framework::project_resource::{
    FileTypeFilter, ProjectDirtyState, ResourceReference, ResourceReferenceError, ResourceScope,
};

#[test]
fn project_dirty_is_or_of_owner_flags() {
    assert!(!ProjectDirtyState::default().is_dirty());
    assert!(
        ProjectDirtyState {
            metadata: true,
            ..Default::default()
        }
        .is_dirty()
    );
    assert!(
        ProjectDirtyState {
            framework: true,
            ..Default::default()
        }
        .is_dirty()
    );
    assert!(
        ProjectDirtyState {
            application: true,
            ..Default::default()
        }
        .is_dirty()
    );
}

#[test]
fn project_resource_reference_is_relative_and_normalized() {
    let reference = ResourceReference::new(
        ResourceScope::Project,
        PathBuf::from("textures").join(".").join("robot.PNG"),
    )
    .unwrap();
    assert_eq!(reference.path, PathBuf::from("textures").join("robot.PNG"));
}

#[test]
fn application_resource_rejects_absolute_path() {
    let absolute = if cfg!(windows) {
        PathBuf::from(r"C:\app\image.png")
    } else {
        PathBuf::from("/app/image.png")
    };
    assert_eq!(
        ResourceReference::new(ResourceScope::Application, absolute),
        Err(ResourceReferenceError::ApplicationPathMustBeRelative),
    );
}

#[test]
fn project_resource_rejects_parent_traversal() {
    assert_eq!(
        ResourceReference::new(ResourceScope::Project, "../outside.png"),
        Err(ResourceReferenceError::ParentTraversalNotAllowed),
    );
}

#[test]
fn external_resource_requires_absolute_path() {
    assert_eq!(
        ResourceReference::new(ResourceScope::External, "relative/image.png"),
        Err(ResourceReferenceError::ExternalPathMustBeAbsolute),
    );
}

#[test]
fn file_type_filter_normalizes_extensions_and_matches_case_insensitively() {
    let filter = FileTypeFilter::new("Images", ["png", ".JPG", "  jpeg  "]);
    assert_eq!(filter.extensions, vec!["png", "jpg", "jpeg"]);
    assert!(filter.accepts(PathBuf::from("image.PNG").as_path()));
    assert!(filter.accepts(PathBuf::from("photo.jpg").as_path()));
    assert!(!filter.accepts(PathBuf::from("notes.txt").as_path()));
}
