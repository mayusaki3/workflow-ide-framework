//! Consumer-visible project lifecycle events.

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectEventKind {
    Created,
    Opened,
    Saved,
    SavedAs,
    CloseRequested,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectEvent {
    pub kind: ProjectEventKind,
    pub root: Option<PathBuf>,
    pub name: Option<String>,
}

impl ProjectEvent {
    pub fn new(kind: ProjectEventKind, root: Option<PathBuf>, name: Option<String>) -> Self {
        Self { kind, root, name }
    }
}

pub type ProjectEventHandler = Box<dyn FnMut(&ProjectEvent)>;

#[derive(Default)]
pub struct ProjectEventDispatcher {
    handlers: Vec<ProjectEventHandler>,
}

impl ProjectEventDispatcher {
    pub fn subscribe(&mut self, handler: impl FnMut(&ProjectEvent) + 'static) {
        self.handlers.push(Box::new(handler));
    }
    pub fn emit(&mut self, event: ProjectEvent) {
        for handler in &mut self.handlers {
            handler(&event);
        }
    }
}
