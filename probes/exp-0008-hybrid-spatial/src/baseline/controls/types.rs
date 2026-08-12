use super::super::EvidenceSectionV2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ControlFamilyV2 {
    Metadata,
    Records,
    Fields,
    Queries,
    Raster,
    Faults,
    Codec,
}

impl ControlFamilyV2 {
    pub(crate) const fn token(self) -> &'static str {
        match self {
            Self::Metadata => "metadata",
            Self::Records => "records",
            Self::Fields => "fields",
            Self::Queries => "queries",
            Self::Raster => "raster",
            Self::Faults => "faults",
            Self::Codec => "codec",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ControlCoverageV2 {
    pub(crate) tags: bool,
    pub(crate) scalars: bool,
    pub(crate) keys: bool,
    pub(crate) options: bool,
    pub(crate) counts: bool,
    pub(crate) bytes: bool,
    pub(crate) metadata: bool,
    pub(crate) adjacent_swaps: bool,
    pub(crate) removals: bool,
    pub(crate) duplicates: bool,
    pub(crate) hit_to_miss: bool,
    pub(crate) miss_to_hit: bool,
    pub(crate) hit_fields: bool,
    pub(crate) local_coordinates: bool,
    pub(crate) raster_dimensions: bool,
    pub(crate) raster_stride: bool,
    pub(crate) raster_length: bool,
    pub(crate) raster_first_middle_last: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ControlReportV2 {
    pub(crate) family: ControlFamilyV2,
    pub(crate) registered: u64,
    pub(crate) detected: u64,
    pub(crate) exact_first_location: bool,
    pub(crate) coverage: ControlCoverageV2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvidenceMutationV2 {
    ReceiptGeneration,
    GeometryAndRaster,
    QueryHitToMiss,
    RasterFirstByte,
    SwapAdjacent(EvidenceSectionV2),
    RemoveRow(EvidenceSectionV2),
    DuplicateRow(EvidenceSectionV2),
    FieldAt {
        case: u8,
        step: u8,
        section: EvidenceSectionV2,
        record: u32,
        field: u32,
        byte: u32,
    },
    QueryMissToHit,
    QueryKey,
    QueryOwner,
    QueryOrdinal,
    QueryLocalX,
    QueryLocalY,
    RasterWidth,
    RasterHeight,
    RasterStride,
    RasterByteLength,
    WidthScalar,
    WidthDeterminant,
    WidthStride,
    WidthDimension,
    WidthKey,
    WidthColor,
    CaseOrdinal,
    CaseName,
    ObservationCase,
    ObservationStep,
    ObservationGeneration,
    ObservationViewportWidth,
    ObservationViewportHeight,
    LiteralMatch,
    ReferenceMatch,
    RepeatMatch,
    SectionRecordCount,
    SectionByteCount,
    SectionDigest,
    SectionTag,
    SectionEncoded,
}
