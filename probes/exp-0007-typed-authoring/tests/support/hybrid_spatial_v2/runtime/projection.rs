use fenestra_ui_ir::prototype::ValidatedConstruction;
use fenestra_ui_runtime::prototype::CommittedRuntimeSnapshot;
use fenestra_ui_spatial::prototype::{
    Affine2V2, REGISTERED_SPATIAL_LIMITS_V2, SpatialAabbV2, SpatialLimitKindV2,
    SpatialOutputAabbV2, SpatialPaintOutputReferenceV2,
};

use super::path::{BoundIdentities, NodePath};
use super::types::{
    NormalizedAabb, NormalizedAffine, NormalizedClip, NormalizedGeometry, NormalizedItem,
    NormalizedPaint, NormalizedPaintReference, NormalizedProjection,
};

pub(super) fn normalize_projection(
    construction: &ValidatedConstruction,
    committed: &CommittedRuntimeSnapshot,
) -> NormalizedProjection {
    let identities = BoundIdentities::bind(construction, committed);
    let spatial = committed
        .spatial()
        .expect("the authored runtime should publish spatial state");
    let snapshot = spatial.snapshot();
    let output = snapshot.output();
    assert_bound(output.geometry().len(), SpatialLimitKindV2::Nodes);
    assert_bound(output.clips().len(), SpatialLimitKindV2::Clips);
    assert_bound(output.paints().len(), SpatialLimitKindV2::PaintItems);
    assert_bound(output.hits().len(), SpatialLimitKindV2::HitItems);
    assert_bound(output.semantics().len(), SpatialLimitKindV2::SemanticItems);

    let mut seen_paths = Vec::with_capacity(output.geometry().len().saturating_sub(1));
    let mapping = output
        .geometry()
        .iter()
        .map(|row| {
            let key = row.key();
            let path = if key.get() == 0 {
                assert!(spatial.logical_node(key).is_none());
                None
            } else {
                let node = spatial
                    .logical_node(key)
                    .expect("every non-sentinel geometry row should map");
                assert_eq!(spatial.spatial_key(node), Some(key));
                let path = identities.node_path(node).clone();
                assert!(!seen_paths.contains(&path));
                seen_paths.push(path.clone());
                Some(path)
            };
            (key.get(), path)
        })
        .collect::<Vec<_>>();
    assert_eq!(seen_paths.len(), committed.node_count());

    let geometry = output
        .geometry()
        .iter()
        .map(|row| NormalizedGeometry {
            key: row.key().get(),
            path: mapping
                .get(usize::try_from(row.key().get()).expect("geometry key should fit"))
                .expect("geometry keys should be dense")
                .1
                .clone(),
            base: [
                row.base_x().raw(),
                row.base_y().raw(),
                row.base_width().raw(),
                row.base_height().raw(),
            ],
            affine: affine(row.world_from_local()),
            determinant: row.world_determinant(),
            aabb: output_aabb(row.world_aabb()),
        })
        .collect();
    let effective = snapshot.effective_clip_aabbs();
    assert_eq!(effective.len(), output.clips().len());
    let clips = output
        .clips()
        .iter()
        .enumerate()
        .map(|(index, row)| NormalizedClip {
            key: row.key().get(),
            owner: row.owner().get(),
            path: owner_path(spatial, &identities, row.owner()),
            parent: row.parent().map(|key| key.get()),
            shape: row.shape().get(),
            affine: affine(row.world_from_local()),
            determinant: row.world_determinant(),
            primitive: output_aabb(row.primitive_world_aabb()),
            effective: aabb(effective[index]),
        })
        .collect();
    let paints = output
        .paints()
        .iter()
        .map(|row| NormalizedPaint {
            key: row.key(),
            owner: row.owner().get(),
            path: owner_path(spatial, &identities, row.owner()),
            affine: affine(row.world_from_local()),
            determinant: row.world_determinant(),
            aabb: output_aabb(row.world_aabb()),
            reference: match row.reference() {
                SpatialPaintOutputReferenceV2::Coverage { shape, brush } => {
                    NormalizedPaintReference::Coverage {
                        shape: shape.get(),
                        brush: brush.get(),
                    }
                }
                SpatialPaintOutputReferenceV2::Image { image } => {
                    NormalizedPaintReference::Image { image: image.get() }
                }
            },
            clip: row.clip().map(|key| key.get()),
            stack: row.stack_ordinal(),
            item: row.item_ordinal(),
        })
        .collect();
    let hits = output
        .hits()
        .iter()
        .map(|row| NormalizedItem {
            key: row.key(),
            owner: row.owner().get(),
            path: owner_path(spatial, &identities, row.owner()),
            affine: affine(row.world_from_local()),
            determinant: row.world_determinant(),
            aabb: output_aabb(row.world_aabb()),
            shape: row.shape().get(),
            clip: row.clip().map(|key| key.get()),
            stack: row.stack_ordinal(),
            item: row.item_ordinal(),
        })
        .collect();
    let semantics = output
        .semantics()
        .iter()
        .map(|row| NormalizedItem {
            key: row.key(),
            owner: row.owner().get(),
            path: owner_path(spatial, &identities, row.owner()),
            affine: affine(row.world_from_local()),
            determinant: row.world_determinant(),
            aabb: output_aabb(row.world_aabb()),
            shape: row.shape().get(),
            clip: row.clip().map(|key| key.get()),
            stack: row.stack_ordinal(),
            item: row.item_ordinal(),
        })
        .collect();
    NormalizedProjection {
        mapping,
        geometry,
        clips,
        paints,
        hits,
        semantics,
    }
}

fn owner_path(
    spatial: fenestra_ui_runtime::prototype::RuntimeSpatialViewV2<'_>,
    identities: &BoundIdentities,
    owner: fenestra_ui_spatial::prototype::SpatialNodeKeyV2,
) -> NodePath {
    let node = spatial
        .logical_node(owner)
        .expect("every item owner should map to a logical node");
    assert_eq!(spatial.spatial_key(node), Some(owner));
    identities.node_path(node).clone()
}

fn affine(value: Affine2V2) -> NormalizedAffine {
    NormalizedAffine([
        value.a().raw(),
        value.b().raw(),
        value.c().raw(),
        value.d().raw(),
        value.tx().raw(),
        value.ty().raw(),
    ])
}

fn output_aabb(value: SpatialOutputAabbV2) -> NormalizedAabb {
    NormalizedAabb {
        empty: value.is_empty(),
        edges: [
            value.min_x().raw(),
            value.min_y().raw(),
            value.max_x().raw(),
            value.max_y().raw(),
        ],
    }
}

fn aabb(value: SpatialAabbV2) -> NormalizedAabb {
    NormalizedAabb {
        empty: value.is_empty(),
        edges: [
            value.min_x().raw(),
            value.min_y().raw(),
            value.max_x().raw(),
            value.max_y().raw(),
        ],
    }
}

fn assert_bound(observed: usize, kind: SpatialLimitKindV2) {
    assert!(observed <= REGISTERED_SPATIAL_LIMITS_V2.limit(kind));
}
