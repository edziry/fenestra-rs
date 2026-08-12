use euclid::{Point2D, Transform2D};

use super::super::types::{
    MAX_RAW_V2, MIN_RAW_V2, NumericAffineInputV2, NumericFaultKindV2, NumericInputV2,
    NumericRecordV2, NumericResultV2, NumericRunV2,
};
use super::{
    accepts_raw, determinant, f64_matrix_to_raw, f64_to_raw, finish, nonfinite_rejected,
    overflow_matrix, raw_to_f64,
};

struct Local;
struct Island;
struct Scene;
struct Device;

pub(super) fn run(inputs: &[NumericInputV2]) -> NumericResultV2<NumericRunV2> {
    let records = inputs
        .iter()
        .map(record)
        .collect::<NumericResultV2<Vec<_>>>()?;
    Ok(finish(inputs, records))
}

fn record(input: &NumericInputV2) -> NumericResultV2<NumericRecordV2> {
    let left = effective::<Island, Scene>(input.left);
    let right = effective::<Local, Island>(input.right);
    let scene = right.then(&left);
    let composition = scene.then(&Transform2D::<f64, Scene, Device>::identity());
    let raw = raw_matrix(composition)?;
    let inverse = composition
        .inverse()
        .ok_or(NumericFaultKindV2::SingularInverse)?;
    let point = inverse.transform_point(Point2D::new(
        raw_to_f64(input.point[0]),
        raw_to_f64(input.point[1]),
    ));
    let transformed_bounds = bounds(composition, input.bounds)?;
    Ok(NumericRecordV2 {
        ordinal: input.ordinal,
        composition: raw,
        determinant: determinant(raw),
        inverse_point: [f64_to_raw(point.x)?, f64_to_raw(point.y)?],
        transformed_bounds,
        rounded_ratios: input
            .ratios
            .map(|(numerator, denominator)| (numerator as f64 / denominator as f64).round() as i64),
    })
}

fn effective<Src, Dst>(input: NumericAffineInputV2) -> Transform2D<f64, Src, Dst> {
    let from_origin = Transform2D::<f64, Src, Src>::translation(
        -raw_to_f64(input.origin[0]),
        -raw_to_f64(input.origin[1]),
    );
    let raw = matrix::<Src, Dst>(input.values);
    let to_origin = Transform2D::<f64, Dst, Dst>::translation(
        raw_to_f64(input.origin[0]),
        raw_to_f64(input.origin[1]),
    );
    from_origin.then(&raw).then(&to_origin)
}

fn matrix<Src, Dst>(values: [i64; 6]) -> Transform2D<f64, Src, Dst> {
    Transform2D::new(
        raw_to_f64(values[0]),
        raw_to_f64(values[1]),
        raw_to_f64(values[2]),
        raw_to_f64(values[3]),
        raw_to_f64(values[4]),
        raw_to_f64(values[5]),
    )
}

fn raw_matrix<Src, Dst>(value: Transform2D<f64, Src, Dst>) -> NumericResultV2<[i64; 6]> {
    f64_matrix_to_raw([
        value.m11, value.m12, value.m21, value.m22, value.m31, value.m32,
    ])
}

fn bounds(
    transform: Transform2D<f64, Local, Device>,
    bounds: [i64; 4],
) -> NumericResultV2<[i64; 4]> {
    let corners = [
        [bounds[0], bounds[1]],
        [bounds[0], bounds[3]],
        [bounds[2], bounds[1]],
        [bounds[2], bounds[3]],
    ];
    let mut output = [i64::MAX, i64::MAX, i64::MIN, i64::MIN];
    for [x, y] in corners {
        let point = transform.transform_point(Point2D::new(raw_to_f64(x), raw_to_f64(y)));
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
            let left = matrix::<Island, Scene>(left);
            let right = matrix::<Local, Island>(right);
            raw_matrix(right.then(&left)).is_err()
        }
        NumericFaultKindV2::SingularInverse => matrix::<Local, Scene>([65_536, 0, 65_536, 0, 0, 0])
            .inverse()
            .is_none(),
        NumericFaultKindV2::NonFiniteCandidate => nonfinite_rejected(),
    }
}
