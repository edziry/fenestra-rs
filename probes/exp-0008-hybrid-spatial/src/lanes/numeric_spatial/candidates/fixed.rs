use fixed::types::I48F16 as F;

use super::super::types::{
    MAX_RAW_V2, MIN_RAW_V2, NumericAffineInputV2, NumericFaultKindV2, NumericInputV2,
    NumericRecordV2, NumericResultV2, NumericRunV2, SCALE_V2,
};
use super::{accepts_raw, finish, nonfinite_rejected, overflow_matrix};

type Matrix = [F; 6];

pub(super) fn run(inputs: &[NumericInputV2]) -> NumericResultV2<NumericRunV2> {
    let records = inputs
        .iter()
        .map(record)
        .collect::<NumericResultV2<Vec<_>>>()?;
    Ok(finish(inputs, records))
}

fn record(input: &NumericInputV2) -> NumericResultV2<NumericRecordV2> {
    let composition = multiply(effective(input.left)?, effective(input.right)?)?;
    let determinant = fixed_determinant(composition)?;
    if determinant == F::ZERO {
        return Err(NumericFaultKindV2::SingularInverse);
    }
    Ok(NumericRecordV2 {
        ordinal: input.ordinal,
        composition: raw_matrix(composition),
        determinant: i128::from(determinant.to_bits()) * i128::from(SCALE_V2),
        inverse_point: inverse_point(composition, determinant, input.point)?,
        transformed_bounds: bounds(composition, input.bounds)?,
        rounded_ratios: input.ratios.map(|(numerator, denominator)| {
            (F::from_num(numerator) / F::from_num(denominator))
                .round()
                .to_num()
        }),
    })
}

fn effective(input: NumericAffineInputV2) -> NumericResultV2<Matrix> {
    let to_origin = translation(input.origin[0], input.origin[1]);
    let from_origin = translation(-input.origin[0], -input.origin[1]);
    multiply(to_origin, multiply(matrix(input.values), from_origin)?)
}

fn multiply(left: Matrix, right: Matrix) -> NumericResultV2<Matrix> {
    let [la, lb, lc, ld, ltx, lty] = left;
    let [ra, rb, rc, rd, rtx, rty] = right;
    Ok([
        product_sum(la, ra, lc, rb, F::ZERO)?,
        product_sum(lb, ra, ld, rb, F::ZERO)?,
        product_sum(la, rc, lc, rd, F::ZERO)?,
        product_sum(lb, rc, ld, rd, F::ZERO)?,
        product_sum(la, rtx, lc, rty, ltx)?,
        product_sum(lb, rtx, ld, rty, lty)?,
    ])
}

fn product_sum(a: F, b: F, c: F, d: F, translation: F) -> NumericResultV2<F> {
    a.checked_mul(b)
        .and_then(|value| c.checked_mul(d).and_then(|other| value.checked_add(other)))
        .and_then(|value| value.checked_add(translation))
        .filter(|value| accepts_raw(value.to_bits()))
        .ok_or(NumericFaultKindV2::CompositionOverflow)
}

fn fixed_determinant(value: Matrix) -> NumericResultV2<F> {
    value[0]
        .checked_mul(value[3])
        .and_then(|left| {
            value[2]
                .checked_mul(value[1])
                .and_then(|right| left.checked_sub(right))
        })
        .ok_or(NumericFaultKindV2::CompositionOverflow)
}

fn inverse_point(transform: Matrix, determinant: F, point: [i64; 2]) -> NumericResultV2<[i64; 2]> {
    let dx = F::from_bits(point[0])
        .checked_sub(transform[4])
        .ok_or(NumericFaultKindV2::CompositionOverflow)?;
    let dy = F::from_bits(point[1])
        .checked_sub(transform[5])
        .ok_or(NumericFaultKindV2::CompositionOverflow)?;
    let x = transform[3]
        .checked_mul(dx)
        .and_then(|left| {
            transform[2]
                .checked_mul(dy)
                .and_then(|right| left.checked_sub(right))
        })
        .and_then(|value| value.checked_div(determinant))
        .ok_or(NumericFaultKindV2::CompositionOverflow)?;
    let y = transform[0]
        .checked_mul(dy)
        .and_then(|left| {
            transform[1]
                .checked_mul(dx)
                .and_then(|right| left.checked_sub(right))
        })
        .and_then(|value| value.checked_div(determinant))
        .ok_or(NumericFaultKindV2::CompositionOverflow)?;
    Ok([x.to_bits(), y.to_bits()])
}

fn bounds(transform: Matrix, bounds: [i64; 4]) -> NumericResultV2<[i64; 4]> {
    let corners = [
        [bounds[0], bounds[1]],
        [bounds[0], bounds[3]],
        [bounds[2], bounds[1]],
        [bounds[2], bounds[3]],
    ];
    let mut output = [i64::MAX, i64::MAX, i64::MIN, i64::MIN];
    for [x, y] in corners {
        let point = transform_point(transform, [F::from_bits(x), F::from_bits(y)])?;
        let point = [point[0].to_bits(), point[1].to_bits()];
        output[0] = output[0].min(point[0]);
        output[1] = output[1].min(point[1]);
        output[2] = output[2].max(point[0]);
        output[3] = output[3].max(point[1]);
    }
    Ok(output)
}

fn transform_point(transform: Matrix, point: [F; 2]) -> NumericResultV2<[F; 2]> {
    Ok([
        product_sum(transform[0], point[0], transform[2], point[1], transform[4])?,
        product_sum(transform[1], point[0], transform[3], point[1], transform[5])?,
    ])
}

fn matrix(values: [i64; 6]) -> Matrix {
    values.map(F::from_bits)
}

fn raw_matrix(values: Matrix) -> [i64; 6] {
    values.map(F::to_bits)
}

fn translation(x: i64, y: i64) -> Matrix {
    matrix([SCALE_V2, 0, 0, SCALE_V2, x, y])
}

pub(super) fn detects(kind: NumericFaultKindV2) -> bool {
    match kind {
        NumericFaultKindV2::BelowMinimum => !accepts_raw(MIN_RAW_V2 - 1),
        NumericFaultKindV2::AboveMaximum => !accepts_raw(MAX_RAW_V2 + 1),
        NumericFaultKindV2::CompositionOverflow => {
            let (left, right) = overflow_matrix();
            multiply(matrix(left), matrix(right)).is_err()
        }
        NumericFaultKindV2::SingularInverse => {
            fixed_determinant(matrix([SCALE_V2, 0, SCALE_V2, 0, 0, 0])) == Ok(F::ZERO)
        }
        NumericFaultKindV2::NonFiniteCandidate => nonfinite_rejected(),
    }
}
