use std::sync::Arc;

use fenestra_ui_layout::prototype::LayoutEngineV1;

use super::*;
use crate::aabb::SpatialAabbV2;
use crate::limits::SpatialLimitsV2;
use crate::model::SpatialViewportV2;
use crate::output_view::SpatialOutputV2;
use crate::owned_input::SpatialOwnedInputV2;
use crate::resolve_error::SpatialResolveErrorV2;

type DynPrepare = fn(
    &dyn LayoutEngineV1,
    Arc<SpatialOwnedInputV2>,
    SpatialLimitsV2,
) -> Result<PreparedSpatialV2, SpatialResolveErrorV2>;
type Materialize = fn(PreparedSpatialV2) -> SpatialResolvedSnapshotV2;
type DynResolve = fn(
    &dyn LayoutEngineV1,
    Arc<SpatialOwnedInputV2>,
    SpatialLimitsV2,
) -> Result<SpatialResolvedSnapshotV2, SpatialResolveErrorV2>;

#[test]
fn preparation_has_the_exact_lifetime_free_unsized_engine_signature() {
    let _: DynPrepare = prepare_spatial_v2::<dyn LayoutEngineV1>;
    let _: DynResolve = resolve_spatial_v2::<dyn LayoutEngineV1>;
    let _: Materialize = materialize_reference_spatial_v2;
    assert_static::<PreparedSpatialV2>();
    assert_static::<SpatialResolvedSnapshotV2>();
    let _: for<'a> fn(&'a SpatialResolvedSnapshotV2) -> SpatialViewportV2 =
        SpatialResolvedSnapshotV2::viewport;
    let _: for<'a> fn(&'a SpatialResolvedSnapshotV2) -> SpatialOutputV2<'a> =
        SpatialResolvedSnapshotV2::output;
    let _: for<'a> fn(&'a SpatialResolvedSnapshotV2) -> &'a [SpatialAabbV2] =
        SpatialResolvedSnapshotV2::effective_clip_aabbs;
}

fn assert_static<T: 'static>() {}
