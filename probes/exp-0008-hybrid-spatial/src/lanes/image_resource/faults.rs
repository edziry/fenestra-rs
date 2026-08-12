use super::candidates::{image_detects, png_detects};
use super::types::{ImageCaseV2, ImageFaultKindV2, ImageFaultV2, ImageResultV2};

const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
const DIMENSION_LIMIT: u32 = 64;
const ENCODED_BYTE_LIMIT: usize = 4_096;

pub(crate) fn image_faults_v2() -> Vec<ImageFaultV2> {
    [
        ImageFaultKindV2::MalformedSignature,
        ImageFaultKindV2::DimensionLimit,
        ImageFaultKindV2::StrideOverflow,
        ImageFaultKindV2::ByteBomb,
        ImageFaultKindV2::UnsupportedColor,
        ImageFaultKindV2::TruncatedData,
    ]
    .into_iter()
    .map(|kind| ImageFaultV2 {
        kind,
        literal: detects(kind),
        png: png_detects(kind),
        image: image_detects(kind),
    })
    .collect()
}

pub(super) fn preflight_cases(cases: &[ImageCaseV2]) -> ImageResultV2<()> {
    for case in cases {
        preflight_bytes(&case.png_bytes)?;
    }
    Ok(())
}

pub(super) fn detects(kind: ImageFaultKindV2) -> bool {
    match kind {
        ImageFaultKindV2::MalformedSignature => preflight_bytes(b"not-png").is_err(),
        ImageFaultKindV2::DimensionLimit => validate_dimensions(65, 1).is_err(),
        ImageFaultKindV2::StrideOverflow => validate_stride(u32::MAX).is_err(),
        ImageFaultKindV2::ByteBomb => validate_byte_length(ENCODED_BYTE_LIMIT + 1).is_err(),
        ImageFaultKindV2::UnsupportedColor => validate_color(8, 2).is_err(),
        ImageFaultKindV2::TruncatedData => preflight_bytes(PNG_SIGNATURE).is_err(),
    }
}

fn preflight_bytes(bytes: &[u8]) -> ImageResultV2<()> {
    if !bytes.starts_with(PNG_SIGNATURE) {
        return Err(ImageFaultKindV2::MalformedSignature);
    }
    if bytes.len() < 33 {
        return Err(ImageFaultKindV2::TruncatedData);
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().expect("IHDR width"));
    let height = u32::from_be_bytes(bytes[20..24].try_into().expect("IHDR height"));
    validate_dimensions(width, height)?;
    validate_stride(width)?;
    validate_byte_length(bytes.len())?;
    validate_color(bytes[24], bytes[25])?;
    if bytes.len() < 12 || &bytes[bytes.len() - 8..bytes.len() - 4] != b"IEND" {
        return Err(ImageFaultKindV2::TruncatedData);
    }
    Ok(())
}

fn validate_dimensions(width: u32, height: u32) -> ImageResultV2<()> {
    if width == 0 || height == 0 || width > DIMENSION_LIMIT || height > DIMENSION_LIMIT {
        Err(ImageFaultKindV2::DimensionLimit)
    } else {
        Ok(())
    }
}

fn validate_stride(width: u32) -> ImageResultV2<u32> {
    width.checked_mul(4).ok_or(ImageFaultKindV2::StrideOverflow)
}

fn validate_byte_length(length: usize) -> ImageResultV2<()> {
    if length > ENCODED_BYTE_LIMIT {
        Err(ImageFaultKindV2::ByteBomb)
    } else {
        Ok(())
    }
}

fn validate_color(bit_depth: u8, color_type: u8) -> ImageResultV2<()> {
    if bit_depth == 8 && color_type == 6 {
        Ok(())
    } else {
        Err(ImageFaultKindV2::UnsupportedColor)
    }
}
