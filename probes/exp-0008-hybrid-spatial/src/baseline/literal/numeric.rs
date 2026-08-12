use crate::baseline::literal_types::{AffineV2, FIXED_ONE_V2, PointV2};

use super::types::{Aabb, Affine};

pub(super) const IDENTITY: Affine = Affine {
    values: [FIXED_ONE_V2, 0, 0, FIXED_ONE_V2, 0, 0],
};

pub(super) fn compose(left: Affine, right: Affine) -> Affine {
    let [la, lb, lc, ld, ltx, lty] = left.values;
    let [ra, rb, rc, rd, rtx, rty] = right.values;
    Affine {
        values: [
            rounded(la as i128 * ra as i128 + lc as i128 * rb as i128),
            rounded(lb as i128 * ra as i128 + ld as i128 * rb as i128),
            rounded(la as i128 * rc as i128 + lc as i128 * rd as i128),
            rounded(lb as i128 * rc as i128 + ld as i128 * rd as i128),
            rounded(
                la as i128 * rtx as i128
                    + lc as i128 * rty as i128
                    + ltx as i128 * FIXED_ONE_V2 as i128,
            ),
            rounded(
                lb as i128 * rtx as i128
                    + ld as i128 * rty as i128
                    + lty as i128 * FIXED_ONE_V2 as i128,
            ),
        ],
    }
}

pub(super) fn placed(local_x: i64, local_y: i64, input: AffineV2) -> Affine {
    let about = compose(
        translation(input.origin.x, input.origin.y),
        compose(
            Affine {
                values: input.values,
            },
            translation(-input.origin.x, -input.origin.y),
        ),
    );
    compose(translation(local_x, local_y), about)
}

pub(super) fn determinant(value: Affine) -> i128 {
    value.values[0] as i128 * value.values[3] as i128
        - value.values[2] as i128 * value.values[1] as i128
}

pub(super) fn inverse_point(value: Affine, point: PointV2) -> Option<PointV2> {
    let [a, b, c, d, tx, ty] = value.values;
    let mut det = determinant(value);
    if det == 0 {
        return None;
    }
    let dx = point.x as i128 - tx as i128;
    let dy = point.y as i128 - ty as i128;
    let mut x = (d as i128 * dx - c as i128 * dy) * FIXED_ONE_V2 as i128;
    let mut y = (a as i128 * dy - b as i128 * dx) * FIXED_ONE_V2 as i128;
    if det < 0 {
        det = -det;
        x = -x;
        y = -y;
    }
    Some(PointV2 {
        x: round_ratio(x, det) as i64,
        y: round_ratio(y, det) as i64,
    })
}

pub(super) fn transform_aabb(value: Affine, bounds: Aabb) -> Aabb {
    if bounds.empty {
        return Aabb::EMPTY;
    }
    let [a, b, c, d, tx, ty] = value.values;
    let [min_x, min_y, max_x, max_y] = bounds.edges;
    let edges = [min_x, max_x, min_y, max_y];
    Aabb::closed([
        transformed_edge([a, c, tx], edges, false),
        transformed_edge([b, d, ty], edges, false),
        transformed_edge([a, c, tx], edges, true),
        transformed_edge([b, d, ty], edges, true),
    ])
}

fn transformed_edge(axis: [i64; 3], edges: [i64; 4], maximum: bool) -> i64 {
    let [first, second, translation] = axis;
    let [min_x, max_x, min_y, max_y] = edges;
    let x = if (first >= 0) == maximum {
        max_x
    } else {
        min_x
    };
    let y = if (second >= 0) == maximum {
        max_y
    } else {
        min_y
    };
    let numerator = first as i128 * x as i128
        + second as i128 * y as i128
        + translation as i128 * FIXED_ONE_V2 as i128;
    if maximum {
        ceil_ratio(numerator, FIXED_ONE_V2 as i128) as i64
    } else {
        floor_ratio(numerator, FIXED_ONE_V2 as i128) as i64
    }
}

pub(super) fn round_ratio(numerator: i128, denominator: i128) -> i128 {
    let magnitude = numerator.unsigned_abs();
    let denominator = denominator as u128;
    let quotient = magnitude / denominator;
    let remainder = magnitude % denominator;
    let rounded = quotient + u128::from(remainder * 2 >= denominator);
    if numerator < 0 {
        -(rounded as i128)
    } else {
        rounded as i128
    }
}

fn rounded(numerator: i128) -> i64 {
    round_ratio(numerator, FIXED_ONE_V2 as i128) as i64
}

fn floor_ratio(numerator: i128, denominator: i128) -> i128 {
    let quotient = numerator / denominator;
    if numerator % denominator < 0 {
        quotient - 1
    } else {
        quotient
    }
}

fn ceil_ratio(numerator: i128, denominator: i128) -> i128 {
    let quotient = numerator / denominator;
    if numerator % denominator > 0 {
        quotient + 1
    } else {
        quotient
    }
}

fn translation(x: i64, y: i64) -> Affine {
    Affine {
        values: [FIXED_ONE_V2, 0, 0, FIXED_ONE_V2, x, y],
    }
}
