pub(crate) const SCALE_V2: i64 = 65_536;
pub(crate) const MIN_RAW_V2: i64 = -140_737_488_289_792;
pub(crate) const MAX_RAW_V2: i64 = 140_737_488_289_792;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NumericCandidateV2 {
    Euclid,
    Kurbo,
    Fixed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NumericOutcomeV2 {
    Pass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NumericCandidateRegistrationV2 {
    pub(crate) kind: NumericCandidateV2,
    pub(crate) name: &'static str,
    pub(crate) version: &'static str,
    pub(crate) features: &'static str,
    pub(crate) outcome: NumericOutcomeV2,
    pub(crate) reason: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NumericAffineInputV2 {
    pub(crate) values: [i64; 6],
    pub(crate) origin: [i64; 2],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NumericInputV2 {
    pub(crate) ordinal: u8,
    pub(crate) left: NumericAffineInputV2,
    pub(crate) right: NumericAffineInputV2,
    pub(crate) point: [i64; 2],
    pub(crate) bounds: [i64; 4],
    pub(crate) ratios: [(i64, i64); 2],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NumericRecordV2 {
    pub(crate) ordinal: u8,
    pub(crate) composition: [i64; 6],
    pub(crate) determinant: i128,
    pub(crate) inverse_point: [i64; 2],
    pub(crate) transformed_bounds: [i64; 4],
    pub(crate) rounded_ratios: [i64; 2],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NumericRunV2 {
    pub(crate) records: Vec<NumericRecordV2>,
    pub(crate) typed_space_witnesses: usize,
    pub(crate) proves_endpoints: bool,
    pub(crate) proves_rounding: bool,
    pub(crate) proves_composition: bool,
    pub(crate) proves_inverse: bool,
    pub(crate) proves_transform_origin: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NumericFaultKindV2 {
    BelowMinimum,
    AboveMaximum,
    CompositionOverflow,
    SingularInverse,
    NonFiniteCandidate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NumericFaultV2 {
    pub(crate) kind: NumericFaultKindV2,
    pub(crate) detected_by_literal: bool,
    pub(crate) detected_by_euclid: bool,
    pub(crate) detected_by_kurbo: bool,
    pub(crate) detected_by_fixed: bool,
}

pub(crate) type NumericResultV2<T> = Result<T, NumericFaultKindV2>;
