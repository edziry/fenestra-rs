use std::sync::Arc;

use fenestra_ui_layout::prototype::LayoutEngineV1;

use super::*;
use crate::limits::SpatialLimitsV2;
use crate::owned_input::SpatialOwnedInputV2;
use crate::resolve_error::SpatialResolveErrorV2;

type DynPrepare = fn(
    &dyn LayoutEngineV1,
    Arc<SpatialOwnedInputV2>,
    SpatialLimitsV2,
) -> Result<PreparedSpatialV2, SpatialResolveErrorV2>;

#[test]
fn preparation_has_the_exact_lifetime_free_unsized_engine_signature() {
    let _: DynPrepare = prepare_spatial_v2::<dyn LayoutEngineV1>;
    assert_static::<PreparedSpatialV2>();
}

fn assert_static<T: 'static>() {}
