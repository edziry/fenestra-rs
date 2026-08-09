use fenestra_ui_runtime::prototype::{LogicalTree, TreeError};
use std::collections::{HashMap, HashSet};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[test]
fn stale_handle_is_rejected_after_slot_reuse() {
    let mut tree = LogicalTree::new();
    let root = tree.insert_root("root").expect("root should be inserted");
    let retired = tree
        .append_child(root, "retired")
        .expect("child should be inserted");

    tree.remove_subtree(retired)
        .expect("child should be removed");
    let replacement = tree
        .append_child(root, "replacement")
        .expect("replacement should be inserted");

    assert_ne!(retired, replacement);
    assert_eq!(tree.value(retired), None);
    assert_eq!(tree.value(replacement), Some(&"replacement"));
    tree.validate().expect("tree should remain valid");
}

#[test]
fn removing_subtree_invalidates_every_descendant() {
    let mut tree = LogicalTree::new();
    let root = tree.insert_root("root").expect("root should be inserted");
    let branch = tree
        .append_child(root, "branch")
        .expect("branch should be inserted");
    let leaf = tree
        .append_child(branch, "leaf")
        .expect("leaf should be inserted");

    tree.remove_subtree(branch)
        .expect("branch should be removed");

    assert!(tree.value(branch).is_none());
    assert!(tree.value(leaf).is_none());
    assert_eq!(tree.children(root), Some(&[][..]));
    assert_eq!(tree.len(), 1);
    tree.validate().expect("tree should remain valid");
}

#[test]
fn unrelated_mutations_preserve_surviving_identity() {
    let mut tree = LogicalTree::new();
    let root = tree.insert_root("root").expect("root should be inserted");
    let removed = tree
        .append_child(root, "removed")
        .expect("first child should be inserted");
    let survivor = tree
        .append_child(root, "survivor")
        .expect("second child should be inserted");

    tree.remove_subtree(removed)
        .expect("first child should be removed");
    let inserted = tree
        .append_child(root, "inserted")
        .expect("new child should be inserted");

    assert_eq!(tree.value(survivor), Some(&"survivor"));
    assert_eq!(tree.parent(survivor), Some(root));
    assert_eq!(tree.children(root), Some(&[survivor, inserted][..]));
    tree.validate().expect("tree should remain valid");
}

#[test]
fn invalid_insertions_return_typed_errors_without_mutation() {
    let mut tree = LogicalTree::new();
    let root = tree.insert_root("root").expect("root should be inserted");
    let retired = tree
        .append_child(root, "retired")
        .expect("child should be inserted");
    tree.remove_subtree(retired)
        .expect("child should be removed");

    assert_eq!(
        tree.insert_root("second root"),
        Err(TreeError::RootAlreadyExists(root))
    );
    assert_eq!(
        tree.append_child(retired, "orphan"),
        Err(TreeError::MissingNode(retired))
    );
    assert_eq!(tree.len(), 1);
    assert_eq!(tree.root(), Some(root));
    tree.validate().expect("tree should remain valid");
}

#[test]
fn removing_root_allows_a_new_distinct_root() {
    let mut tree = LogicalTree::new();
    let first_root = tree
        .insert_root("first root")
        .expect("root should be inserted");
    tree.append_child(first_root, "child")
        .expect("child should be inserted");

    tree.remove_subtree(first_root)
        .expect("root should be removed");
    assert!(tree.is_empty());
    assert_eq!(tree.root(), None);

    let second_root = tree
        .insert_root("second root")
        .expect("new root should be inserted");

    assert_ne!(first_root, second_root);
    assert_eq!(tree.value(first_root), None);
    assert_eq!(tree.value(second_root), Some(&"second root"));
    tree.validate().expect("tree should remain valid");
}

#[test]
fn handles_are_scoped_to_their_originating_tree() {
    let mut first = LogicalTree::new();
    let first_root = first
        .insert_root("first")
        .expect("first root should be inserted");
    let mut second = LogicalTree::new();
    let second_root = second
        .insert_root("second")
        .expect("second root should be inserted");

    assert_ne!(first_root, second_root);
    assert_eq!(second.value(first_root), None);
    assert_eq!(second.children(first_root), None);
    assert_eq!(second.parent(first_root), None);
    assert_eq!(
        second.append_child(first_root, "foreign child"),
        Err(TreeError::MissingNode(first_root))
    );
    assert_eq!(
        second.remove_subtree(first_root),
        Err(TreeError::MissingNode(first_root))
    );
    assert_eq!(second.root(), Some(second_root));
    assert_eq!(second.len(), 1);
    second.validate().expect("second tree should remain valid");
}

#[test]
fn node_debug_representation_keeps_arena_coordinates_private() {
    let mut tree = LogicalTree::new();
    let root = tree.insert_root("root").expect("root should be inserted");

    assert_eq!(format!("{root:?}"), "NodeId(..)");
    assert_eq!(
        TreeError::MissingNode(root).to_string(),
        "logical node NodeId(..) is missing"
    );
}

