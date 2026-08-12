use kurbo::{Affine, Point};

use super::super::types::{
    MAX_RAW_V2, MIN_RAW_V2, NumericAffineInputV2, NumericFaultKindV2, NumericInputV2,
    NumericRecordV2, NumericResultV2, NumericRunV2,
};
use super::{
    accepts_raw, determinant, f64_matrix_to_raw, f64_to_raw, finish, nonfinite_rejected,
    overflow_matrix, raw_to_f64,
};

pub(super) fn run(inputs: &[NumericInputV2]) -> NumericResultV2<NumericRunV2> {
    let records = inputs
        .iter()
        .map(record)
        .collect::<NumericResultV2<Vec<_>>>()?;
    Ok(finish(inputs, records))
}

fn record(input: &NumericInputV2) -> NumericResultV2<NumericRecordV2> {
    let composition = effective(input.left) * effective(input.right);
    let raw = f64_matrix_to_raw(composition.as_coeffs())?;
    if composition.determinant() == 0.0 {
        return Err(NumericFaultKindV2::SingularInverse);
    }
    let inverse = composition.inverse();
    let point = inverse * Point::new(raw_to_f64(input.point[0]), raw_to_f64(input.point[1]));
    Ok(NumericRecordV2 {
        ordinal: input.ordinal,
        composition: raw,
        determinant: determinant(raw),
        inverse_point: [f64_to_raw(point.x)?, f64_to_raw(point.y)?],
        transformed_bounds: bounds(composition, input.bounds)?,
        rounded_ratios: input
            .ratios
            .map(|(numerator, denominator)| (numerator as f64 / denominator as f64).round() as i64),
    })
}

fn effective(input: NumericAffineInputV2) -> Affine {
    Affine::translate((raw_to_f64(input.origin[0]), raw_to_f64(input.origin[1])))
        * matrix(input.values)
        * Affine::translate((-raw_to_f64(input.origin[0]), -raw_to_f64(input.origin[1])))
}

fn matrix(values: [i64; 6]) -> Affine {
    Affine::new(values.map(raw_to_f64))
}

fn bounds(transform: Affine, bounds: [i64; 4]) -> NumericResultV2<[i64; 4]> {
    let corners = [
        [bounds[0], bounds[1]],
        [bounds[0], bounds[3]],
        [bounds[2], bounds[1]],
        [bounds[2], bounds[3]],
    ];
    let mut output = [i64::MAX, i64::MAX, i64::MIN, i64::MIN];
    for [x, y] in corners {
        let point = transform * Point::new(raw_to_f64(x), raw_to_f64(y));
        let point = [f64_to_raw(point.x)?, f64_to_raw(point.y)?];
        output[0] = output[0].min(point[0]);
        output[1] = output[1].min(point[1]);
        output[2] = output[2].max(point[0]);
        output[3] = output[3].max(point[1]);
    }
    Ok(output)
}

pub(super) fn detects(kind: NumericFaultKindV2) -> bool {
    match kind {
        NumericFaultKindV2::BelowMinimum => !accepts_raw(MIN_RAW_V2 - 1),
        NumericFaultKindV2::AboveMaximum => !accepts_raw(MAX_RAW_V2 + 1),
        NumericFaultKindV2::CompositionOverflow => {
            let (left, right) = overflow_matrix();
            f64_matrix_to_raw((matrix(left) * matrix(right)).as_coeffs()).is_err()
        }
        NumericFaultKindV2::SingularInverse => {
            matrix([65_536, 0, 65_536, 0, 0, 0]).determinant() == 0.0
        }
        NumericFaultKindV2::NonFiniteCandidate => nonfinite_rejected(),
    }
}
