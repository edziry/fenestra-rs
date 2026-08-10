use super::super::{NormalizedManifestEntry, NormalizedMutation, NormalizedReceipt};
use super::RuntimeArtifactEncodeErrorV1;
use super::path::{fragment_path, node_path};
use super::value::{invalidation, property_value, surface};

use super::encode::ArtifactWriterV1;

pub(super) fn encode_receipt(
    writer: &mut ArtifactWriterV1,
    receipt: &NormalizedReceipt,
) -> Result<(), RuntimeArtifactEncodeErrorV1> {
    writer.push(&format!(
        "receipt|begin|generation={}|mutations={}|invalidates={}",
        receipt.generation(),
        receipt.mutations().len(),
        invalidation(receipt.invalidation())
    ))?;
    for (order, mutation) in receipt.mutations().iter().enumerate() {
        encode_mutation(writer, order, mutation)?;
    }
    writer.push("receipt|end")
}

fn encode_mutation(
    writer: &mut ArtifactWriterV1,
    order: usize,
    mutation: &NormalizedMutation,
) -> Result<(), RuntimeArtifactEncodeErrorV1> {
    match mutation {
        NormalizedMutation::PropertyChanged {
            node,
            property,
            old_value,
            new_value,
        } => writer.push(&format!(
            "mutation|i={order}|kind=property|node={}|property={}|old={}|new={}",
            node_path(node),
            property.get(),
            property_value(old_value),
            property_value(new_value),
        )),
        NormalizedMutation::KeyInserted {
            fragment,
            key,
            root,
            final_index,
            created,
        } => {
            writer.push(&format!(
                "mutation|i={order}|kind=insert|fragment={}|key={key}|root={}|final={final_index}",
                fragment_path(fragment),
                node_path(root),
            ))?;
            encode_manifest(writer, order, "created", created)
        }
        NormalizedMutation::KeyMoved {
            fragment,
            key,
            root,
            old_index,
            final_index,
        } => writer.push(&format!(
            "mutation|i={order}|kind=move|fragment={}|key={key}|root={}|old={old_index}|final={final_index}",
            fragment_path(fragment),
            node_path(root),
        )),
        NormalizedMutation::KeyRemoved {
            fragment,
            key,
            root,
            old_index,
            retired,
        } => {
            writer.push(&format!(
                "mutation|i={order}|kind=remove|fragment={}|key={key}|root={}|old={old_index}",
                fragment_path(fragment),
                node_path(root),
            ))?;
            encode_manifest(writer, order, "retired", retired)
        }
        NormalizedMutation::HeadlessSurfaceChanged {
            old_surface,
            new_surface,
        } => writer.push(&format!(
            "mutation|i={order}|kind=surface|old={}|new={}",
            surface(*old_surface),
            surface(*new_surface),
        )),
    }
}

fn encode_manifest(
    writer: &mut ArtifactWriterV1,
    mutation: usize,
    kind: &str,
    entries: &[NormalizedManifestEntry],
) -> Result<(), RuntimeArtifactEncodeErrorV1> {
    for (order, entry) in entries.iter().enumerate() {
        let location = match entry {
            NormalizedManifestEntry::Node(path) => format!("node={}", node_path(path)),
            NormalizedManifestEntry::Fragment(path) => {
                format!("fragment={}", fragment_path(path))
            }
        };
        writer.push(&format!(
            "manifest|mutation={mutation}|kind={kind}|i={order}|{location}"
        ))?;
    }
    Ok(())
}
