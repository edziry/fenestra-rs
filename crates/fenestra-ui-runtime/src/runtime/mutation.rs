use std::fmt;
use std::slice;

use fenestra_ui_ir::prototype::{InvalidationSet, PropertyId, PropertyValue};

use crate::logical_tree::NodeId;

use super::fragment::FragmentId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PropertyChange {
    pub(crate) node: NodeId,
    pub(crate) property: PropertyId,
    pub(crate) old_value: PropertyValue,
    pub(crate) new_value: PropertyValue,
    pub(crate) invalidation: InvalidationSet,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ManifestItem {
    Node(NodeId),
    Fragment(FragmentId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KeyInsert {
    pub(crate) fragment: FragmentId,
    pub(crate) key: u64,
    pub(crate) root: NodeId,
    pub(crate) final_index: usize,
    pub(crate) created: Vec<ManifestItem>,
    pub(crate) invalidation: InvalidationSet,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KeyMove {
    pub(crate) fragment: FragmentId,
    pub(crate) key: u64,
    pub(crate) root: NodeId,
    pub(crate) old_index: usize,
    pub(crate) final_index: usize,
    pub(crate) invalidation: InvalidationSet,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KeyRemove {
    pub(crate) fragment: FragmentId,
    pub(crate) key: u64,
    pub(crate) root: NodeId,
    pub(crate) old_index: usize,
    pub(crate) retired: Vec<ManifestItem>,
    pub(crate) invalidation: InvalidationSet,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum MutationRecord {
    PropertyChanged(PropertyChange),
    KeyInserted(KeyInsert),
    KeyMoved(KeyMove),
    KeyRemoved(KeyRemove),
}

impl MutationRecord {
    pub(crate) fn invalidation(&self) -> InvalidationSet {
        match self {
            Self::PropertyChanged(change) => change.invalidation,
            Self::KeyInserted(insert) => insert.invalidation,
            Self::KeyMoved(movement) => movement.invalidation,
            Self::KeyRemoved(removal) => removal.invalidation,
        }
    }

    pub(crate) fn is_effective(&self) -> bool {
        match self {
            Self::PropertyChanged(change) => change.old_value != change.new_value,
            Self::KeyInserted(_) | Self::KeyMoved(_) | Self::KeyRemoved(_) => true,
        }
    }
}

/// Typed borrowed view of one ordered mutation record.
#[derive(Clone, Copy)]
pub enum MutationRecordView<'a> {
    /// A typed property slot changed.
    PropertyChanged(PropertyChangeView<'a>),
    /// A keyed member and its factory expansion were created.
    KeyInserted(KeyInsertView<'a>),
    /// A keyed member moved within its owning fragment.
    KeyMoved(KeyMoveView<'a>),
    /// A keyed member and its nested expansion were retired.
    KeyRemoved(KeyRemoveView<'a>),
}

impl fmt::Debug for MutationRecordView<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = match self {
            Self::PropertyChanged(_) => "PropertyChanged",
            Self::KeyInserted(_) => "KeyInserted",
            Self::KeyMoved(_) => "KeyMoved",
            Self::KeyRemoved(_) => "KeyRemoved",
        };
        formatter.write_str(kind)
    }
}

/// Borrowed payload for a typed property change.
#[derive(Clone, Copy)]
pub struct PropertyChangeView<'a> {
    change: &'a PropertyChange,
}

impl<'a> PropertyChangeView<'a> {
    /// Returns the changed live node identity.
    #[must_use]
    pub const fn node(self) -> NodeId {
        self.change.node
    }

    /// Returns the component-local property symbol.
    #[must_use]
    pub const fn property(self) -> PropertyId {
        self.change.property
    }

    /// Returns the value before the transaction.
    #[must_use]
    pub const fn old_value(self) -> &'a PropertyValue {
        &self.change.old_value
    }

    /// Returns the final value after coalescing.
    #[must_use]
    pub const fn new_value(self) -> &'a PropertyValue {
        &self.change.new_value
    }
}

/// Borrowed payload for a keyed insertion.
#[derive(Clone, Copy)]
pub struct KeyInsertView<'a> {
    insert: &'a KeyInsert,
}

impl<'a> KeyInsertView<'a> {
    /// Returns the owning fragment.
    #[must_use]
    pub const fn fragment(self) -> FragmentId {
        self.insert.fragment
    }

    /// Returns the inserted key.
    #[must_use]
    pub const fn key(self) -> u64 {
        self.insert.key
    }

    /// Returns the created member root.
    #[must_use]
    pub const fn root(self) -> NodeId {
        self.insert.root
    }

    /// Returns the member's final local index.
    #[must_use]
    pub const fn final_index(self) -> usize {
        self.insert.final_index
    }

    /// Iterates every identity created by the factory expansion.
    pub fn created(self) -> ManifestIter<'a> {
        ManifestIter::new(&self.insert.created)
    }
}

/// Borrowed payload for a keyed move.
#[derive(Clone, Copy)]
pub struct KeyMoveView<'a> {
    movement: &'a KeyMove,
}

impl KeyMoveView<'_> {
    /// Returns the owning fragment.
    #[must_use]
    pub const fn fragment(self) -> FragmentId {
        self.movement.fragment
    }

    /// Returns the moved key.
    #[must_use]
    pub const fn key(self) -> u64 {
        self.movement.key
    }

    /// Returns the stable member root.
    #[must_use]
    pub const fn root(self) -> NodeId {
        self.movement.root
    }

    /// Returns the member's previous local index.
    #[must_use]
    pub const fn old_index(self) -> usize {
        self.movement.old_index
    }

    /// Returns the member's final local index.
    #[must_use]
    pub const fn final_index(self) -> usize {
        self.movement.final_index
    }
}

