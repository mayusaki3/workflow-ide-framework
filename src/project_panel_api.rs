//! Application-owned logical Project Panel tree and invalidation contract.
//! Existing ProjectPanelModel remains available for legacy callers.

use crate::project::ProjectContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPanelNode {
    pub id: String,
    pub label: String,
    pub kind: AppPanelNodeKind,
    pub children: Vec<AppPanelNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppPanelNodeKind {
    Category {
        type_id: String,
    },
    LogicalResource {
        resource_id: String,
        type_id: String,
    },
    RegisteredResource {
        registry_resource_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppPanelAction {
    Selected {
        node_ids: Vec<String>,
    },
    Activated {
        node_id: String,
    },
    ContextAction {
        node_ids: Vec<String>,
        action_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppPanelActionResult {
    Handled,
    NotHandled,
    Rejected { reason: String },
}

pub trait ApplicationProjectPanelProvider {
    fn project_panel_roots(&self, project: &ProjectContext) -> Vec<AppPanelNode>;
    fn handle_project_panel_action(
        &mut self,
        project: &ProjectContext,
        action: AppPanelAction,
    ) -> AppPanelActionResult;
}

/// Coalesces application notifications until the framework consumes the snapshot request.
/// Project switching clears pending work to avoid leaking stale notifications.
#[derive(Debug, Clone, Default)]
pub struct ProjectPanelInvalidation {
    pending: bool,
}

impl ProjectPanelInvalidation {
    pub fn invalidate_application_project_panel(&mut self) {
        self.pending = true;
    }

    pub fn take_pending(&mut self) -> bool {
        std::mem::take(&mut self.pending)
    }

    pub fn clear_on_project_change(&mut self) {
        self.pending = false;
    }

    pub fn is_pending(&self) -> bool {
        self.pending
    }
}
