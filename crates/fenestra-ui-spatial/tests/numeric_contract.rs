use fenestra_ui_spatial::prototype::{
    Affine2V2, SpatialAabbV2, SpatialAffineComponentV2, SpatialArithmeticOperationV2,
    SpatialAxisV2, SpatialPointV2, SpatialScalarV2, SpatialTransformErrorKindV2,
    SpatialTransformScalarFieldV2, SpatialTransformStageV2, round_ratio_v2,
};

#[allow(clippy::wrong_self_convention, dead_code)]
trait ScalarRedFallback: Sized {
    const FRACTIONAL_BITS: u32 = 0;
    const SCALE: i64 = 0;
    const MIN_RAW: i64 = 0;
    const MAX_RAW: i64 = 0;

    fn is_in_domain(self) -> bool {
        panic!("numeric RED fallback")
    }

    fn checked_from_i32(_: i32) -> Option<Self> {
        panic!("numeric RED fallback")
    }

    fn checked_add(self, _: Self) -> Option<Self> {
        panic!("numeric RED fallback")
    }

    fn checked_sub(self, _: Self) -> Option<Self> {
        panic!("numeric RED fallback")
    }

    fn checked_neg(self) -> Option<Self> {
        panic!("numeric RED fallback")
    }

    fn checked_mul(self, _: Self) -> Option<Self> {
        panic!("numeric RED fallback")
    }

    fn checked_div(self, _: Self) -> Option<Self> {
        panic!("numeric RED fallback")
    }
}

impl ScalarRedFallback for SpatialScalarV2 {}

#[allow(dead_code)]
trait AffineRedFallback: Sized {
    fn identity() -> Self {
        panic!("numeric RED fallback")
    }

    fn translation(_: SpatialScalarV2, _: SpatialScalarV2) -> Self {
        panic!("numeric RED fallback")
    }

    fn scale(_: SpatialScalarV2, _: SpatialScalarV2) -> Self {
        panic!("numeric RED fallback")
    }

    fn quarter_turn_clockwise() -> Self {
        panic!("numeric RED fallback")
    }

    fn checked_compose(self, _: Self) -> Result<Self, SpatialAffineComponentV2> {
        panic!("numeric RED fallback")
    }

    fn checked_apply_point(self, _: SpatialPointV2) -> Result<SpatialPointV2, SpatialAxisV2> {
        panic!("numeric RED fallback")
    }

    fn determinant_raw(self) -> i128 {
        panic!("numeric RED fallback")
    }

    fn inverse_point(self, _: SpatialPointV2) -> Option<SpatialPointV2> {
        panic!("numeric RED fallback")
    }

    fn checked_transform_aabb(
        self,
        _: SpatialAabbV2,
    ) -> Result<SpatialAabbV2, SpatialArithmeticOperationV2> {
        panic!("numeric RED fallback")
    }
}

impl AffineRedFallback for Affine2V2 {}

#[path = "numeric_contract/mod.rs"]
mod cases;

const SCALE: i64 = 65_536;

fn scalar(raw: i64) -> SpatialScalarV2 {
    SpatialScalarV2::new(raw)
}

fn integer(value: i64) -> SpatialScalarV2 {
    scalar(value * SCALE)
}

fn point(x: i64, y: i64) -> SpatialPointV2 {
    SpatialPointV2::new(scalar(x), scalar(y))
}

fn affine(values: [i64; 6]) -> Affine2V2 {
    Affine2V2::new(
        scalar(values[0]),
        scalar(values[1]),
        scalar(values[2]),
        scalar(values[3]),
        scalar(values[4]),
        scalar(values[5]),
    )
}

fn integer_affine(values: [i64; 6]) -> Affine2V2 {
    affine(values.map(|value| value * SCALE))
}

fn aabb(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> SpatialAabbV2 {
    SpatialAabbV2::from_edges(scalar(min_x), scalar(min_y), scalar(max_x), scalar(max_y))
        .expect("test AABB must be canonical")
}
