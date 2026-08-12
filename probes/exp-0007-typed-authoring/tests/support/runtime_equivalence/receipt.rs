use fenestra_ui_ir::prototype::{
    InvalidationSet, PropertyId, PropertyValue, ValidatedConstruction,
};
use fenestra_ui_runtime::prototype::{
    CommitReceipt, CommittedRuntimeSnapshot, HeadlessSurface, ManifestEntry, MutationRecordView,
};
use fenestra_ui_testkit::prototype::{FragmentPathV1, NodePathV1};

use super::identity::BoundIdentities;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NormalizedManifestEntry {
    Node(NodePathV1),
    Fragment(FragmentPathV1),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NormalizedMutation {
    PropertyChanged {
        node: NodePathV1,
        property: PropertyId,
        old_value: PropertyValue,
        new_value: PropertyValue,
    },
    KeyInserted {
        fragment: FragmentPathV1,
        key: u64,
        root: NodePathV1,
        final_index: usize,
        created: Vec<NormalizedManifestEntry>,
    },
    KeyMoved {
        fragment: FragmentPathV1,
        key: u64,
        root: NodePathV1,
        old_index: usize,
        final_index: usize,
    },
    KeyRemoved {
        fragment: FragmentPathV1,
        key: u64,
        root: NodePathV1,
        old_index: usize,
        retired: Vec<NormalizedManifestEntry>,
    },
    HeadlessSurfaceChanged {
        old_surface: HeadlessSurface,
        new_surface: HeadlessSurface,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedReceipt {
    generation: u64,
    mutations: Vec<NormalizedMutation>,
    invalidation: InvalidationSet,
}

impl NormalizedReceipt {
    pub const fn new(
        generation: u64,
        mutations: Vec<NormalizedMutation>,
        invalidation: InvalidationSet,
    ) -> Self {
        Self {
            generation,
            mutations,
            invalidation,
        }
    }

    #[allow(dead_code)]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    #[allow(dead_code)]
    pub fn mutations(&self) -> &[NormalizedMutation] {
        &self.mutations
    }

    #[allow(dead_code)]
    pub const fn invalidation(&self) -> InvalidationSet {
        self.invalidation
    }
}

pub(super) fn normalize_receipt(
    construction: &ValidatedConstruction,
    before: &CommittedRuntimeSnapshot,
    after: &CommittedRuntimeSnapshot,
    receipt: &CommitReceipt,
) -> Result<NormalizedReceipt, &'static str> {
    if receipt.generation() != after.generation() {
        return Err("runtime-receipt-generation");
    }
    let before_ids = BoundIdentities::bind(construction, before)?;
    let after_ids = BoundIdentities::bind(construction, after)?;
    let identities = TransitionIdentities {
        before: &before_ids,
        after: &after_ids,
    };
    let mutations = receipt
        .mutations()
        .map(|record| normalize_mutation(&identities, record))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(NormalizedReceipt::new(
        receipt.generation().get(),
        mutations,
        receipt.invalidation(),
    ))
}

struct TransitionIdentities<'a> {
    before: &'a BoundIdentities,
    after: &'a BoundIdentities,
}

impl TransitionIdentities<'_> {
    fn node(
        &self,
        node: fenestra_ui_runtime::prototype::NodeId,
    ) -> Result<NodePathV1, &'static str> {
        match (self.before.node_path(node), self.after.node_path(node)) {
            (Some(before), Some(after)) if before == after => Ok(before.clone()),
            (Some(path), None) | (None, Some(path)) => Ok(path.clone()),
            _ => Err("runtime-node-path"),
        }
    }

    fn fragment(
        &self,
        fragment: fenestra_ui_runtime::prototype::FragmentId,
    ) -> Result<FragmentPathV1, &'static str> {
        match (
            self.before.fragment_path(fragment),
            self.after.fragment_path(fragment),
        ) {
            (Some(before), Some(after)) if before == after => Ok(before.clone()),
            (Some(path), None) | (None, Some(path)) => Ok(path.clone()),
            _ => Err("runtime-fragment-path"),
        }
    }

    fn manifest(
        &self,
        entries: impl Iterator<Item = ManifestEntry>,
    ) -> Result<Vec<NormalizedManifestEntry>, &'static str> {
        entries
            .map(|entry| match entry {
                ManifestEntry::Node(node) => self.node(node).map(NormalizedManifestEntry::Node),
                ManifestEntry::Fragment(fragment) => self
                    .fragment(fragment)
                    .map(NormalizedManifestEntry::Fragment),
            })
            .collect()
    }
}

fn normalize_mutation(
    identities: &TransitionIdentities<'_>,
    record: MutationRecordView<'_>,
) -> Result<NormalizedMutation, &'static str> {
    Ok(match record {
        MutationRecordView::PropertyChanged(change) => NormalizedMutation::PropertyChanged {
            node: identities.node(change.node())?,
            property: change.property(),
            old_value: change.old_value().clone(),
            new_value: change.new_value().clone(),
        },
        MutationRecordView::KeyInserted(insert) => NormalizedMutation::KeyInserted {
            fragment: identities.fragment(insert.fragment())?,
            key: insert.key(),
            root: identities.node(insert.root())?,
            final_index: insert.final_index(),
            created: identities.manifest(insert.created())?,
        },
        MutationRecordView::KeyMoved(movement) => NormalizedMutation::KeyMoved {
            fragment: identities.fragment(movement.fragment())?,
            key: movement.key(),
            root: identities.node(movement.root())?,
            old_index: movement.old_index(),
            final_index: movement.final_index(),
        },
        MutationRecordView::KeyRemoved(removal) => NormalizedMutation::KeyRemoved {
            fragment: identities.fragment(removal.fragment())?,
            key: removal.key(),
            root: identities.node(removal.root())?,
            old_index: removal.old_index(),
            retired: identities.manifest(removal.retired())?,
        },
        MutationRecordView::HeadlessSurfaceChanged(change) => {
            NormalizedMutation::HeadlessSurfaceChanged {
                old_surface: change.old_surface(),
                new_surface: change.new_surface(),
            }
        }
        MutationRecordView::SpatialViewportChanged(_) => {
            return Err("runtime-spatial-viewport");
        }
    })
}
