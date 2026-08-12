use crate::model::{Affine2V2, SpatialPointV2, SpatialScalarV2};
use crate::numeric::{exact_ratio_to_scalar, round_to_scalar};
use crate::vocabulary::{SpatialAffineComponentV2, SpatialAxisV2};

impl Affine2V2 {
    /// Returns the canonical identity transform.
    #[must_use]
    pub const fn identity() -> Self {
        Self::new(
            SpatialScalarV2::new(SpatialScalarV2::SCALE),
            SpatialScalarV2::new(0),
            SpatialScalarV2::new(0),
            SpatialScalarV2::new(SpatialScalarV2::SCALE),
            SpatialScalarV2::new(0),
            SpatialScalarV2::new(0),
        )
    }

    /// Returns a raw translation transform.
    #[must_use]
    pub const fn translation(x: SpatialScalarV2, y: SpatialScalarV2) -> Self {
        Self::new(
            SpatialScalarV2::new(SpatialScalarV2::SCALE),
            SpatialScalarV2::new(0),
            SpatialScalarV2::new(0),
            SpatialScalarV2::new(SpatialScalarV2::SCALE),
            x,
            y,
        )
    }

    /// Returns a raw axis-aligned scale transform.
    #[must_use]
    pub const fn scale(x: SpatialScalarV2, y: SpatialScalarV2) -> Self {
        Self::new(
            x,
            SpatialScalarV2::new(0),
            SpatialScalarV2::new(0),
            y,
            SpatialScalarV2::new(0),
            SpatialScalarV2::new(0),
        )
    }

    /// Returns one clockwise quarter turn in y-down scene coordinates.
    #[must_use]
    pub const fn quarter_turn_clockwise() -> Self {
        Self::new(
            SpatialScalarV2::new(0),
            SpatialScalarV2::new(SpatialScalarV2::SCALE),
            SpatialScalarV2::new(-SpatialScalarV2::SCALE),
            SpatialScalarV2::new(0),
            SpatialScalarV2::new(0),
            SpatialScalarV2::new(0),
        )
    }

    /// Composes `right` before this transform with one rounding per component.
    #[must_use = "the composed transform or its failing component must be handled"]
    pub const fn checked_compose(self, right: Self) -> Result<Self, SpatialAffineComponentV2> {
        let a = match rounded_two(self.a(), right.a(), self.c(), right.b()) {
            Some(value) => value,
            None => return Err(SpatialAffineComponentV2::A),
        };
        let b = match rounded_two(self.b(), right.a(), self.d(), right.b()) {
            Some(value) => value,
            None => return Err(SpatialAffineComponentV2::B),
        };
        let c = match rounded_two(self.a(), right.c(), self.c(), right.d()) {
            Some(value) => value,
            None => return Err(SpatialAffineComponentV2::C),
        };
        let d = match rounded_two(self.b(), right.c(), self.d(), right.d()) {
            Some(value) => value,
            None => return Err(SpatialAffineComponentV2::D),
        };
        let tx = match rounded_three(self.a(), right.tx(), self.c(), right.ty(), self.tx()) {
            Some(value) => value,
            None => return Err(SpatialAffineComponentV2::Tx),
        };
        let ty = match rounded_three(self.b(), right.tx(), self.d(), right.ty(), self.ty()) {
            Some(value) => value,
            None => return Err(SpatialAffineComponentV2::Ty),
        };
        Ok(Self::new(a, b, c, d, tx, ty))
    }

    /// Applies this transform to one point with one rounding per axis.
    #[must_use = "the transformed point or its failing axis must be handled"]
    pub const fn checked_apply_point(
        self,
        point: SpatialPointV2,
    ) -> Result<SpatialPointV2, SpatialAxisV2> {
        let x = match rounded_three(self.a(), point.x(), self.c(), point.y(), self.tx()) {
            Some(value) => value,
            None => return Err(SpatialAxisV2::X),
        };
        let y = match rounded_three(self.b(), point.x(), self.d(), point.y(), self.ty()) {
            Some(value) => value,
            None => return Err(SpatialAxisV2::Y),
        };
        Ok(SpatialPointV2::new(x, y))
    }

