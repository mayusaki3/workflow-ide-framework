use std::{cell::RefCell, rc::Rc};
use workflow_ide_framework::{
    application_event::{ApplicationEvent, ApplicationEventDispatcher},
    command::{CommandDefinition, CommandRegistry, CommandRegistryError},
    project_event::{ProjectEvent, ProjectEventDispatcher, ProjectEventKind},
    status::{StatusItem, StatusRegistry, StatusRegistryError},
};

#[test]
fn command_registry_rejects_duplicates_and_respects_enabled() {
    let count = Rc::new(RefCell::new(0));
    let captured = count.clone();
    let mut registry = CommandRegistry::default();
    registry.register(CommandDefinition::new("app.run", "Run", move || *captured.borrow_mut() += 1)).unwrap();
    assert!(matches!(registry.register(CommandDefinition::new("app.run", "Run", || {})), Err(CommandRegistryError::DuplicateId(_))));
    assert_eq!(registry.dispatch("app.run").unwrap(), true);
    registry.set_enabled("app.run", false).unwrap();
    assert_eq!(registry.dispatch("app.run").unwrap(), false);
    assert_eq!(*count.borrow(), 1);
}

#[test]
fn status_registry_updates_value_and_visibility() {
    let mut registry = StatusRegistry::default();
    registry.register(StatusItem::new("app.mode", "Mode", "Ready")).unwrap();
    assert!(matches!(registry.register(StatusItem::new("app.mode", "Mode", "Other")), Err(StatusRegistryError::DuplicateId(_))));
    registry.set_value("app.mode", "Busy").unwrap();
    registry.set_visible("app.mode", false).unwrap();
    let item = registry.get("app.mode").unwrap();
    assert_eq!(item.value, "Busy");
    assert!(!item.visible);
    assert!(matches!(registry.set_value("missing", "x"), Err(StatusRegistryError::NotFound(_))));
}

#[test]
fn application_started_is_emitted_once() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let captured = events.clone();
    let mut dispatcher = ApplicationEventDispatcher::default();
    dispatcher.subscribe(move |event| captured.borrow_mut().push(event));
    dispatcher.emit(ApplicationEvent::Starting);
    dispatcher.emit(ApplicationEvent::Started);
    dispatcher.emit(ApplicationEvent::Started);
    assert_eq!(&*events.borrow(), &[ApplicationEvent::Starting, ApplicationEvent::Started]);
}

#[test]
fn project_event_preserves_kind_and_context() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let captured = events.clone();
    let mut dispatcher = ProjectEventDispatcher::default();
    dispatcher.subscribe(move |event| captured.borrow_mut().push(event.clone()));
    dispatcher.emit(ProjectEvent::new(ProjectEventKind::Saved, Some("/tmp/project".into()), Some("Example".into())));
    let events = events.borrow();
    assert_eq!(events[0].kind, ProjectEventKind::Saved);
    assert_eq!(events[0].name.as_deref(), Some("Example"));
}


#[test]
fn shared_status_handle_updates_registered_item() {
    use workflow_ide_framework::status::StatusHandle;
    let handle = StatusHandle::new();
    handle.register(StatusItem::new("app.connection", "Connection", "Offline")).unwrap();
    let consumer = handle.clone();
    consumer.set_value("app.connection", "Online").unwrap();
    consumer.set_visible("app.connection", true).unwrap();
    let items = handle.snapshot().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].value, "Online");
}
