use fenestra_ui_ir::prototype::ValidatedConstruction;
use fenestra_ui_runtime::prototype::CommittedRuntimeSnapshot;
use fenestra_ui_spatial::prototype::{ReferenceRasterLimitsV2, SpatialPointV2, SpatialScalarV2};

use super::path::BoundIdentities;
use super::types::{NormalizedHit, NormalizedHitQuery, NormalizedRaster};

const PIXEL_LIMIT: usize = 35_840;
const QUERY_LIMIT: usize = PIXEL_LIMIT + 4;

pub(super) fn normalize_queries(
    construction: &ValidatedConstruction,
    committed: &CommittedRuntimeSnapshot,
) -> Vec<NormalizedHitQuery> {
    let identities = BoundIdentities::bind(construction, committed);
    let spatial = committed
        .spatial()
        .expect("the authored runtime should publish spatial state");
    let snapshot = spatial.snapshot();
    let viewport = snapshot.viewport();
    let width = u32::try_from(viewport.width()).expect("the fixture viewport should be positive");
    let height = u32::try_from(viewport.height()).expect("the fixture viewport should be positive");
    let pixels = usize::try_from(width)
        .expect("the fixture width should fit")
        .checked_mul(usize::try_from(height).expect("the fixture height should fit"))
        .expect("the fixture pixel count should fit");
    assert!(pixels <= PIXEL_LIMIT);
    let query_count = pixels.checked_add(4).expect("query count should fit");
    assert!(query_count <= QUERY_LIMIT);
    let mut queries = Vec::with_capacity(query_count);
    for y in 0..height {
        for x in 0..width {
            let point = point(
                i64::from(x) * SpatialScalarV2::SCALE + SpatialScalarV2::SCALE / 2,
                i64::from(y) * SpatialScalarV2::SCALE + SpatialScalarV2::SCALE / 2,
            );
            queries.push(normalize_query(point, spatial, &identities));
        }
    }
    for point in [
        point(-1, 0),
        point(0, -1),
        point(i64::from(width) * SpatialScalarV2::SCALE, 0),
        point(0, i64::from(height) * SpatialScalarV2::SCALE),
    ] {
        queries.push(normalize_query(point, spatial, &identities));
    }
    assert_eq!(queries.len(), query_count);
    queries
}

pub(super) fn normalize_raster(committed: &CommittedRuntimeSnapshot) -> NormalizedRaster {
    let snapshot = committed
        .spatial()
        .expect("the authored runtime should publish spatial state")
        .snapshot();
    let raster = snapshot
        .rasterize_reference(ReferenceRasterLimitsV2::new(PIXEL_LIMIT))
        .expect("the authored snapshot should fit its exact raster bound");
    NormalizedRaster {
        width: raster.width(),
        height: raster.height(),
        stride: raster.stride(),
        bytes: raster.bytes().to_vec().into_boxed_slice(),
    }
}

fn normalize_query(
    scene: SpatialPointV2,
    spatial: fenestra_ui_runtime::prototype::RuntimeSpatialViewV2<'_>,
    identities: &BoundIdentities,
) -> NormalizedHitQuery {
    let result = spatial.snapshot().hit_test(scene).map(|hit| {
        let node = spatial
            .logical_node(hit.owner())
            .expect("a winning hit owner should map to one logical node");
        assert_eq!(spatial.spatial_key(node), Some(hit.owner()));
        let local = hit.local_point();
        NormalizedHit {
            key: hit.key(),
            owner: hit.owner().get(),
            path: identities.node_path(node).clone(),
            item: hit.item_ordinal(),
            local: [local.x().raw(), local.y().raw()],
        }
    });
    NormalizedHitQuery {
        scene: [scene.x().raw(), scene.y().raw()],
        result,
    }
}

const fn point(x: i64, y: i64) -> SpatialPointV2 {
    SpatialPointV2::new(SpatialScalarV2::new(x), SpatialScalarV2::new(y))
}
