use fenestra_ui_spatial::prototype::{
    ReferenceRasterErrorKindV2, SpatialErrorLocationV2, SpatialLimitKindV2,
    SpatialOutputErrorKindV2, SpatialResolveErrorKindV2,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RawSpatialFaultV2 {
    pub(crate) label: &'static str,
    pub(crate) kind: SpatialResolveErrorKindV2,
    pub(crate) location: SpatialErrorLocationV2,
    pub(crate) observed: Option<u128>,
    pub(crate) maximum: Option<u128>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RawLimitBoundaryV2 {
    pub(crate) kind: SpatialLimitKindV2,
    pub(crate) equality_passes: bool,
    pub(crate) one_over_kind: SpatialResolveErrorKindV2,
    pub(crate) location: SpatialErrorLocationV2,
    pub(crate) observed: u128,
    pub(crate) maximum: u128,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RawRasterFaultV2 {
    pub(crate) kind: ReferenceRasterErrorKindV2,
    pub(crate) location: SpatialErrorLocationV2,
    pub(crate) observed: u128,
    pub(crate) maximum: u128,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RollbackEvidenceV2 {
    pub(crate) attempted_generation: u64,
    pub(crate) retained_generation: u64,
    pub(crate) before_digest: u64,
    pub(crate) after_digest: u64,
    pub(crate) before_allocation: usize,
    pub(crate) after_allocation: usize,
    pub(crate) before_state: Vec<u8>,
    pub(crate) after_state: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RawFaultEvidenceV2 {
    pub(crate) raw_inputs: Vec<RawSpatialFaultV2>,
    pub(crate) limits: Vec<RawLimitBoundaryV2>,
    pub(crate) output_faults: [SpatialOutputErrorKindV2; 10],
    pub(crate) dependency_cycle: RawSpatialFaultV2,
    pub(crate) singular: RawSpatialFaultV2,
    pub(crate) raster: RawRasterFaultV2,
    pub(crate) rollback: RollbackEvidenceV2,
    pub(crate) native_faults: usize,
    pub(crate) native_presenter_rows: usize,
    pub(crate) candidate_faults: usize,
}
