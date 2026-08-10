use fenestra_ui_runtime::prototype::RuntimeGeneration;

use super::surface::NativeSurfaceTupleV1;
use super::types::{
    NativeContractErrorKindV1, NativeFrameLimitsV1, NativeLimitKindV1, NativePhysicalExtentV1,
    NativeScaleFactorV1, NativeSceneRectangleV1,
};

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const SCALE_MICROS: i64 = 1_000_000;

#[derive(Debug)]
pub(super) struct CpuFrameV1 {
    runtime_generation: RuntimeGeneration,
    surface: NativeSurfaceTupleV1,
    pixels: Vec<u32>,
    accounted_bytes: usize,
    digest: u64,
}

impl CpuFrameV1 {
    pub(super) const fn runtime_generation(&self) -> RuntimeGeneration {
        self.runtime_generation
    }

    pub(super) const fn surface_tuple(&self) -> NativeSurfaceTupleV1 {
        self.surface
    }

    pub(super) fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    pub(super) const fn accounted_bytes(&self) -> usize {
        self.accounted_bytes
    }

    pub(super) const fn digest(&self) -> u64 {
        self.digest
    }
}

pub(super) fn build_cpu_frame_v1(
    runtime_generation: RuntimeGeneration,
    surface: NativeSurfaceTupleV1,
    scene: &[NativeSceneRectangleV1],
    limits: NativeFrameLimitsV1,
) -> Result<Option<CpuFrameV1>, NativeContractErrorKindV1> {
    build_cpu_frame_with_reserver_v1(runtime_generation, surface, scene, limits, reserve_pixels)
}

pub(super) fn build_cpu_frame_with_reserver_v1<F>(
    runtime_generation: RuntimeGeneration,
    surface: NativeSurfaceTupleV1,
    scene: &[NativeSceneRectangleV1],
    limits: NativeFrameLimitsV1,
    reserver: F,
) -> Result<Option<CpuFrameV1>, NativeContractErrorKindV1>
where
    F: FnOnce(usize) -> Result<Vec<u32>, ()>,
{
    let extent = surface.physical();
    if extent.is_zero() {
        return Ok(None);
    }
    let scale = surface.scale();
    let (pixel_count, accounted_bytes) = preflight(extent, scale, scene, limits)?;
    let mut pixels = reserver(pixel_count).map_err(|()| NativeContractErrorKindV1::Allocation)?;
    if pixels.capacity() < pixel_count {
        return Err(NativeContractErrorKindV1::Allocation);
    }
    pixels.clear();
    pixels.resize(pixel_count, 0);
    for record in scene {
        draw(&mut pixels, extent, scale, *record)?;
    }
    let digest = pixels
        .iter()
        .flat_map(|pixel| pixel.to_le_bytes())
        .fold(FNV_OFFSET, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME)
        });
    Ok(Some(CpuFrameV1 {
        runtime_generation,
        surface,
        pixels,
        accounted_bytes,
        digest,
    }))
}

fn reserve_pixels(pixel_count: usize) -> Result<Vec<u32>, ()> {
    let mut pixels = Vec::new();
    pixels.try_reserve_exact(pixel_count).map_err(|_| ())?;
    Ok(pixels)
}

