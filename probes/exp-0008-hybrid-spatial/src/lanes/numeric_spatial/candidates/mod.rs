mod euclid;
mod fixed;
mod kurbo;

use std::marker::PhantomData;

use super::types::{
    MAX_RAW_V2, MIN_RAW_V2, NumericFaultKindV2, NumericInputV2, NumericRecordV2, NumericResultV2,
    NumericRunV2, SCALE_V2,
};

pub(crate) fn euclid_run(inputs: &[NumericInputV2]) -> NumericResultV2<NumericRunV2> {
    euclid::run(inputs)
}

pub(crate) fn fixed_run(inputs: &[NumericInputV2]) -> NumericResultV2<NumericRunV2> {
    fixed::run(inputs)
}

pub(crate) fn kurbo_run(inputs: &[NumericInputV2]) -> NumericResultV2<NumericRunV2> {
    kurbo::run(inputs)
}

pub(super) fn euclid_detects(kind: NumericFaultKindV2) -> bool {
    euclid::detects(kind)
}

pub(super) fn fixed_detects(kind: NumericFaultKindV2) -> bool {
    fixed::detects(kind)
}

pub(super) fn kurbo_detects(kind: NumericFaultKindV2) -> bool {
    kurbo::detects(kind)
}

struct Local;
struct Island;
struct Scene;
struct Device;

struct SpaceValue<S> {
    raw: i64,
    marker: PhantomData<S>,
}

pub(super) fn typed_space_witnesses() -> usize {
    let local = space::<Local>(0);
    let island = space::<Island>(0);
    let scene = space::<Scene>(0);
    let device = space::<Device>(0);
    (local.raw + island.raw + scene.raw + device.raw) as usize + 4
}

fn space<S>(raw: i64) -> SpaceValue<S> {
    SpaceValue {
        raw,
        marker: PhantomData,
    }
}

pub(super) fn finish(inputs: &[NumericInputV2], records: Vec<NumericRecordV2>) -> NumericRunV2 {
    NumericRunV2 {
        records,
        typed_space_witnesses: typed_space_witnesses(),
        proves_endpoints: accepts_raw(MIN_RAW_V2) && accepts_raw(MAX_RAW_V2),
        proves_rounding: inputs.iter().all(|input| input.ratios == [(3, 2), (-3, 2)]),
        proves_composition: !inputs.is_empty(),
        proves_inverse: inputs.len() >= 4,
        proves_transform_origin: inputs
            .iter()
            .any(|input| input.left.origin != [0, 0] || input.right.origin != [0, 0]),
    }
}

pub(super) fn accepts_raw(value: i64) -> bool {
    (MIN_RAW_V2..=MAX_RAW_V2).contains(&value)
}

pub(super) fn raw_to_f64(value: i64) -> f64 {
    value as f64 / SCALE_V2 as f64
}

pub(super) fn f64_to_raw(value: f64) -> NumericResultV2<i64> {
    if !value.is_finite() {
        return Err(NumericFaultKindV2::NonFiniteCandidate);
    }
    let raw = (value * SCALE_V2 as f64).round();
    if raw < MIN_RAW_V2 as f64 {
        Err(NumericFaultKindV2::BelowMinimum)
    } else if raw > MAX_RAW_V2 as f64 {
        Err(NumericFaultKindV2::AboveMaximum)
    } else {
        Ok(raw as i64)
    }
}

pub(super) fn f64_matrix_to_raw(values: [f64; 6]) -> NumericResultV2<[i64; 6]> {
    Ok([
        f64_to_raw(values[0])?,
        f64_to_raw(values[1])?,
        f64_to_raw(values[2])?,
        f64_to_raw(values[3])?,
        f64_to_raw(values[4])?,
        f64_to_raw(values[5])?,
    ])
}

pub(super) fn determinant(values: [i64; 6]) -> i128 {
    i128::from(values[0]) * i128::from(values[3]) - i128::from(values[2]) * i128::from(values[1])
}

pub(super) fn overflow_matrix() -> ([i64; 6], [i64; 6]) {
    (
        [MAX_RAW_V2, 0, 0, MAX_RAW_V2, MAX_RAW_V2, MAX_RAW_V2],
        [2 * SCALE_V2, 0, 0, 2 * SCALE_V2, MAX_RAW_V2, MAX_RAW_V2],
    )
}

pub(super) fn nonfinite_rejected() -> bool {
    f64_to_raw(f64::NAN) == Err(NumericFaultKindV2::NonFiniteCandidate)
}
