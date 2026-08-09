use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::arena::{Arena, ArenaId};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct TreeId(u64);

impl TreeId {
    fn next() -> Self {
        static NEXT_TREE_ID: AtomicU64 = AtomicU64::new(1);

        let id = NEXT_TREE_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .unwrap_or_else(|_| panic!("logical tree identity space exhausted"));
        Self(id)
    }
}

/// Opaque generational identity for one logical node.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct NodeId {
    tree: TreeId,
    arena: ArenaId,
}

impl fmt::Debug for NodeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("NodeId(..)")
    }
}

/// Typed failures for logical tree mutations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TreeError {
    /// A root already exists and must be removed before another is inserted.
    RootAlreadyExists(NodeId),
    /// The requested node is absent or its generation is stale.
    MissingNode(NodeId),
}

impl fmt::Display for TreeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RootAlreadyExists(root) => {
                write!(formatter, "logical tree already has root {root:?}")
            }
            Self::MissingNode(node) => write!(formatter, "logical node {node:?} is missing"),
        }
    }
}

impl Error for TreeError {}

/// Failure reported when internal logical tree links are inconsistent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TreeInvariantError {
    message: &'static str,
}

impl TreeInvariantError {
    const fn new(message: &'static str) -> Self {
        Self { message }
    }

    /// Returns the stable prototype diagnostic message.
    #[must_use]
    pub const fn message(self) -> &'static str {
        self.message
    }
}

impl fmt::Display for TreeInvariantError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message)
    }
}

impl Error for TreeInvariantError {}

#[derive(Debug)]
struct LogicalNode<T> {
    value: T,
    parent: Option<NodeId>,
    children: Vec<NodeId>,
}

/// Experimental rooted logical tree with generational node identities.
pub struct LogicalTree<T> {
    id: TreeId,
    nodes: Arena<LogicalNode<T>>,
    root: Option<NodeId>,
}

impl<T> LogicalTree<T> {
    /// Creates an empty logical tree.
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: TreeId::next(),
            nodes: Arena::new(),
            root: None,
        }
    }

    /// Inserts the only root allowed by this prototype.
    pub fn insert_root(&mut self, value: T) -> Result<NodeId, TreeError> {
        if let Some(root) = self.root {
            return Err(TreeError::RootAlreadyExists(root));
        }

        let arena = self.nodes.insert(LogicalNode {
            value,
            parent: None,
            children: Vec::new(),
        });
        let root = NodeId {
            tree: self.id,
            arena,
        };
        self.root = Some(root);
        Ok(root)
    }

    /// Appends a child to a live parent.
    pub fn append_child(&mut self, parent: NodeId, value: T) -> Result<NodeId, TreeError> {
        if self.get_node(parent).is_none() {
            return Err(TreeError::MissingNode(parent));
        }

        let arena = self.nodes.insert(LogicalNode {
            value,
            parent: Some(parent),
            children: Vec::new(),
        });
        let child = NodeId {
            tree: self.id,
            arena,
        };
        self.get_node_mut(parent)
            .expect("validated parent must remain present")
            .children
            .push(child);
        Ok(child)
    }

    /// Removes a node and every descendant, invalidating all of their handles.
    pub fn remove_subtree(&mut self, node: NodeId) -> Result<(), TreeError> {
        let stored = self.get_node(node).ok_or(TreeError::MissingNode(node))?;
        let parent = stored.parent;

        let mut removal_order = Vec::new();
        let mut pending = vec![node];
        while let Some(current) = pending.pop() {
            let current_node = self
                .get_node(current)
                .expect("reachable child must remain present");
            removal_order.push(current);
            pending.extend(current_node.children.iter().copied());
        }
        let mut retired = Vec::with_capacity(removal_order.len());

        match parent {
            Some(parent) => {
                let parent_node = self
                    .get_node_mut(parent)
                    .expect("live node must have a live parent");
                parent_node.children.retain(|child| *child != node);
            }
            None => self.root = None,
        }

        for current in removal_order.into_iter().rev() {
            retired.push(
                self.nodes
                    .remove(current.arena)
                    .expect("collected node must remain present"),
            );
        }
        drop(retired);

        Ok(())
    }

    /// Returns the value for a live node.
    #[must_use]
    pub fn value(&self, node: NodeId) -> Option<&T> {
        self.get_node(node).map(|stored| &stored.value)
    }

    /// Returns the parent for a live non-root node.
    #[must_use]
    pub fn parent(&self, node: NodeId) -> Option<NodeId> {
        self.get_node(node).and_then(|stored| stored.parent)
    }

    /// Returns the ordered child identities for a live node.
    #[must_use]
    pub fn children(&self, node: NodeId) -> Option<&[NodeId]> {
        self.get_node(node).map(|stored| stored.children.as_slice())
    }

    /// Returns the current root identity.
    #[must_use]
    pub const fn root(&self) -> Option<NodeId> {
        self.root
    }

    /// Returns the number of live nodes.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns whether the tree contains no live nodes.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn get_node(&self, node: NodeId) -> Option<&LogicalNode<T>> {
        if node.tree != self.id {
            return None;
        }
        self.nodes.get(node.arena)
    }

    fn get_node_mut(&mut self, node: NodeId) -> Option<&mut LogicalNode<T>> {
        if node.tree != self.id {
            return None;
        }
        self.nodes.get_mut(node.arena)
    }

    /// Validates root reachability and reciprocal parent-child links.
    pub fn validate(&self) -> Result<(), TreeInvariantError> {
        let Some(root) = self.root else {
            return (self.nodes.len() == 0)
                .then_some(())
                .ok_or(TreeInvariantError::new("nodes exist without a root"));
        };

        let root_node = self
            .get_node(root)
            .ok_or(TreeInvariantError::new("root identity is stale"))?;
        if root_node.parent.is_some() {
            return Err(TreeInvariantError::new("root has a parent"));
        }

        let mut visited = HashSet::new();
        let mut pending = vec![root];
        while let Some(parent) = pending.pop() {
            if !visited.insert(parent) {
                return Err(TreeInvariantError::new("node is reachable more than once"));
            }

            let parent_node = self
                .get_node(parent)
                .ok_or(TreeInvariantError::new("reachable node is stale"))?;
            for child in &parent_node.children {
                let child_node = self
                    .get_node(*child)
                    .ok_or(TreeInvariantError::new("child identity is stale"))?;
                if child_node.parent != Some(parent) {
                    return Err(TreeInvariantError::new(
                        "child does not point to its parent",
                    ));
                }
                pending.push(*child);
            }
        }

        if visited.len() != self.nodes.len() {
            return Err(TreeInvariantError::new(
                "live nodes are unreachable from the root",
            ));
        }

        Ok(())
    }
}

impl<T> Default for LogicalTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
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
}