fn preflight(
    extent: NativePhysicalExtentV1,
    scale: NativeScaleFactorV1,
    scene: &[NativeSceneRectangleV1],
    limits: NativeFrameLimitsV1,
) -> Result<(usize, usize), NativeContractErrorKindV1> {
    if extent.width() > limits.width() {
        return Err(limit_error(NativeLimitKindV1::Width));
    }
    if extent.height() > limits.height() {
        return Err(limit_error(NativeLimitKindV1::Height));
    }
    let pixel_count = usize::try_from(extent.width())
        .ok()
        .and_then(|width| {
            usize::try_from(extent.height())
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .ok_or(NativeContractErrorKindV1::ArithmeticExhausted)?;
    if pixel_count > limits.pixels() {
        return Err(limit_error(NativeLimitKindV1::Pixels));
    }
    let accounted_bytes = pixel_count
        .checked_mul(size_of::<u32>())
        .ok_or(NativeContractErrorKindV1::ArithmeticExhausted)?;
    if accounted_bytes > limits.bytes() {
        return Err(limit_error(NativeLimitKindV1::Bytes));
    }
    for (index, record) in scene.iter().enumerate() {
        validate_rectangle(scale, *record)
            .map_err(|_| NativeContractErrorKindV1::InvalidRectangle(index))?;
        if record.color()[3] != u8::MAX {
            return Err(NativeContractErrorKindV1::UnsupportedAlpha(index));
        }
    }
    Ok((pixel_count, accounted_bytes))
}

fn validate_rectangle(
    scale: NativeScaleFactorV1,
    record: NativeSceneRectangleV1,
) -> Result<(), NativeContractErrorKindV1> {
    let rectangle = record.rectangle();
    if rectangle.width() < 0 || rectangle.height() < 0 {
        return Err(NativeContractErrorKindV1::ArithmeticExhausted);
    }
    let right = i64::from(rectangle.x())
        .checked_add(i64::from(rectangle.width()))
        .ok_or(NativeContractErrorKindV1::ArithmeticExhausted)?;
    let bottom = i64::from(rectangle.y())
        .checked_add(i64::from(rectangle.height()))
        .ok_or(NativeContractErrorKindV1::ArithmeticExhausted)?;
    let scale = i64::from(scale.micros());
    scaled_floor(i64::from(rectangle.x()), scale)?;
    scaled_floor(i64::from(rectangle.y()), scale)?;
    scaled_ceil(right, scale)?;
    scaled_ceil(bottom, scale)?;
    Ok(())
}

fn draw(
    pixels: &mut [u32],
    extent: NativePhysicalExtentV1,
    scale: NativeScaleFactorV1,
    record: NativeSceneRectangleV1,
) -> Result<(), NativeContractErrorKindV1> {
    let rectangle = record.rectangle();
    let right = i64::from(rectangle.x())
        .checked_add(i64::from(rectangle.width()))
        .ok_or(NativeContractErrorKindV1::ArithmeticExhausted)?;
    let bottom = i64::from(rectangle.y())
        .checked_add(i64::from(rectangle.height()))
        .ok_or(NativeContractErrorKindV1::ArithmeticExhausted)?;
    let scale = i64::from(scale.micros());
    let left = scaled_floor(i64::from(rectangle.x()), scale)?;
    let top = scaled_floor(i64::from(rectangle.y()), scale)?;
    let right = scaled_ceil(right, scale)?;
    let bottom = scaled_ceil(bottom, scale)?;
    let left = left.clamp(0, i64::from(extent.width()));
    let right = right.clamp(0, i64::from(extent.width()));
    let top = top.clamp(0, i64::from(extent.height()));
    let bottom = bottom.clamp(0, i64::from(extent.height()));
    if left >= right || top >= bottom {
        return Ok(());
    }
    let width = usize::try_from(extent.width())
        .map_err(|_| NativeContractErrorKindV1::ArithmeticExhausted)?;
    let color = record.color();
    let packed = u32::from(color[0]) << 16 | u32::from(color[1]) << 8 | u32::from(color[2]);
    for y in top..bottom {
        let row = usize::try_from(y)
            .map_err(|_| NativeContractErrorKindV1::ArithmeticExhausted)?
            .checked_mul(width)
            .ok_or(NativeContractErrorKindV1::ArithmeticExhausted)?;
        for x in left..right {
            let index = row
                .checked_add(
                    usize::try_from(x)
                        .map_err(|_| NativeContractErrorKindV1::ArithmeticExhausted)?,
                )
                .ok_or(NativeContractErrorKindV1::ArithmeticExhausted)?;
            *pixels
                .get_mut(index)
                .ok_or(NativeContractErrorKindV1::ArithmeticExhausted)? = packed;
        }
    }
    Ok(())
}

fn scaled_floor(edge: i64, scale: i64) -> Result<i64, NativeContractErrorKindV1> {
    let numerator = edge
        .checked_mul(scale)
        .ok_or(NativeContractErrorKindV1::ArithmeticExhausted)?;
    Ok(floor_div(numerator, SCALE_MICROS))
}

fn scaled_ceil(edge: i64, scale: i64) -> Result<i64, NativeContractErrorKindV1> {
    let numerator = edge
        .checked_mul(scale)
        .ok_or(NativeContractErrorKindV1::ArithmeticExhausted)?;
    floor_div(
        numerator
            .checked_neg()
            .ok_or(NativeContractErrorKindV1::ArithmeticExhausted)?,
        SCALE_MICROS,
    )
    .checked_neg()
    .ok_or(NativeContractErrorKindV1::ArithmeticExhausted)
}

const fn floor_div(numerator: i64, denominator: i64) -> i64 {
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    if remainder < 0 {
        quotient - 1
    } else {
        quotient
    }
}

const fn limit_error(kind: NativeLimitKindV1) -> NativeContractErrorKindV1 {
    NativeContractErrorKindV1::LimitExceeded(kind)
}
