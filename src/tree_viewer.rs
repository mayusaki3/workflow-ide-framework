use eframe::egui;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: String,
    pub label: String,
    pub node_type: Option<String>,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self { id: id.into(), label: label.into(), node_type: None, children: Vec::new() }
    }

    pub fn node_type(mut self, node_type: impl Into<String>) -> Self {
        self.node_type = Some(node_type.into());
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
}

#[derive(Debug, Default)]
pub struct TreeResponse {
    pub action: Option<TreeAction>,
}

pub fn show(ui: &mut egui::Ui, model: &mut TreeModel) -> TreeResponse {
    let mut response = TreeResponse::default();
    let roots = model.roots.clone();
    for node in &roots {
        show_node(ui, node, model, &mut response);
    }
    response
}

fn show_node(ui: &mut egui::Ui, node: &TreeNode, model: &mut TreeModel, response: &mut TreeResponse) {
    if node.children.is_empty() {
        let selected = model.selected_id.as_deref() == Some(node.id.as_str());
        if ui.selectable_label(selected, &node.label).clicked() {
            model.selected_id = Some(node.id.clone());
            response.action = Some(TreeAction::Selected { node_id: node.id.clone() });
        }
        return;
    }

    let header = egui::CollapsingHeader::new(&node.label)
        .id_salt((&node.id, "wfide_tree"))
        .show(ui, |ui| {
            for child in &node.children {
                show_node(ui, child, model, response);
            }
        });

    if header.header_response.clicked() {
        model.selected_id = Some(node.id.clone());
        response.action = Some(TreeAction::Selected { node_id: node.id.clone() });
    }
}
