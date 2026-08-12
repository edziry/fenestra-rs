use crate::error::SpatialErrorLocationV2;
use crate::output_field::{SpatialOutputFieldV2, SpatialOutputTableV2};
use crate::resolve_error::{
    SpatialOutputErrorKindV2, SpatialResolveErrorKindV2, SpatialResolveErrorV2,
};

pub(super) const fn count_error(table: SpatialOutputTableV2) -> SpatialResolveErrorV2 {
    SpatialResolveErrorV2::non_limit(
        SpatialResolveErrorKindV2::Output(SpatialOutputErrorKindV2::RecordCountMismatch),
        SpatialErrorLocationV2::Output { table },
    )
}

pub(super) const fn output_error(
    kind: SpatialOutputErrorKindV2,
    table: SpatialOutputTableV2,
    index: u32,
    field: SpatialOutputFieldV2,
) -> SpatialResolveErrorV2 {
    SpatialResolveErrorV2::non_limit(
        SpatialResolveErrorKindV2::Output(kind),
        SpatialErrorLocationV2::OutputRecord {
            table,
            index,
            field,
        },
    )
}

pub(super) fn ordinal(index: usize) -> u32 {
    u32::try_from(index).expect("prepared output ordinal fits u32")
}