#[test]
fn identities_do_not_repeat_when_a_tree_is_recreated() {
    let retired = {
        let mut tree = LogicalTree::new();
        tree.insert_root("retired")
            .expect("retired root should be inserted")
    };
    let mut replacement_tree = LogicalTree::new();
    let replacement = replacement_tree
        .insert_root("replacement")
        .expect("replacement root should be inserted");

    assert_ne!(retired, replacement);
    assert_eq!(replacement_tree.value(retired), None);
    assert_eq!(
        replacement_tree.remove_subtree(retired),
        Err(TreeError::MissingNode(retired))
    );
    replacement_tree
        .validate()
        .expect("replacement tree should remain valid");
}

#[derive(Debug)]
struct DropProbe {
    panic_once: Option<Arc<AtomicBool>>,
}

impl DropProbe {
    fn stable() -> Self {
        Self { panic_once: None }
    }

    fn panicking(flag: Arc<AtomicBool>) -> Self {
        Self {
            panic_once: Some(flag),
        }
    }
}

impl Drop for DropProbe {
    fn drop(&mut self) {
        let Some(flag) = &self.panic_once else {
            return;
        };

        if flag.swap(false, Ordering::SeqCst) {
            panic!("injected drop failure");
        }
    }
}

#[test]
fn subtree_retirement_finishes_before_removed_values_are_dropped() {
    let panic_once = Arc::new(AtomicBool::new(true));
    let mut tree = LogicalTree::new();
    let root = tree
        .insert_root(DropProbe::stable())
        .expect("root should be inserted");
    let branch = tree
        .append_child(root, DropProbe::stable())
        .expect("branch should be inserted");
    let leaf = tree
        .append_child(branch, DropProbe::panicking(Arc::clone(&panic_once)))
        .expect("leaf should be inserted");

    let result = catch_unwind(AssertUnwindSafe(|| tree.remove_subtree(branch)));

    assert!(result.is_err());
    assert!(tree.value(branch).is_none());
    assert!(tree.value(leaf).is_none());
    assert_eq!(tree.children(root), Some(&[][..]));
    assert_eq!(tree.len(), 1);
    tree.validate()
        .expect("tree should be valid after a value destructor panics");
}

#[test]
fn removing_a_stale_handle_returns_an_error_without_mutation() {
    let mut tree = LogicalTree::new();
    let root = tree.insert_root("root").expect("root should be inserted");
    let retired = tree
        .append_child(root, "retired")
        .expect("child should be inserted");
    tree.remove_subtree(retired)
        .expect("child should be removed");
    let replacement = tree
        .append_child(root, "replacement")
        .expect("replacement should be inserted");

    assert_eq!(
        tree.remove_subtree(retired),
        Err(TreeError::MissingNode(retired))
    );
    assert_eq!(tree.children(root), Some(&[replacement][..]));
    assert_eq!(tree.len(), 2);
    tree.validate().expect("tree should remain valid");
}

#[derive(Debug)]
struct ModelNode {
    value: u32,
    parent: Option<fenestra_ui_runtime::prototype::NodeId>,
    children: Vec<fenestra_ui_runtime::prototype::NodeId>,
}

#[test]
fn generated_branching_lifecycles_match_an_independent_model() {
    let mut tree = LogicalTree::new();
    let root = tree.insert_root(0).expect("root should be inserted");
    let mut model = HashMap::from([(
        root,
        ModelNode {
            value: 0,
            parent: None,
            children: Vec::new(),
        },
    )]);
    let mut live = vec![root];
    let mut retired = Vec::new();
    let mut state = 0x4d59_5df4_d0f3_3173_u64;

    for value in 1..=1_000 {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);

        if live.len() == 1 || !state.is_multiple_of(3) {
            let parent = live[state as usize % live.len()];
            let node = tree
                .append_child(parent, value)
                .expect("generated child should be inserted");
            model
                .get_mut(&parent)
                .expect("model parent should be live")
                .children
                .push(node);
            model.insert(
                node,
                ModelNode {
                    value,
                    parent: Some(parent),
                    children: Vec::new(),
                },
            );
            live.push(node);
        } else {
            let target = live[1 + state as usize % (live.len() - 1)];
            let parent = model
                .get(&target)
                .and_then(|node| node.parent)
                .expect("non-root model node should have a parent");
            model
                .get_mut(&parent)
                .expect("model parent should be live")
                .children
                .retain(|child| *child != target);

            let mut removed = HashSet::new();
            let mut pending = vec![target];
            while let Some(node) = pending.pop() {
                assert!(removed.insert(node));
                pending.extend(
                    model
                        .get(&node)
                        .expect("removed model node should be live")
                        .children
                        .iter()
                        .copied(),
                );
            }

            tree.remove_subtree(target)
                .expect("generated subtree should be removed");
            for node in &removed {
                model.remove(node);
                retired.push(*node);
            }
            live.retain(|node| !removed.contains(node));
        }

        assert_eq!(tree.len(), model.len());
        for (node, expected) in &model {
            assert_eq!(tree.value(*node), Some(&expected.value));
            assert_eq!(tree.parent(*node), expected.parent);
            assert_eq!(tree.children(*node), Some(expected.children.as_slice()));
        }
        assert!(retired.iter().all(|node| tree.value(*node).is_none()));
        tree.validate().expect("generated tree should remain valid");
    }
}
