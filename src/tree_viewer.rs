use eframe::egui;

/// Generic visual severity, independent of Project Panel diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeNodeStatus {
    Normal,
    Warning,
    Error,
    Unknown,
}

/// Optional presentation metadata. Absence preserves the ordinary tree label.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TreeNodePresentation {
    pub icon: Option<String>,
    pub tooltip: Option<String>,
    pub status: Option<TreeNodeStatus>,
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: String,
    pub label: String,
    pub node_type: Option<String>,
    pub presentation: TreeNodePresentation,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            node_type: None,
            presentation: TreeNodePresentation::default(),
            children: Vec::new(),
        }
    }

    pub fn node_type(mut self, node_type: impl Into<String>) -> Self {
        self.node_type = Some(node_type.into());
        self
    }

    /// Attach generic presentation data without changing node identity.
    pub fn presentation(mut self, presentation: TreeNodePresentation) -> Self {
        self.presentation = presentation;
        self
    }

    pub fn child(mut self, child: TreeNode) -> Self {
        self.children.push(child);
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct TreeModel {
    pub roots: Vec<TreeNode>,
    pub selected_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeAction {
    Selected { node_id: String },
    SelectionChanged { node_ids: Vec<String> },
    Activated { node_id: String },
    ContextRequested { node_ids: Vec<String> },
}

#[derive(Debug, Default)]
pub struct TreeResponse {
    pub actions: Vec<TreeAction>,
}

impl TreeResponse {
    fn selected(&mut self, node_id: &str) {
        self.actions.push(TreeAction::Selected {
            node_id: node_id.to_owned(),
        });
    }
}

/// Maintains multi-selection independently from the legacy single selected_id field.
#[derive(Debug, Clone, Default)]
pub struct TreeSelection {
    pub node_ids: Vec<String>,
}

impl TreeSelection {
    pub fn select(&mut self, node_id: &str, additive: bool) {
        if !additive {
            self.node_ids.clear();
        }
        if !self.node_ids.iter().any(|id| id == node_id) {
            self.node_ids.push(node_id.to_owned());
        }
    }

    pub fn retain_existing(&mut self, model: &TreeModel) {
        fn collect(node: &TreeNode, ids: &mut std::collections::HashSet<String>) {
            ids.insert(node.id.clone());
            for child in &node.children {
                collect(child, ids);
            }
        }
        let mut valid = std::collections::HashSet::new();
        for root in &model.roots {
            collect(root, &mut valid);
        }
        self.node_ids.retain(|id| valid.contains(id));
    }
}

pub fn show(ui: &mut egui::Ui, model: &mut TreeModel) -> TreeResponse {
    let mut response = TreeResponse::default();
    let roots = model.roots.clone();
    for node in &roots {
        show_node(ui, node, model, &mut response);
    }
    response
}

fn show_node(
    ui: &mut egui::Ui,
    node: &TreeNode,
    model: &mut TreeModel,
    response: &mut TreeResponse,
) {
    if node.children.is_empty() {
        let selected = model.selected_id.as_deref() == Some(node.id.as_str());
        if ui.selectable_label(selected, node_display_label(node)).clicked() {
            model.selected_id = Some(node.id.clone());
            response.selected(&node.id);
        }
        return;
    }

    let header = egui::CollapsingHeader::new(node_display_label(node))
        .id_salt((&node.id, "wfide_tree"))
        .show(ui, |ui| {
            for child in &node.children {
                show_node(ui, child, model, response);
            }
        });

    if header.header_response.clicked() {
        model.selected_id = Some(node.id.clone());
        response.selected(&node.id);
    }
}

/// Extended interaction mode for application-owned project panels.
/// The legacy show() function and TreeAction::Selected remain unchanged.
pub fn show_with_selection(
    ui: &mut egui::Ui,
    model: &mut TreeModel,
    selection: &mut TreeSelection,
) -> TreeResponse {
    let mut response = TreeResponse::default();
    selection.retain_existing(model);
    let roots = model.roots.clone();
    for root in &roots {
        show_node_extended(ui, root, model, selection, &mut response);
    }
    response
}

fn show_node_extended(
    ui: &mut egui::Ui,
    node: &TreeNode,
    model: &mut TreeModel,
    selection: &mut TreeSelection,
    response: &mut TreeResponse,
) {
    let selected = selection.node_ids.contains(&node.id);
    let header = if node.children.is_empty() {
        ui.selectable_label(selected, &node.label)
    } else {
        let result = egui::CollapsingHeader::new(&node.label)
            .id_salt((&node.id, "wfide_tree_extended"))
            .show(ui, |ui| {
                for child in &node.children {
                    show_node_extended(ui, child, model, selection, response);
                }
            });
        result.header_response
    };
    let header = if let Some(tooltip) = &node.presentation.tooltip {
        header.on_hover_text(tooltip)
    } else {
        header
    };
    if header.clicked() {
        let additive = ui.input(|input| input.modifiers.command || input.modifiers.ctrl);
        selection.select(&node.id, additive);
        model.selected_id = Some(node.id.clone());
        response.selected(&node.id);
        response.actions.push(TreeAction::SelectionChanged {
            node_ids: selection.node_ids.clone(),
        });
    }
    if header.double_clicked() {
        response.actions.push(TreeAction::Activated {
            node_id: node.id.clone(),
        });
    }
    if header.secondary_clicked() {
        if !selection.node_ids.contains(&node.id) {
            selection.select(&node.id, false);
            model.selected_id = Some(node.id.clone());
        }
        response.actions.push(TreeAction::ContextRequested {
            node_ids: selection.node_ids.clone(),
        });
    }
}

fn node_display_label(node: &TreeNode) -> String {
    match &node.presentation.icon {
        Some(icon) => format!("{icon} {}", node.label),
        None => node.label.clone(),
    }
}
