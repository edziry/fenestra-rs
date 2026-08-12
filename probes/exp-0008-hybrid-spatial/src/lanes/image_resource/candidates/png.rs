use std::io::Cursor;

use super::super::faults::{detects as detects_fault, preflight_cases};
use super::super::types::{
    ImageCandidateV2, ImageCaseV2, ImageFaultKindV2, ImageRecordV2, ImageResultV2, ImageRunV2,
    orientation_from_exif, profile_fingerprint,
};

pub(crate) fn run(cases: &[ImageCaseV2]) -> ImageResultV2<ImageRunV2> {
    preflight_cases(cases)?;
    let records = cases
        .iter()
        .map(decode)
        .collect::<ImageResultV2<Vec<_>>>()?;
    Ok(ImageRunV2 {
        candidate: Some(ImageCandidateV2::Png),
        used_orientation_adapter: records
            .iter()
            .any(|record| record.orientation != super::super::types::ImageOrientationV2::None),
        records,
    })
}

pub(crate) fn detects(kind: ImageFaultKindV2) -> bool {
    detects_fault(kind)
}

fn decode(case: &ImageCaseV2) -> ImageResultV2<ImageRecordV2> {
    let decoder = png::Decoder::new(Cursor::new(&case.png_bytes));
    let mut reader = decoder
        .read_info()
        .map_err(|_| ImageFaultKindV2::TruncatedData)?;
    let buffer_size = reader
        .output_buffer_size()
        .ok_or(ImageFaultKindV2::ByteBomb)?;
    let mut bytes = vec![0; buffer_size];
    let output = reader
        .next_frame(&mut bytes)
        .map_err(|_| ImageFaultKindV2::TruncatedData)?;
    bytes.truncate(output.buffer_size());
    let info = reader.info();
    Ok(ImageRecordV2 {
        ordinal: case.ordinal,
        width: output.width,
        height: output.height,
        stride: output.line_size as u32,
        rgba8: bytes,
        gamma_scaled: info.gamma().map(png::ScaledFloat::into_scaled),
        profile_fingerprint: info.icc_profile.as_deref().map(profile_fingerprint),
        orientation: info
            .exif_metadata
            .as_deref()
            .map(orientation_from_exif)
            .unwrap_or(super::super::types::ImageOrientationV2::None),
    })
}
