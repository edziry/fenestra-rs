use super::types::{
    MAX_RAW_V2, MIN_RAW_V2, NumericAffineInputV2, NumericFaultKindV2, NumericInputV2,
    NumericRecordV2, NumericResultV2, NumericRunV2, SCALE_V2,
};

pub(crate) fn literal_numeric_run_v2(inputs: &[NumericInputV2]) -> NumericResultV2<NumericRunV2> {
    let records = inputs
        .iter()
        .map(record)
        .collect::<NumericResultV2<Vec<_>>>()?;
    Ok(NumericRunV2 {
        records,
        typed_space_witnesses: 4,
        proves_endpoints: in_domain(MIN_RAW_V2) && in_domain(MAX_RAW_V2),
        proves_rounding: inputs.iter().all(|input| input.ratios == [(3, 2), (-3, 2)]),
        proves_composition: !inputs.is_empty(),
        proves_inverse: inputs.len() >= 4,
        proves_transform_origin: inputs
            .iter()
            .any(|input| input.left.origin != [0, 0] || input.right.origin != [0, 0]),
    })
}

fn record(input: &NumericInputV2) -> NumericResultV2<NumericRecordV2> {
    let left = effective(input.left)?;
    let right = effective(input.right)?;
    let composition = compose(left, right)?;
    let determinant = determinant(composition);
    if determinant == 0 {
        return Err(NumericFaultKindV2::SingularInverse);
    }
    Ok(NumericRecordV2 {
        ordinal: input.ordinal,
        composition,
        determinant,
        inverse_point: inverse_point(composition, input.point)?,
        transformed_bounds: transform_bounds(composition, input.bounds)?,
        rounded_ratios: input.ratios.map(|(numerator, denominator)| {
            round_ratio(i128::from(numerator), i128::from(denominator)) as i64
        }),
    })
}

fn effective(input: NumericAffineInputV2) -> NumericResultV2<[i64; 6]> {
    let to_origin = translation(input.origin[0], input.origin[1]);
    let from_origin = translation(-input.origin[0], -input.origin[1]);
    compose(to_origin, compose(input.values, from_origin)?)
}

pub(super) fn compose(left: [i64; 6], right: [i64; 6]) -> NumericResultV2<[i64; 6]> {
    let [la, lb, lc, ld, ltx, lty] = left;
    let [ra, rb, rc, rd, rtx, rty] = right;
    Ok([
        product_sum(la, ra, lc, rb, 0)?,
        product_sum(lb, ra, ld, rb, 0)?,
        product_sum(la, rc, lc, rd, 0)?,
        product_sum(lb, rc, ld, rd, 0)?,
        product_sum(la, rtx, lc, rty, ltx)?,
        product_sum(lb, rtx, ld, rty, lty)?,
    ])
}

fn product_sum(a: i64, b: i64, c: i64, d: i64, translation: i64) -> NumericResultV2<i64> {
    let numerator = i128::from(a) * i128::from(b)
        + i128::from(c) * i128::from(d)
        + i128::from(translation) * i128::from(SCALE_V2);
    checked_raw(round_ratio(numerator, i128::from(SCALE_V2)))
}

fn inverse_point(transform: [i64; 6], point: [i64; 2]) -> NumericResultV2<[i64; 2]> {
    let [a, b, c, d, tx, ty] = transform;
    let determinant = determinant(transform);
    if determinant == 0 {
        return Err(NumericFaultKindV2::SingularInverse);
    }
    let dx = i128::from(point[0] - tx);
    let dy = i128::from(point[1] - ty);
    let x = (i128::from(d) * dx - i128::from(c) * dy) * i128::from(SCALE_V2);
    let y = (i128::from(a) * dy - i128::from(b) * dx) * i128::from(SCALE_V2);
    Ok([
        checked_raw(round_ratio(x, determinant))?,
        checked_raw(round_ratio(y, determinant))?,
    ])
}

fn transform_bounds(transform: [i64; 6], bounds: [i64; 4]) -> NumericResultV2<[i64; 4]> {
    let corners = [
        [bounds[0], bounds[1]],
        [bounds[0], bounds[3]],
        [bounds[2], bounds[1]],
        [bounds[2], bounds[3]],
    ];
    let transformed = corners.map(|point| transform_point(transform, point));
    let mut result = [i64::MAX, i64::MAX, i64::MIN, i64::MIN];
    for point in transformed {
        let point = point?;
        result[0] = result[0].min(point[0]);
        result[1] = result[1].min(point[1]);
        result[2] = result[2].max(point[0]);
        result[3] = result[3].max(point[1]);
    }
    Ok(result)
}

fn transform_point(transform: [i64; 6], point: [i64; 2]) -> NumericResultV2<[i64; 2]> {
    Ok([
        product_sum(transform[0], point[0], transform[2], point[1], transform[4])?,
        product_sum(transform[1], point[0], transform[3], point[1], transform[5])?,
    ])
}

pub(super) const fn determinant(value: [i64; 6]) -> i128 {
    value[0] as i128 * value[3] as i128 - value[2] as i128 * value[1] as i128
}

pub(super) fn round_ratio(numerator: i128, denominator: i128) -> i128 {
    let negative = (numerator < 0) != (denominator < 0);
    let numerator = numerator.unsigned_abs();
    let denominator = denominator.unsigned_abs();
    let rounded =
        numerator / denominator + u128::from((numerator % denominator) * 2 >= denominator);
    if negative {
        -(rounded as i128)
    } else {
        rounded as i128
    }
}

pub(super) fn checked_raw(value: i128) -> NumericResultV2<i64> {
    if value < i128::from(MIN_RAW_V2) {
        Err(NumericFaultKindV2::BelowMinimum)
    } else if value > i128::from(MAX_RAW_V2) {
        Err(NumericFaultKindV2::AboveMaximum)
    } else {
        Ok(value as i64)
    }
}

pub(super) const fn in_domain(value: i64) -> bool {
    value >= MIN_RAW_V2 && value <= MAX_RAW_V2
}

pub(super) fn detects(kind: NumericFaultKindV2) -> bool {
    match kind {
        NumericFaultKindV2::BelowMinimum => {
            checked_raw(i128::from(MIN_RAW_V2) - 1) == Err(NumericFaultKindV2::BelowMinimum)
        }
        NumericFaultKindV2::AboveMaximum => {
            checked_raw(i128::from(MAX_RAW_V2) + 1) == Err(NumericFaultKindV2::AboveMaximum)
        }
        NumericFaultKindV2::CompositionOverflow => compose(
            [MAX_RAW_V2, 0, 0, MAX_RAW_V2, MAX_RAW_V2, MAX_RAW_V2],
            [2 * SCALE_V2, 0, 0, 2 * SCALE_V2, MAX_RAW_V2, MAX_RAW_V2],
        )
        .is_err(),
        NumericFaultKindV2::SingularInverse => determinant([SCALE_V2, 0, SCALE_V2, 0, 0, 0]) == 0,
        NumericFaultKindV2::NonFiniteCandidate => !f64::NAN.is_finite(),
    }
}

const fn translation(x: i64, y: i64) -> [i64; 6] {
    [SCALE_V2, 0, 0, SCALE_V2, x, y]
}
