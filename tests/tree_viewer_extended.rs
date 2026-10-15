use workflow_ide_framework::tree_viewer::{TreeModel, TreeNode, TreeSelection};

#[test]
fn pp_hyb_013_multi_selection_preserves_distinct_nodes() {
    let mut selection = TreeSelection::default();
    selection.select("robot01", false);
    selection.select("sysid01", true);
    selection.select("robot01", true);
    assert_eq!(selection.node_ids, vec!["robot01", "sysid01"]);
    selection.select("robot02", false);
    assert_eq!(selection.node_ids, vec!["robot02"]);
}

#[test]
fn pp_hyb_012_selection_prunes_removed_project_nodes() {
    let mut selection = TreeSelection::default();
    selection.select("old-robot", false);
    selection.select("new-robot", true);
    let tree = TreeModel {
        roots: vec![TreeNode::new("new-robot", "New robot")],
        selected_id: None,
    };
    selection.retain_existing(&tree);
    assert_eq!(selection.node_ids, vec!["new-robot"]);
}
