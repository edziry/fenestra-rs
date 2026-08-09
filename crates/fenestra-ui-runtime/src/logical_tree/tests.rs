use super::LogicalTree;

fn tree_with_child() -> (LogicalTree<&'static str>, super::NodeId, super::NodeId) {
    let mut tree = LogicalTree::new();
    let root = tree.insert_root("root").expect("root should be inserted");
    let child = tree
        .append_child(root, "child")
        .expect("child should be inserted");
    (tree, root, child)
}

fn assert_invalid(tree: &LogicalTree<&str>, expected: &'static str) {
    assert_eq!(
        tree.validate()
            .expect_err("corrupted tree should be rejected")
            .message(),
        expected
    );
}

#[test]
fn validation_rejects_nodes_without_a_root() {
    let (mut tree, _, _) = tree_with_child();
    tree.root = None;

    assert_invalid(&tree, "nodes exist without a root");
}

#[test]
fn validation_rejects_a_stale_root() {
    let mut tree = LogicalTree::new();
    let root = tree.insert_root("root").expect("root should be inserted");
    tree.nodes
        .remove(root.arena)
        .expect("root should be removed from the arena");

    assert_invalid(&tree, "root identity is stale");
}

#[test]
fn validation_rejects_a_root_with_a_parent() {
    let (mut tree, root, child) = tree_with_child();
    tree.nodes
        .get_mut(root.arena)
        .expect("root should be live")
        .parent = Some(child);

    assert_invalid(&tree, "root has a parent");
}

#[test]
fn validation_rejects_a_stale_child() {
    let (mut tree, _, child) = tree_with_child();
    tree.nodes
        .remove(child.arena)
        .expect("child should be removed from the arena");

    assert_invalid(&tree, "child identity is stale");
}

#[test]
fn validation_rejects_a_child_with_the_wrong_parent() {
    let (mut tree, _, child) = tree_with_child();
    tree.nodes
        .get_mut(child.arena)
        .expect("child should be live")
        .parent = None;

    assert_invalid(&tree, "child does not point to its parent");
}

#[test]
fn validation_rejects_duplicate_reachability() {
    let (mut tree, root, child) = tree_with_child();
    tree.nodes
        .get_mut(root.arena)
        .expect("root should be live")
        .children
        .push(child);

    assert_invalid(&tree, "node is reachable more than once");
}

#[test]
fn validation_rejects_unreachable_live_nodes() {
    let (mut tree, root, _) = tree_with_child();
    tree.nodes
        .get_mut(root.arena)
        .expect("root should be live")
        .children
        .clear();

    assert_invalid(&tree, "live nodes are unreachable from the root");
}

#[test]
fn transaction_reorder_uses_final_direct_child_positions() {
    let mut tree = LogicalTree::new();
    let root = tree.insert_root("root").unwrap();
    let first = tree.append_child(root, "first").unwrap();
    let middle = tree.append_child(root, "middle").unwrap();
    let last = tree.append_child(root, "last").unwrap();

    tree.reorder_direct_child(root, last, 0).unwrap();
    assert_eq!(tree.children(root), Some(&[last, first, middle][..]));
    tree.reorder_direct_child(root, last, 2).unwrap();
    assert_eq!(tree.children(root), Some(&[first, middle, last][..]));
    tree.reorder_direct_child(root, middle, 1).unwrap();
    assert_eq!(tree.children(root), Some(&[first, middle, last][..]));
}

#[test]
fn rejected_transaction_reorder_does_not_mutate_children() {
    let mut tree = LogicalTree::new();
    let root = tree.insert_root("root").unwrap();
    let child = tree.append_child(root, "child").unwrap();
    let mut foreign = LogicalTree::new();
    let foreign_root = foreign.insert_root("foreign").unwrap();
    let expected = tree.children(root).unwrap().to_vec();

    assert_eq!(tree.reorder_direct_child(root, foreign_root, 0), Err(()));
    assert_eq!(tree.reorder_direct_child(child, child, 0), Err(()));
    assert_eq!(tree.reorder_direct_child(root, child, 1), Err(()));
    assert_eq!(tree.children(root), Some(expected.as_slice()));
}
