use super::types::Aabb;

pub const SCALE: i64 = 65_536;
pub type Point = [i64; 2];
pub type Affine = [i64; 6];

pub const fn fixed(value: i32) -> i64 {
    value as i64 * SCALE
}

pub const fn identity() -> Affine {
    [SCALE, 0, 0, SCALE, 0, 0]
}

pub const fn translation(x: i64, y: i64) -> Affine {
    [SCALE, 0, 0, SCALE, x, y]
}

pub fn compose(left: Affine, right: Affine) -> Affine {
    [
        rounded_two(left[0], right[0], left[2], right[1]),
        rounded_two(left[1], right[0], left[3], right[1]),
        rounded_two(left[0], right[2], left[2], right[3]),
        rounded_two(left[1], right[2], left[3], right[3]),
        rounded_three(left[0], right[4], left[2], right[5], left[4]),
        rounded_three(left[1], right[4], left[3], right[5], left[5]),
    ]
}

pub fn about(local: Affine, origin: Point) -> Affine {
    compose(
        translation(origin[0], origin[1]),
        compose(local, translation(-origin[0], -origin[1])),
    )
}

pub fn determinant(value: Affine) -> i128 {
    i128::from(value[0]) * i128::from(value[3]) - i128::from(value[2]) * i128::from(value[1])
}

pub fn inverse_point(value: Affine, point: Point) -> Point {
    let determinant = determinant(value);
    assert_ne!(determinant, 0);
    let dx = i128::from(point[0]) - i128::from(value[4]);
    let dy = i128::from(point[1]) - i128::from(value[5]);
    let x = (i128::from(value[3]) * dx - i128::from(value[2]) * dy) * i128::from(SCALE);
    let y = (i128::from(value[0]) * dy - i128::from(value[1]) * dx) * i128::from(SCALE);
    if determinant < 0 {
        [
            i64::try_from(round_ratio(-x, -determinant)).expect("inverse x should fit"),
            i64::try_from(round_ratio(-y, -determinant)).expect("inverse y should fit"),
        ]
    } else {
        [
            i64::try_from(round_ratio(x, determinant)).expect("inverse x should fit"),
            i64::try_from(round_ratio(y, determinant)).expect("inverse y should fit"),
        ]
    }
}

pub fn project(value: Affine, bounds: Aabb) -> Aabb {
    if bounds.empty {
        return Aabb::EMPTY;
    }
    let [min_x, min_y, max_x, max_y] = bounds.edges;
    Aabb::new([
        transformed_edge(
            value[0], value[2], value[4], min_x, min_y, max_x, max_y, false,
        ),
        transformed_edge(
            value[1], value[3], value[5], min_x, min_y, max_x, max_y, false,
        ),
        transformed_edge(
            value[0], value[2], value[4], min_x, min_y, max_x, max_y, true,
        ),
        transformed_edge(
            value[1], value[3], value[5], min_x, min_y, max_x, max_y, true,
        ),
    ])
}

#[allow(clippy::too_many_arguments)]
fn transformed_edge(
    first: i64,
    second: i64,
    translation: i64,
    min_x: i64,
    min_y: i64,
    max_x: i64,
    max_y: i64,
    maximum: bool,
) -> i64 {
    let first_input = select_edge(first, min_x, max_x, maximum);
    let second_input = select_edge(second, min_y, max_y, maximum);
    let numerator = i128::from(first) * i128::from(first_input)
        + i128::from(second) * i128::from(second_input)
        + i128::from(translation) * i128::from(SCALE);
    let raw = if maximum {
        ceil_ratio(numerator, i128::from(SCALE))
    } else {
        floor_ratio(numerator, i128::from(SCALE))
    };
    i64::try_from(raw).expect("projected fixture edge should fit")
}

fn select_edge(coefficient: i64, minimum: i64, maximum: i64, upper: bool) -> i64 {
    if (coefficient >= 0) == upper {
        maximum
    } else {
        minimum
    }
}

pub fn intersect(left: Aabb, right: Aabb) -> Aabb {
    if left.empty || right.empty {
        return Aabb::EMPTY;
    }
    let edges = [
        left.edges[0].max(right.edges[0]),
        left.edges[1].max(right.edges[1]),
        left.edges[2].min(right.edges[2]),
        left.edges[3].min(right.edges[3]),
    ];
    if edges[0] > edges[2] || edges[1] > edges[3] {
        Aabb::EMPTY
    } else {
        Aabb::new(edges)
    }
}

pub fn contains(bounds: Aabb, point: Point) -> bool {
    !bounds.empty
        && point[0] >= bounds.edges[0]
        && point[1] >= bounds.edges[1]
        && point[0] <= bounds.edges[2]
        && point[1] <= bounds.edges[3]
}

pub fn round_ratio(numerator: i128, denominator: i128) -> i128 {
    assert!(denominator > 0);
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

fn rounded_two(left_a: i64, right_a: i64, left_b: i64, right_b: i64) -> i64 {
    let numerator =
        i128::from(left_a) * i128::from(right_a) + i128::from(left_b) * i128::from(right_b);
    i64::try_from(round_ratio(numerator, i128::from(SCALE))).expect("affine value should fit")
}

fn rounded_three(left_a: i64, right_a: i64, left_b: i64, right_b: i64, translation: i64) -> i64 {
    let numerator = i128::from(left_a) * i128::from(right_a)
        + i128::from(left_b) * i128::from(right_b)
        + i128::from(translation) * i128::from(SCALE);
    i64::try_from(round_ratio(numerator, i128::from(SCALE))).expect("affine value should fit")
}

impl Aabb {
    pub const EMPTY: Self = Self {
        empty: true,
        edges: [0; 4],
    };

    pub const fn new(edges: [i64; 4]) -> Self {
        Self {
            empty: false,
            edges,
        }
    }
}
