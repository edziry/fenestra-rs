use fenestra_ui_runtime::prototype::{HeadlessPoint, HeadlessRect, HeadlessSurface};

const SCALE_MICROS: f64 = 1_000_000.0;
const SCALE_MICROS_U64: u64 = 1_000_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct NativePhysicalExtentV1 {
    width: u32,
    height: u32,
}

impl NativePhysicalExtentV1 {
    pub(super) const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub(super) const fn width(self) -> u32 {
        self.width
    }

    pub(super) const fn height(self) -> u32 {
        self.height
    }

    pub(super) const fn is_zero(self) -> bool {
        self.width == 0 || self.height == 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct NativePhysicalPointV1 {
    x: f64,
    y: f64,
}

impl NativePhysicalPointV1 {
    pub(super) const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub(super) const fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct NativeScaleFactorV1 {
    micros: u32,
}

impl NativeScaleFactorV1 {
    pub(super) fn try_from_f64(value: f64) -> Result<Self, NativeContractErrorKindV1> {
        if !value.is_finite() || value <= 0.0 {
            return Err(NativeContractErrorKindV1::InvalidScale);
        }
        let rounded = (value * SCALE_MICROS).round();
        if rounded < 1.0 || rounded > f64::from(8_000_000_u32) {
            return Err(NativeContractErrorKindV1::InvalidScale);
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let micros = rounded as u32;
        Ok(Self { micros })
    }

    pub(super) const fn micros(self) -> u32 {
        self.micros
    }

    pub(super) fn logical_surface(
        self,
        extent: NativePhysicalExtentV1,
    ) -> Result<HeadlessSurface, NativeContractErrorKindV1> {
        if extent.is_zero() {
            return Ok(HeadlessSurface::new(0, 0));
        }
        let width = logical_extent(extent.width, self.micros)?;
        let height = logical_extent(extent.height, self.micros)?;
        Ok(HeadlessSurface::new(width, height))
    }

    pub(super) fn logical_point(
        self,
        point: NativePhysicalPointV1,
    ) -> Result<HeadlessPoint, NativeContractErrorKindV1> {
        Ok(HeadlessPoint::new(
            logical_coordinate(point.x, self.micros)?,
            logical_coordinate(point.y, self.micros)?,
        ))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum NativeLimitKindV1 {
    Width,
    Height,
    Pixels,
    Bytes,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum NativeContractErrorKindV1 {
    InvalidScale,
    InvalidPoint,
    EnvironmentScaleChanged,
    ArithmeticExhausted,
    LimitExceeded(NativeLimitKindV1),
    InvalidRectangle(usize),
    UnsupportedAlpha(usize),
    Allocation,
    Invariant,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct NativeFrameLimitsV1 {
    width: u32,
    height: u32,
    pixels: usize,
    bytes: usize,
}

impl NativeFrameLimitsV1 {
    pub(super) const fn new(width: u32, height: u32, pixels: usize, bytes: usize) -> Self {
        Self {
            width,
            height,
            pixels,
            bytes,
        }
    }

    pub(super) const fn width(self) -> u32 {
        self.width
    }

    pub(super) const fn height(self) -> u32 {
        self.height
    }

    pub(super) const fn pixels(self) -> usize {
        self.pixels
    }

    pub(super) const fn bytes(self) -> usize {
        self.bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct NativeSceneRectangleV1 {
    rectangle: HeadlessRect,
    color: [u8; 4],
}

impl NativeSceneRectangleV1 {
    pub(super) const fn new(rectangle: HeadlessRect, color: [u8; 4]) -> Self {
        Self { rectangle, color }
    }

    pub(super) const fn rectangle(self) -> HeadlessRect {
        self.rectangle
    }

    pub(super) const fn color(self) -> [u8; 4] {
        self.color
    }
}

fn logical_extent(physical: u32, scale_micros: u32) -> Result<i32, NativeContractErrorKindV1> {
    let numerator = u64::from(physical)
        .checked_mul(SCALE_MICROS_U64)
        .ok_or(NativeContractErrorKindV1::ArithmeticExhausted)?;
    let divisor = u64::from(scale_micros);
    let rounded = numerator
        .checked_add(divisor - 1)
        .ok_or(NativeContractErrorKindV1::ArithmeticExhausted)?
        / divisor;
    i32::try_from(rounded).map_err(|_| NativeContractErrorKindV1::ArithmeticExhausted)
}

fn logical_coordinate(physical: f64, scale_micros: u32) -> Result<i32, NativeContractErrorKindV1> {
    if !physical.is_finite() {
        return Err(NativeContractErrorKindV1::InvalidPoint);
    }
    let logical = (physical * SCALE_MICROS / f64::from(scale_micros)).floor();
    if logical < f64::from(i32::MIN) || logical > f64::from(i32::MAX) {
        return Err(NativeContractErrorKindV1::InvalidPoint);
    }
    #[allow(clippy::cast_possible_truncation)]
    Ok(logical as i32)
}
