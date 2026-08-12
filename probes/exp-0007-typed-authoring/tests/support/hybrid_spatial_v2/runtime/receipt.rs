use fenestra_ui_runtime::prototype::{
    CommitReceipt, CommittedRuntimeSnapshot, ManifestEntry, MutationRecordView,
};

use super::path::BoundIdentities;
use super::types::{NormalizedManifestEntry, NormalizedMutation, NormalizedReceipt};

pub(super) fn initial_receipt() -> NormalizedReceipt {
    NormalizedReceipt {
        generation: 0,
        invalidation: fenestra_ui_ir::prototype::InvalidationSet::NONE,
        mutations: Vec::new(),
    }
}

pub(super) fn normalize_receipt(
    construction: &fenestra_ui_ir::prototype::ValidatedConstruction,
    before: &CommittedRuntimeSnapshot,
    receipt: &CommitReceipt,
    after: &CommittedRuntimeSnapshot,
) -> NormalizedReceipt {
    let old = BoundIdentities::bind(construction, before);
    let new = BoundIdentities::bind(construction, after);
    let mutations = receipt
        .mutations()
        .map(|mutation| match mutation {
            MutationRecordView::PropertyChanged(change) => NormalizedMutation::Property {
                node: new.node_path(change.node()).clone(),
                property: change.property().get(),
                old: change.old_value().clone(),
                new: change.new_value().clone(),
            },
            MutationRecordView::KeyInserted(insert) => NormalizedMutation::Insert {
                fragment: new.fragment_path(insert.fragment()).clone(),
                key: insert.key(),
                root: new.node_path(insert.root()).clone(),
                final_index: insert.final_index(),
                created: insert
                    .created()
                    .map(|entry| normalize_manifest(entry, &new))
                    .collect(),
            },
            MutationRecordView::KeyMoved(movement) => NormalizedMutation::Move {
                fragment: new.fragment_path(movement.fragment()).clone(),
                key: movement.key(),
                root: new.node_path(movement.root()).clone(),
                old_index: movement.old_index(),
                final_index: movement.final_index(),
            },
            MutationRecordView::KeyRemoved(removal) => NormalizedMutation::Remove {
                fragment: old.fragment_path(removal.fragment()).clone(),
                key: removal.key(),
                root: old.node_path(removal.root()).clone(),
                old_index: removal.old_index(),
                retired: removal
                    .retired()
                    .map(|entry| normalize_manifest(entry, &old))
                    .collect(),
            },
            MutationRecordView::SpatialViewportChanged(change) => NormalizedMutation::Viewport {
                old: change.old_viewport(),
                new: change.new_viewport(),
            },
            MutationRecordView::HeadlessSurfaceChanged(_) => {
                panic!("a spatial runtime should not publish headless mutations")
            }
        })
        .collect();
    NormalizedReceipt {
        generation: receipt.generation().get(),
        invalidation: receipt.invalidation(),
        mutations,
    }
}

fn normalize_manifest(
    entry: ManifestEntry,
    identities: &BoundIdentities,
) -> NormalizedManifestEntry {
    match entry {
        ManifestEntry::Node(node) => {
            NormalizedManifestEntry::Node(identities.node_path(node).clone())
        }
        ManifestEntry::Fragment(fragment) => {
            NormalizedManifestEntry::Fragment(identities.fragment_path(fragment).clone())
        }
    }
}
