#![allow(
    dead_code,
    reason = "the private lane is invoked only by its exclusive conformance tests"
)]

mod backend;
mod driver;
mod port;
mod presenter;
mod stage;
mod types;

#[cfg(test)]
pub(crate) use backend::spatial_pre_present_notify_source_v2;
#[allow(
    unused_imports,
    reason = "the private adapter surface is exercised by its exclusive conformance tests"
)]
pub(crate) use backend::{
    SpatialPresenterBackendErrorV2, SpatialPresenterBufferPortV2, SpatialPresenterSurfacePortV2,
};
#[allow(
    unused_imports,
    reason = "the private adapter surface is exercised by its exclusive conformance tests"
)]
pub(crate) use driver::present_spatial_offer_v2;
#[allow(
    unused_imports,
    reason = "the private adapter surface is exercised by its exclusive conformance tests"
)]
pub(crate) use port::SpatialPresenterPortV2;
#[allow(
    unused_imports,
    reason = "the private adapter surface is exercised by its exclusive conformance tests"
)]
pub(crate) use presenter::SpatialReferencePresenterV2;
#[allow(
    unused_imports,
    reason = "the private adapter surface is exercised by its exclusive conformance tests"
)]
pub(crate) use stage::stage_reference_pixels_with_reserver_v2;
#[allow(
    unused_imports,
    reason = "the private adapter surface is exercised by its exclusive conformance tests"
)]
pub(crate) use types::{
    SpatialPhysicalExtentV2, SpatialPresentErrorKindV2, SpatialPresentationLimitKindV2,
    SpatialPresentationLimitsV2, SpatialPresentationOutcomeV2, SpatialRasterInputV2,
    SpatialSurfaceTupleV2,
};
