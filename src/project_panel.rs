use crate::{project_resource::ResourceEntry, resource_registry::ResourceRegistry, tree_viewer::{TreeModel, TreeNode}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceProviderKind {
    Framework,
    Application,
    Project,
    External,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvidedResource {
    pub id: String,
    pub label: String,
    pub logical_path: std::path::PathBuf,
    pub provider: ResourceProviderKind,
}

impl ProvidedResource {
    pub fn new(id: impl Into<String>, label: impl Into<String>, logical_path: impl Into<std::path::PathBuf>, provider: ResourceProviderKind) -> Self {
        Self { id: id.into(), label: label.into(), logical_path: logical_path.into(), provider }
    }
}

#[derive(Debug, Clone)]
pub enum ProjectPanelNode {
    Logical { id: String, label: String, children: Vec<ProjectPanelNode> },
    Resources { id: String, label: String, providers: Vec<ResourceProviderKind> },
}

impl ProjectPanelNode {
    pub fn logical(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self::Logical { id: id.into(), label: label.into(), children: Vec::new() }
    }

    pub fn resources(id: impl Into<String>, label: impl Into<String>, providers: impl IntoIterator<Item = ResourceProviderKind>) -> Self {
        Self::Resources { id: id.into(), label: label.into(), providers: providers.into_iter().collect() }
    }

    pub fn child(mut self, child: ProjectPanelNode) -> Self {
        if let Self::Logical { children, .. } = &mut self { children.push(child); }
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct ProjectPanelModel {
    pub roots: Vec<ProjectPanelNode>,
    pub framework_resources: Vec<ProvidedResource>,
    pub application_resources: Vec<ProvidedResource>,
}

impl ProjectPanelModel {
    pub fn tree(&self, registry: &ResourceRegistry) -> TreeModel {
        let resources = self.all_resources(registry);
        TreeModel {
            roots: self.roots.iter().map(|node| build_node(node, &resources)).collect(),
            selected_id: None,
        }
    }

    fn all_resources(&self, registry: &ResourceRegistry) -> Vec<ProvidedResource> {
        let mut resources = self.framework_resources.clone();
        resources.extend(self.application_resources.clone());
        resources.extend(registry.entries().iter().map(resource_from_registry));
        resources
    }
}

fn resource_from_registry(entry: &ResourceEntry) -> ProvidedResource {
    use crate::project_resource::ResourceScope;
    let provider = match entry.reference.scope {
        ResourceScope::Application => ResourceProviderKind::Application,
        ResourceScope::Project => ResourceProviderKind::Project,
        ResourceScope::External => ResourceProviderKind::External,
    };
    let label = entry.reference.path.file_name().map(|value| value.to_string_lossy().into_owned()).unwrap_or_else(|| entry.reference.path.display().to_string());
    ProvidedResource::new(entry.resource_id.clone(), label, entry.reference.path.clone(), provider)
}

fn build_node(node: &ProjectPanelNode, resources: &[ProvidedResource]) -> TreeNode {
    match node {
        ProjectPanelNode::Logical { id, label, children } => {
            children.iter().fold(TreeNode::new(id, label).node_type("project.logical"), |tree, child| tree.child(build_node(child, resources)))
        }
        ProjectPanelNode::Resources { id, label, providers } => {
            let mut matching = resources.iter().filter(|resource| providers.contains(&resource.provider)).cloned().collect::<Vec<_>>();
            matching.sort_by(|a, b| a.logical_path.cmp(&b.logical_path).then(a.label.cmp(&b.label)));
            matching.into_iter().fold(TreeNode::new(id, label).node_type("project.resources"), |tree, resource| {
                tree.child(TreeNode::new(format!("resource:{}", resource.id), resource.label).node_type(format!("resource.{:?}", resource.provider).to_ascii_lowercase()))
            })
        }
    }
}
