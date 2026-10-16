//! Application-owned logical Project Panel tree and invalidation contract.
//! Existing ProjectPanelModel remains available for legacy callers.

use crate::project::ProjectContext;
use std::path::{Component, Path, PathBuf};

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
    Directory {
        relative_path: PathBuf,
        filter: DirectoryFilter,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryFilter {
    pub extensions: Vec<String>,
    pub include_files: bool,
    pub include_directories: bool,
    pub recursive: bool,
    pub show_hidden: bool,
}

impl Default for DirectoryFilter {
    fn default() -> Self {
        Self {
            extensions: Vec::new(),
            include_files: true,
            include_directories: true,
            recursive: true,
            show_hidden: false,
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppPanelTreeError {
    DuplicateNodeId(String),
    UnknownRegisteredResource(String),
    ProjectContextRequired,
    InvalidDirectoryPath(PathBuf),
    DirectoryReadFailed(PathBuf),
}

/// Resolve application-owned nodes against the existing Registry without
/// treating unregistered physical files as registered resources.
pub fn build_application_project_tree(
    roots: &[AppPanelNode],
    registry: &crate::resource_registry::ResourceRegistry,
) -> Result<crate::tree_viewer::TreeModel, AppPanelTreeError> {
    build_tree_internal(roots, registry, None)
}

pub fn build_application_project_tree_with_directories(
    roots: &[AppPanelNode],
    registry: &crate::resource_registry::ResourceRegistry,
    project: &ProjectContext,
) -> Result<crate::tree_viewer::TreeModel, AppPanelTreeError> {
    build_tree_internal(roots, registry, Some(project))
}

fn build_tree_internal(
    roots: &[AppPanelNode],
    registry: &crate::resource_registry::ResourceRegistry,
    project: Option<&ProjectContext>,
) -> Result<crate::tree_viewer::TreeModel, AppPanelTreeError> {
    let mut seen = std::collections::HashSet::new();
    let roots = roots
        .iter()
        .map(|node| build_application_node(node, registry, &mut seen, project))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(crate::tree_viewer::TreeModel {
        roots,
        selected_id: None,
    })
}

fn build_application_node(
    node: &AppPanelNode,
    registry: &crate::resource_registry::ResourceRegistry,
    seen: &mut std::collections::HashSet<String>,
    project: Option<&ProjectContext>,
) -> Result<crate::tree_viewer::TreeNode, AppPanelTreeError> {
    if !seen.insert(node.id.clone()) {
        return Err(AppPanelTreeError::DuplicateNodeId(node.id.clone()));
    }
    let node_type = match &node.kind {
        AppPanelNodeKind::Category { .. } => "project.category",
        AppPanelNodeKind::Directory { .. } => "project.directory",
        AppPanelNodeKind::LogicalResource { .. } => "project.logical",
        AppPanelNodeKind::RegisteredResource {
            registry_resource_id,
        } => {
            if registry.find_by_id(registry_resource_id).is_none() {
                return Err(AppPanelTreeError::UnknownRegisteredResource(
                    registry_resource_id.clone(),
                ));
            }
            "project.registered_resource"
        }
    };
    let mut tree = crate::tree_viewer::TreeNode::new(&node.id, &node.label).node_type(node_type);
    for child in &node.children {
        tree = tree.child(build_application_node(child, registry, seen, project)?);
    }
    if let AppPanelNodeKind::Directory {
        relative_path,
        filter,
    } = &node.kind
    {
        let project = project.ok_or(AppPanelTreeError::ProjectContextRequired)?;
        if relative_path.as_os_str().is_empty()
            || relative_path
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
        {
            return Err(AppPanelTreeError::InvalidDirectoryPath(
                relative_path.clone(),
            ));
        }
        let root = std::fs::canonicalize(&project.root)
            .map_err(|_| AppPanelTreeError::DirectoryReadFailed(project.root.clone()))?;
        let path = project.root.join(relative_path);
        if path.exists() {
            let canonical = std::fs::canonicalize(&path)
                .map_err(|_| AppPanelTreeError::DirectoryReadFailed(relative_path.clone()))?;
            if !canonical.starts_with(&root) || !canonical.is_dir() {
                return Err(AppPanelTreeError::InvalidDirectoryPath(
                relative_path.clone(),
            ));
            }
            append_directory(&mut tree, &canonical, &root, &node.id, filter, seen)?;
        }
    }
    Ok(tree)
}

fn append_directory(
    parent: &mut crate::tree_viewer::TreeNode,
    directory: &Path,
    root: &Path,
    mount_id: &str,
    filter: &DirectoryFilter,
    seen: &mut std::collections::HashSet<String>,
) -> Result<(), AppPanelTreeError> {
    let mut entries = std::fs::read_dir(directory)
        .map_err(|_| AppPanelTreeError::DirectoryReadFailed(directory.to_path_buf()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| AppPanelTreeError::DirectoryReadFailed(directory.to_path_buf()))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if !filter.show_hidden && name.starts_with('.') {
            continue;
        }
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|_| AppPanelTreeError::DirectoryReadFailed(path.clone()))?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        let canonical = std::fs::canonicalize(&path)
            .map_err(|_| AppPanelTreeError::DirectoryReadFailed(path.clone()))?;
        if !canonical.starts_with(root) {
            continue;
        }
        let relative = canonical
            .strip_prefix(root)
            .map_err(|_| AppPanelTreeError::InvalidDirectoryPath(path.clone()))?;
        let id = format!("{mount_id}:{}", relative.display());
        if metadata.is_dir() {
            let mut child =
                crate::tree_viewer::TreeNode::new(&id, &name).node_type("project.directory_entry");
            if filter.recursive {
                append_directory(&mut child, &canonical, root, mount_id, filter, seen)?;
            }
            if filter.include_directories {
                if !seen.insert(id.clone()) {
                    return Err(AppPanelTreeError::DuplicateNodeId(id));
                }
                parent.children.push(child);
            } else if filter.recursive {
                parent.children.extend(child.children);
            }
        } else if metadata.is_file() && filter.include_files {
            let matches = filter.extensions.is_empty()
                || path
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|ext| {
                        filter.extensions.iter().any(|allowed| {
                            allowed.trim_start_matches('.').eq_ignore_ascii_case(ext)
                        })
                    });
            if matches {
                if !seen.insert(id.clone()) {
                    return Err(AppPanelTreeError::DuplicateNodeId(id));
                }
                parent.children.push(
                    crate::tree_viewer::TreeNode::new(id, name).node_type("project.file_entry"),
                );
            }
        }
    }
    Ok(())
}

/// Convert extended Tree Viewer events into application-facing events.
/// Context menus choose an action separately; ContextRequested itself is not dispatched.
pub fn app_action_from_tree_action(
    action: &crate::tree_viewer::TreeAction,
) -> Option<AppPanelAction> {
    match action {
        crate::tree_viewer::TreeAction::Selected { node_id } => Some(AppPanelAction::Selected {
            node_ids: vec![node_id.clone()],
        }),
        crate::tree_viewer::TreeAction::SelectionChanged { node_ids } => {
            Some(AppPanelAction::Selected {
                node_ids: node_ids.clone(),
            })
        }
        crate::tree_viewer::TreeAction::Activated { node_id } => Some(AppPanelAction::Activated {
            node_id: node_id.clone(),
        }),
        crate::tree_viewer::TreeAction::ContextRequested { .. } => None,
    }
}

/// Dispatch an application action and invoke the supported framework fallback
/// only if the application explicitly returns NotHandled.
pub fn dispatch_project_panel_action(
    provider: &mut impl ApplicationProjectPanelProvider,
    project: &ProjectContext,
    action: AppPanelAction,
    fallback: impl FnOnce(&AppPanelAction) -> AppPanelActionResult,
) -> AppPanelActionResult {
    match provider.handle_project_panel_action(project, action.clone()) {
        AppPanelActionResult::NotHandled => fallback(&action),
        result => result,
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProjectPanelLifecycle {
    project_root: Option<std::path::PathBuf>,
    invalidation: ProjectPanelInvalidation,
}

impl ProjectPanelLifecycle {
    /// A successful open or Save As changes the active project context.
    pub fn activate_project(&mut self, project: &ProjectContext) {
        if self.project_root.as_ref() != Some(&project.root) {
            self.invalidation.clear_on_project_change();
            self.project_root = Some(project.root.clone());
        }
        self.invalidation.invalidate_application_project_panel();
    }

    /// Drop pending work on close, including any project-specific state.
    pub fn close_project(&mut self) {
        self.project_root = None;
        self.invalidation.clear_on_project_change();
    }

    /// Called after the application project adapter finishes restoring its state.
    pub fn invalidate_application_project_panel(&mut self) {
        if self.project_root.is_some() {
            self.invalidation.invalidate_application_project_panel();
        }
    }

    pub fn take_refresh_request(&mut self) -> Option<ProjectContext> {
        if !self.invalidation.take_pending() {
            return None;
        }
        self.project_root.clone().map(ProjectContext::new)
    }
}

/// Thread-safe, cloneable signal for requesting a fresh application tree snapshot.
/// Repeated requests are coalesced until the framework consumes the signal.
#[derive(Debug, Clone, Default)]
pub struct ProjectPanelRefreshHandle {
    pending: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl ProjectPanelRefreshHandle {
    pub fn invalidate_application_project_panel(&self) {
        self.pending
            .store(true, std::sync::atomic::Ordering::Release);
    }

    pub fn take_pending(&self) -> bool {
        self.pending
            .swap(false, std::sync::atomic::Ordering::AcqRel)
    }
}
