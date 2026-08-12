use crate::model::SpatialScalarV2;

/// Rounds one signed ratio to nearest with exact halves away from zero.
#[must_use]
pub const fn round_ratio_v2(numerator: i128, positive_denominator: i128) -> Option<i128> {
    if positive_denominator <= 0 {
        return None;
    }

    let magnitude = numerator.unsigned_abs();
    let denominator = positive_denominator as u128;
    let quotient = magnitude / denominator;
    let remainder = magnitude % denominator;
    let rounded = quotient + ((remainder * 2 >= denominator) as u128);

    if numerator < 0 {
        if rounded == 1_u128 << 127 {
            Some(i128::MIN)
        } else {
            Some(-(rounded as i128))
        }
    } else {
        Some(rounded as i128)
    }
}

impl SpatialScalarV2 {
    /// Number of fractional bits in the version-2 scalar format.
    pub const FRACTIONAL_BITS: u32 = 16;
    /// Raw units in one logical unit.
    pub const SCALE: i64 = 65_536;
    /// Inclusive minimum raw value in the canonical scalar domain.
    pub const MIN_RAW: i64 = -140_737_488_289_792;
    /// Inclusive maximum raw value in the canonical scalar domain.
    pub const MAX_RAW: i64 = 140_737_488_289_792;

    /// Reports whether this raw value belongs to the canonical scalar domain.
    #[must_use]
    pub const fn is_in_domain(self) -> bool {
        self.raw() >= Self::MIN_RAW && self.raw() <= Self::MAX_RAW
    }

    /// Converts one integer exactly when it belongs to the scalar domain.
    #[must_use]
    pub const fn checked_from_i32(value: i32) -> Option<Self> {
        scalar_from_i128((value as i128) * (Self::SCALE as i128))
    }

    /// Adds two canonical scalars without saturation or wrapping.
    #[must_use]
    pub const fn checked_add(self, rhs: Self) -> Option<Self> {
        if !self.is_in_domain() || !rhs.is_in_domain() {
            return None;
        }
        scalar_from_i128((self.raw() as i128) + (rhs.raw() as i128))
    }

    /// Subtracts two canonical scalars without saturation or wrapping.
    #[must_use]
    pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
        if !self.is_in_domain() || !rhs.is_in_domain() {
            return None;
        }
        scalar_from_i128((self.raw() as i128) - (rhs.raw() as i128))
    }

    /// Negates one canonical scalar without saturation or wrapping.
    #[must_use]
    pub const fn checked_neg(self) -> Option<Self> {
        if !self.is_in_domain() {
            return None;
        }
        scalar_from_i128(-(self.raw() as i128))
    }

    /// Multiplies two canonical fixed-point scalars with one rounding step.
    #[must_use]
    pub const fn checked_mul(self, rhs: Self) -> Option<Self> {
        if !self.is_in_domain() || !rhs.is_in_domain() {
            return None;
        }
        let numerator = (self.raw() as i128) * (rhs.raw() as i128);
        round_to_scalar(numerator, Self::SCALE as i128)
    }

    /// Divides two canonical fixed-point scalars with one rounding step.
    #[must_use]
    pub const fn checked_div(self, rhs: Self) -> Option<Self> {
        if !self.is_in_domain() || !rhs.is_in_domain() || rhs.raw() == 0 {
            return None;
        }

        let mut numerator = (self.raw() as i128) * (Self::SCALE as i128);
        let mut denominator = rhs.raw() as i128;
        if denominator < 0 {
            numerator = -numerator;
            denominator = -denominator;
        }
        round_to_scalar(numerator, denominator)
    }
}

pub(crate) const fn scalar_from_i128(raw: i128) -> Option<SpatialScalarV2> {
    if raw < SpatialScalarV2::MIN_RAW as i128 || raw > SpatialScalarV2::MAX_RAW as i128 {
        None
    } else {
        Some(SpatialScalarV2::new(raw as i64))
    }
}

pub(crate) const fn round_to_scalar(
    numerator: i128,
    positive_denominator: i128,
) -> Option<SpatialScalarV2> {
    match round_ratio_v2(numerator, positive_denominator) {
        Some(raw) => scalar_from_i128(raw),
        None => None,
    }
}

pub(crate) const fn exact_ratio_to_scalar(
    numerator: i128,
    positive_denominator: i128,
) -> Option<SpatialScalarV2> {
    if positive_denominator <= 0 {
        return None;
    }

    let magnitude = numerator.unsigned_abs();
    let denominator = positive_denominator as u128;
    let quotient = magnitude / denominator;
    let remainder = magnitude % denominator;
    let maximum = SpatialScalarV2::MAX_RAW as u128;
    if quotient > maximum || (quotient == maximum && remainder != 0) {
        return None;
    }
    round_to_scalar(numerator, positive_denominator)
}

pub(crate) const fn floor_ratio(numerator: i128, positive_denominator: i128) -> Option<i128> {
    if positive_denominator <= 0 {
        return None;
    }
    let quotient = numerator / positive_denominator;
    let remainder = numerator % positive_denominator;
    if remainder < 0 {
        quotient.checked_sub(1)
    } else {
        Some(quotient)
    }
}

pub(crate) const fn ceil_ratio(numerator: i128, positive_denominator: i128) -> Option<i128> {
    if positive_denominator <= 0 {
        return None;
    }
    let quotient = numerator / positive_denominator;
    let remainder = numerator % positive_denominator;
    if remainder > 0 {
        quotient.checked_add(1)
    } else {
        Some(quotient)
    }
}
