use fenestra_ui_spatial::prototype::SpatialViewportV2;

use super::types::{
    SpatialPhysicalExtentV2, SpatialPresentErrorKindV2, SpatialPresentationLimitKindV2,
    SpatialPresentationLimitsV2, SpatialRasterInputV2, StagedSpatialPixelsV2,
};

pub(crate) fn stage_reference_pixels_with_reserver_v2<F>(
    logical: SpatialViewportV2,
    physical: SpatialPhysicalExtentV2,
    raster: SpatialRasterInputV2<'_>,
    limits: SpatialPresentationLimitsV2,
    reserver: F,
) -> Result<Option<StagedSpatialPixelsV2>, SpatialPresentErrorKindV2>
where
    F: FnOnce(usize) -> Result<Vec<u32>, ()>,
{
    if physical.is_zero() {
        return Ok(None);
    }
    if logical.width() == 0 || logical.height() == 0 {
        return Err(SpatialPresentErrorKindV2::ZeroLogicalRaster);
    }
    validate_raster(logical, raster)?;
    let logical_width = logical_extent(logical.width())?;
    let logical_height = logical_extent(logical.height())?;
    let logical_pixels = pixel_count(logical_width, logical_height)?;
    if logical_pixels > limits.reference_pixels() {
        return Err(limit(SpatialPresentationLimitKindV2::ReferencePixels));
    }
    if physical.width() > limits.physical_width() {
        return Err(limit(SpatialPresentationLimitKindV2::PhysicalWidth));
    }
    if physical.height() > limits.physical_height() {
        return Err(limit(SpatialPresentationLimitKindV2::PhysicalHeight));
    }
    let physical_pixels = pixel_count(physical.width(), physical.height())?;
    if physical_pixels > limits.physical_pixels() {
        return Err(limit(SpatialPresentationLimitKindV2::PhysicalPixels));
    }
    let physical_bytes = physical_pixels
        .checked_mul(size_of::<u32>())
        .ok_or(SpatialPresentErrorKindV2::Invariant)?;
    if physical_bytes > limits.physical_bytes() {
        return Err(limit(SpatialPresentationLimitKindV2::PhysicalBytes));
    }
    let mut pixels =
        reserver(physical_pixels).map_err(|()| SpatialPresentErrorKindV2::Allocation)?;
    if pixels.capacity() < physical_pixels {
        return Err(SpatialPresentErrorKindV2::Allocation);
    }
    pixels.clear();
    for y in 0..physical.height() {
        let source_y = source_index(y, physical.height(), logical_height);
        for x in 0..physical.width() {
            let source_x = source_index(x, physical.width(), logical_width);
            let source = usize::try_from(source_y)
                .ok()
                .and_then(|row| row.checked_mul(usize::try_from(logical_width).ok()?))
                .and_then(|row| row.checked_add(usize::try_from(source_x).ok()?))
                .and_then(|index| index.checked_mul(4))
                .ok_or(SpatialPresentErrorKindV2::Invariant)?;
            let source_end = source
                .checked_add(4)
                .ok_or(SpatialPresentErrorKindV2::RasterMetadata)?;
            let bytes = raster
                .bytes()
                .get(source..source_end)
                .ok_or(SpatialPresentErrorKindV2::RasterMetadata)?;
            pixels.push(u32::from(bytes[0]) << 16 | u32::from(bytes[1]) << 8 | u32::from(bytes[2]));
        }
    }
    let digest = digest(&pixels);
    Ok(Some(StagedSpatialPixelsV2::new(physical, pixels, digest)))
}

fn validate_raster(
    logical: SpatialViewportV2,
    raster: SpatialRasterInputV2<'_>,
) -> Result<(), SpatialPresentErrorKindV2> {
    let width = logical_extent(logical.width())?;
    let height = logical_extent(logical.height())?;
    let stride = u64::from(width)
        .checked_mul(4)
        .ok_or(SpatialPresentErrorKindV2::RasterMetadata)?;
    let bytes = usize::try_from(stride)
        .ok()
        .and_then(|value| value.checked_mul(usize::try_from(height).ok()?))
        .ok_or(SpatialPresentErrorKindV2::RasterMetadata)?;
    if raster.width() != width
        || raster.height() != height
        || raster.stride() != stride
        || raster.bytes().len() != bytes
    {
        return Err(SpatialPresentErrorKindV2::RasterMetadata);
    }
    Ok(())
}

fn logical_extent(value: i32) -> Result<u32, SpatialPresentErrorKindV2> {
    u32::try_from(value).map_err(|_| SpatialPresentErrorKindV2::RasterMetadata)
}

fn pixel_count(width: u32, height: u32) -> Result<usize, SpatialPresentErrorKindV2> {
    usize::try_from(width)
        .ok()
        .and_then(|width| {
            usize::try_from(height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .ok_or(SpatialPresentErrorKindV2::Invariant)
}

fn source_index(physical: u32, physical_extent: u32, logical_extent: u32) -> u32 {
    let numerator = (u128::from(physical) * 2 + 1) * u128::from(logical_extent);
    let denominator = u128::from(physical_extent) * 2;
    u32::try_from((numerator / denominator).min(u128::from(logical_extent - 1)))
        .expect("bounded source index should fit")
}

fn digest(pixels: &[u32]) -> u64 {
    pixels
        .iter()
        .flat_map(|pixel| pixel.to_le_bytes())
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
        })
}

const fn limit(kind: SpatialPresentationLimitKindV2) -> SpatialPresentErrorKindV2 {
    SpatialPresentErrorKindV2::LimitExceeded(kind)
}
