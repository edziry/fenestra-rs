use fenestra_ui_spatial::prototype::{
    Affine2V2, REGISTERED_REFERENCE_RASTER_LIMITS_V2, SpatialAabbV2, SpatialOutputAabbV2,
    SpatialPaintOutputReferenceV2, SpatialResolvedSnapshotV2,
};

use crate::baseline::literal_types::{PointV2, SceneInputV2};
use crate::baseline::model::EvidenceBuildErrorV2;
use crate::baseline::model_projection::{
    NormalizedAabbV2, NormalizedClipV2, NormalizedGeometryV2, NormalizedHitV2, NormalizedItemV2,
    NormalizedPaintReferenceV2, NormalizedPaintV2, NormalizedProjectionV2, NormalizedQueryV2,
    NormalizedRasterV2,
};

use super::adapter::point;

pub(super) fn projection(
    scene: &SceneInputV2,
    snapshot: &SpatialResolvedSnapshotV2,
) -> Result<NormalizedProjectionV2, EvidenceBuildErrorV2> {
    let output = snapshot.output();
    let geometry = output
        .geometry()
        .iter()
        .map(|row| NormalizedGeometryV2 {
            key: row.key().get(),
            path: path(scene, row.key().get()),
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
    let clips = output
        .clips()
        .iter()
        .zip(snapshot.effective_clip_aabbs())
        .map(|(row, effective)| NormalizedClipV2 {
            key: row.key().get(),
            owner: row.owner().get(),
            path: path(scene, row.owner().get()),
            parent: row.parent().map(|key| key.get()),
            shape: row.shape().get(),
            affine: affine(row.world_from_local()),
            determinant: row.world_determinant(),
            primitive: output_aabb(row.primitive_world_aabb()),
            effective: aabb(*effective),
        })
        .collect();
    let paints = output
        .paints()
        .iter()
        .map(|row| NormalizedPaintV2 {
            key: row.key(),
            owner: row.owner().get(),
            path: path(scene, row.owner().get()),
            affine: affine(row.world_from_local()),
            determinant: row.world_determinant(),
            aabb: output_aabb(row.world_aabb()),
            reference: match row.reference() {
                SpatialPaintOutputReferenceV2::Coverage { shape, brush } => {
                    NormalizedPaintReferenceV2::Coverage {
                        shape: shape.get(),
                        brush: brush.get(),
                    }
                }
                SpatialPaintOutputReferenceV2::Image { image } => {
                    NormalizedPaintReferenceV2::Image { image: image.get() }
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
        .map(|row| NormalizedItemV2 {
            key: row.key(),
            owner: row.owner().get(),
            path: path(scene, row.owner().get()),
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
        .map(|row| NormalizedItemV2 {
            key: row.key(),
            owner: row.owner().get(),
            path: path(scene, row.owner().get()),
            affine: affine(row.world_from_local()),
            determinant: row.world_determinant(),
            aabb: output_aabb(row.world_aabb()),
            shape: row.shape().get(),
            clip: row.clip().map(|key| key.get()),
            stack: row.stack_ordinal(),
            item: row.item_ordinal(),
        })
        .collect();
    let queries = scene
        .queries
        .iter()
        .map(|query| NormalizedQueryV2 {
            scene: *query,
            result: snapshot.hit_test(point(*query)).map(|hit| NormalizedHitV2 {
                key: hit.key(),
                owner: hit.owner().get(),
                path: path(scene, hit.owner().get()),
                item: hit.item_ordinal(),
                local: PointV2 {
                    x: hit.local_point().x().raw(),
                    y: hit.local_point().y().raw(),
                },
            }),
        })
        .collect();
    let raw_raster = snapshot
        .paint_frame()
        .rasterize_reference(REGISTERED_REFERENCE_RASTER_LIMITS_V2)
        .map_err(|_| EvidenceBuildErrorV2 {
            location: "reference-raster",
        })?;
    let raster = NormalizedRasterV2 {
        width: raw_raster.width(),
        height: raw_raster.height(),
        stride: raw_raster.stride(),
        bytes: raw_raster.bytes().to_vec(),
    };
    let mapping = if scene.nodes.iter().all(|node| node.path.is_none()) {
        Vec::new()
    } else {
        scene
            .nodes
            .iter()
            .map(|node| (node.key, node.path.clone()))
            .collect()
    };
    Ok(NormalizedProjectionV2 {
        mapping,
        geometry,
        clips,
        paints,
        hits,
        semantics,
        queries,
        raster,
    })
}

fn affine(value: Affine2V2) -> [i64; 6] {
    [
        value.a().raw(),
        value.b().raw(),
        value.c().raw(),
        value.d().raw(),
        value.tx().raw(),
        value.ty().raw(),
    ]
}

fn output_aabb(value: SpatialOutputAabbV2) -> NormalizedAabbV2 {
    NormalizedAabbV2 {
        empty: value.is_empty(),
        min_x: value.min_x().raw(),
        min_y: value.min_y().raw(),
        max_x: value.max_x().raw(),
        max_y: value.max_y().raw(),
    }
}

fn aabb(value: SpatialAabbV2) -> NormalizedAabbV2 {
    NormalizedAabbV2 {
        empty: value.is_empty(),
        min_x: value.min_x().raw(),
        min_y: value.min_y().raw(),
        max_x: value.max_x().raw(),
        max_y: value.max_y().raw(),
    }
}

fn path(scene: &SceneInputV2, key: u32) -> Option<String> {
    scene
        .nodes
        .iter()
        .find(|node| node.key == key)
        .and_then(|node| node.path.clone())
}