    /// Returns the exact widened raw determinant without applying an epsilon.
    #[must_use]
    pub const fn determinant_raw(self) -> i128 {
        (self.a().raw() as i128) * (self.d().raw() as i128)
            - (self.c().raw() as i128) * (self.b().raw() as i128)
    }

    /// Maps one scene point through the exact forward-matrix inverse formula.
    #[must_use]
    pub const fn inverse_point(self, point: SpatialPointV2) -> Option<SpatialPointV2> {
        if !affine_is_in_domain(self) || !point_is_in_domain(point) {
            return None;
        }

        let determinant = self.determinant_raw();
        if determinant == 0 {
            return None;
        }
        let dx = (point.x().raw() as i128) - (self.tx().raw() as i128);
        let dy = (point.y().raw() as i128) - (self.ty().raw() as i128);
        let x = match inverse_numerator(self.d(), dx, self.c(), dy) {
            Some(value) => value,
            None => return None,
        };
        let y = match inverse_numerator(self.a(), dy, self.b(), dx) {
            Some(value) => value,
            None => return None,
        };

        let (x, y, determinant) = if determinant < 0 {
            let x = match x.checked_neg() {
                Some(value) => value,
                None => return None,
            };
            let y = match y.checked_neg() {
                Some(value) => value,
                None => return None,
            };
            let determinant = match determinant.checked_neg() {
                Some(value) => value,
                None => return None,
            };
            (x, y, determinant)
        } else {
            (x, y, determinant)
        };

        let x = match exact_ratio_to_scalar(x, determinant) {
            Some(value) => value,
            None => return None,
        };
        let y = match exact_ratio_to_scalar(y, determinant) {
            Some(value) => value,
            None => return None,
        };
        Some(SpatialPointV2::new(x, y))
    }
}

const fn rounded_two(
    left_a: SpatialScalarV2,
    right_a: SpatialScalarV2,
    left_b: SpatialScalarV2,
    right_b: SpatialScalarV2,
) -> Option<SpatialScalarV2> {
    let first = (left_a.raw() as i128) * (right_a.raw() as i128);
    let second = (left_b.raw() as i128) * (right_b.raw() as i128);
    let numerator = match first.checked_add(second) {
        Some(value) => value,
        None => return None,
    };
    round_to_scalar(numerator, SpatialScalarV2::SCALE as i128)
}

const fn rounded_three(
    left_a: SpatialScalarV2,
    right_a: SpatialScalarV2,
    left_b: SpatialScalarV2,
    right_b: SpatialScalarV2,
    translation: SpatialScalarV2,
) -> Option<SpatialScalarV2> {
    let first = (left_a.raw() as i128) * (right_a.raw() as i128);
    let second = (left_b.raw() as i128) * (right_b.raw() as i128);
    let translation = (translation.raw() as i128) * (SpatialScalarV2::SCALE as i128);
    let numerator = match first.checked_add(second) {
        Some(value) => value,
        None => return None,
    };
    let numerator = match numerator.checked_add(translation) {
        Some(value) => value,
        None => return None,
    };
    round_to_scalar(numerator, SpatialScalarV2::SCALE as i128)
}

const fn inverse_numerator(
    positive: SpatialScalarV2,
    positive_value: i128,
    negative: SpatialScalarV2,
    negative_value: i128,
) -> Option<i128> {
    let first = match (positive.raw() as i128).checked_mul(positive_value) {
        Some(value) => value,
        None => return None,
    };
    let second = match (negative.raw() as i128).checked_mul(negative_value) {
        Some(value) => value,
        None => return None,
    };
    let difference = match first.checked_sub(second) {
        Some(value) => value,
        None => return None,
    };
    difference.checked_mul(SpatialScalarV2::SCALE as i128)
}

const fn affine_is_in_domain(affine: Affine2V2) -> bool {
    affine.a().is_in_domain()
        && affine.b().is_in_domain()
        && affine.c().is_in_domain()
        && affine.d().is_in_domain()
        && affine.tx().is_in_domain()
        && affine.ty().is_in_domain()
}

const fn point_is_in_domain(point: SpatialPointV2) -> bool {
    point.x().is_in_domain() && point.y().is_in_domain()
}
