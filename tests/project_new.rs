use workflow_ide_framework::{
    project::ProjectContext,
    project_lifecycle::{NewProjectError, ProjectSession, ProjectStorageState},
    project_resource::NewProjectStoragePolicy,
};

#[test]
fn deferred_new_project_starts_unsaved_and_dirty() {
    let session = ProjectSession::new_project(NewProjectStoragePolicy::Deferred, None).unwrap();
    assert_eq!(session.storage, ProjectStorageState::Unsaved);
    assert!(session.dirty.metadata && session.dirty.framework && session.dirty.application);
    assert!(session.current_save_id.is_none());
}

#[test]
fn required_new_project_requires_root() {
    assert_eq!(
        ProjectSession::new_project(NewProjectStoragePolicy::Required, None),
        Err(NewProjectError::StorageRequired)
    );
}

#[test]
fn required_new_project_accepts_root() {
    let context = ProjectContext::new("demo");
    let session = ProjectSession::new_project(NewProjectStoragePolicy::Required, Some(context.clone())).unwrap();
    assert_eq!(session.context(), Some(&context));
    assert!(session.dirty.is_dirty());
}

#[test]
fn deferred_project_can_gain_storage_on_first_save() {
    let mut session = ProjectSession::new_project(NewProjectStoragePolicy::Deferred, None).unwrap();
    let context = ProjectContext::new("saved-project");
    session.set_storage(context.clone());
    assert_eq!(session.context(), Some(&context));
}