/// Borrowed payload for a keyed removal.
#[derive(Clone, Copy)]
pub struct KeyRemoveView<'a> {
    removal: &'a KeyRemove,
}

impl<'a> KeyRemoveView<'a> {
    /// Returns the owning fragment.
    #[must_use]
    pub const fn fragment(self) -> FragmentId {
        self.removal.fragment
    }

    /// Returns the removed key.
    #[must_use]
    pub const fn key(self) -> u64 {
        self.removal.key
    }

    /// Returns the retired member root.
    #[must_use]
    pub const fn root(self) -> NodeId {
        self.removal.root
    }

    /// Returns the member's previous local index.
    #[must_use]
    pub const fn old_index(self) -> usize {
        self.removal.old_index
    }

    /// Iterates every identity retired by the removed subtree.
    pub fn retired(self) -> ManifestIter<'a> {
        ManifestIter::new(&self.removal.retired)
    }
}

/// One identity in a deterministic structural lifecycle manifest.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManifestEntry {
    /// A logical node identity.
    Node(NodeId),
    /// A structural fragment identity.
    Fragment(FragmentId),
}

/// Iterator over a structural lifecycle manifest.
pub struct ManifestIter<'a> {
    items: slice::Iter<'a, ManifestItem>,
}

impl<'a> ManifestIter<'a> {
    fn new(items: &'a [ManifestItem]) -> Self {
        Self {
            items: items.iter(),
        }
    }
}

impl Iterator for ManifestIter<'_> {
    type Item = ManifestEntry;

    fn next(&mut self) -> Option<Self::Item> {
        Some(match self.items.next()? {
            ManifestItem::Node(node) => ManifestEntry::Node(*node),
            ManifestItem::Fragment(fragment) => ManifestEntry::Fragment(*fragment),
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.items.size_hint()
    }
}

impl ExactSizeIterator for ManifestIter<'_> {}

/// Iterator over the ordered typed mutation log.
pub struct MutationIter<'a> {
    records: slice::Iter<'a, MutationRecord>,
}

impl<'a> MutationIter<'a> {
    pub(crate) fn new(records: &'a [MutationRecord]) -> Self {
        Self {
            records: records.iter(),
        }
    }
}

impl<'a> Iterator for MutationIter<'a> {
    type Item = MutationRecordView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(match self.records.next()? {
            MutationRecord::PropertyChanged(change) => {
                MutationRecordView::PropertyChanged(PropertyChangeView { change })
            }
            MutationRecord::KeyInserted(insert) => {
                MutationRecordView::KeyInserted(KeyInsertView { insert })
            }
            MutationRecord::KeyMoved(movement) => {
                MutationRecordView::KeyMoved(KeyMoveView { movement })
            }
            MutationRecord::KeyRemoved(removal) => {
                MutationRecordView::KeyRemoved(KeyRemoveView { removal })
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.records.size_hint()
    }
}

impl ExactSizeIterator for MutationIter<'_> {}
