use fenestra_ui_layout::prototype::ReferenceStackEngineV1;
use fenestra_ui_spatial::prototype::{
    REGISTERED_REFERENCE_RASTER_LIMITS_V2, REGISTERED_SPATIAL_LIMITS_V2, SpatialViewportV2,
    resolve_spatial_v2,
};

use super::input::empty_owned;
use super::types::RawRasterFaultV2;

pub(super) fn raster_limit() -> RawRasterFaultV2 {
    let viewport = SpatialViewportV2::new(2_113, 1_985);
    let snapshot = resolve_spatial_v2(
        &ReferenceStackEngineV1::new(),
        empty_owned(viewport),
        REGISTERED_SPATIAL_LIMITS_V2,
    )
    .expect("empty raster fault scene resolves");
    let result = snapshot
        .paint_frame()
        .rasterize_reference(REGISTERED_REFERENCE_RASTER_LIMITS_V2);
    let error = match result {
        Ok(_) => panic!("the real registered raster pixel limit must reject one over"),
        Err(error) => error,
    };
    RawRasterFaultV2 {
        kind: error.kind(),
        location: error.location(),
        observed: error.observed().expect("raster limit observed evidence"),
        maximum: error.maximum().expect("raster limit maximum evidence"),
    }
}
