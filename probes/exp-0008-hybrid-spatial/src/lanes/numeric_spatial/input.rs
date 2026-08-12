use super::types::{NumericAffineInputV2 as A, NumericInputV2, SCALE_V2 as S};

const ID: A = A {
    values: [S, 0, 0, S, 0, 0],
    origin: [0, 0],
};

pub(crate) fn numeric_inputs_v2() -> Vec<NumericInputV2> {
    vec![
        input(0, ID, ID, [S / 2, -S / 2], [-S, -S, S, S]),
        input(
            1,
            affine([S, 0, 0, S, S / 2, -S / 2], [0, 0]),
            affine([S, 0, 0, S, 2 * S, S], [0, 0]),
            [3 * S, S / 2],
            [-S / 2, -S, 2 * S, S / 2],
        ),
        input(
            2,
            affine([2 * S, 0, 0, S / 2, 0, 0], [0, 0]),
            affine([0, S, -S, 0, 0, 0], [0, 0]),
            [S, 2 * S],
            [-S, -S, S, 2 * S],
        ),
        input(
            3,
            affine([S, S / 2, 0, S, 0, 0], [0, 0]),
            affine([S, 0, S / 2, S, S / 4, -S / 4], [0, 0]),
            [2 * S, -S],
            [-2 * S, -S, S, 3 * S],
        ),
        input(
            4,
            affine([-S, 0, 0, S, 0, 0], [0, 0]),
            affine([S, 0, 0, -S, S, S], [0, 0]),
            [-2 * S, 3 * S],
            [-S, -2 * S, 2 * S, S],
        ),
        input(
            5,
            affine([0, S, -S, 0, 0, 0], [S / 2, S / 2]),
            affine([S, 0, 0, S, S / 2, 0], [-S / 2, S / 4]),
            [S, 0],
            [0, 0, 2 * S, 3 * S],
        ),
        input(
            6,
            affine([3 * S / 2, 0, 0, S / 2, 0, 0], [S / 4, -S / 4]),
            affine([S / 2, 0, 0, 2 * S, S / 2, -S], [0, 0]),
            [2 * S, 2 * S],
            [-S, -S / 2, 2 * S, 2 * S],
        ),
        input(
            7,
            affine([S, 0, 0, S, 1_i64 << 45, -(1_i64 << 44)], [0, 0]),
            ID,
            [1_i64 << 40, -(1_i64 << 39)],
            [-(1_i64 << 38), -(1_i64 << 37), 1_i64 << 38, 1_i64 << 37],
        ),
    ]
}

const fn affine(values: [i64; 6], origin: [i64; 2]) -> A {
    A { values, origin }
}

const fn input(
    ordinal: u8,
    left: A,
    right: A,
    point: [i64; 2],
    bounds: [i64; 4],
) -> NumericInputV2 {
    NumericInputV2 {
        ordinal,
        left,
        right,
        point,
        bounds,
        ratios: [(3, 2), (-3, 2)],
    }
}
