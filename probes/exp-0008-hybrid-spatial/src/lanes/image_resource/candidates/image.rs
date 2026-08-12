use std::io::Cursor;

use image::ImageDecoder;
use image::codecs::png::PngDecoder;

use super::super::faults::{detects as detects_fault, preflight_cases};
use super::super::types::{
    ImageCandidateV2, ImageCaseV2, ImageFaultKindV2, ImageOrientationV2, ImageRecordV2,
    ImageResultV2, ImageRunV2, profile_fingerprint,
};

pub(crate) fn run(cases: &[ImageCaseV2]) -> ImageResultV2<ImageRunV2> {
    preflight_cases(cases)?;
    let records = cases
        .iter()
        .map(decode)
        .collect::<ImageResultV2<Vec<_>>>()?;
    Ok(ImageRunV2 {
        candidate: Some(ImageCandidateV2::Image),
        used_orientation_adapter: false,
        records,
    })
}

pub(crate) fn detects(kind: ImageFaultKindV2) -> bool {
    detects_fault(kind)
}

fn decode(case: &ImageCaseV2) -> ImageResultV2<ImageRecordV2> {
    let mut decoder = PngDecoder::new(Cursor::new(&case.png_bytes))
        .map_err(|_| ImageFaultKindV2::TruncatedData)?;
    let (width, height) = decoder.dimensions();
    let gamma_scaled = decoder
        .gamma_value()
        .map_err(|_| ImageFaultKindV2::TruncatedData)?
        .map(|gamma| (gamma * 100_000.0).round() as u32);
    let profile_fingerprint = decoder
        .icc_profile()
        .map_err(|_| ImageFaultKindV2::TruncatedData)?
        .as_deref()
        .map(profile_fingerprint);
    let orientation = match decoder
        .orientation()
        .map_err(|_| ImageFaultKindV2::TruncatedData)?
    {
        image::metadata::Orientation::NoTransforms => ImageOrientationV2::None,
        image::metadata::Orientation::Rotate90 => ImageOrientationV2::Rotate90,
        _ => return Err(ImageFaultKindV2::UnsupportedColor),
    };
    let mut rgba8 =
        vec![0; usize::try_from(decoder.total_bytes()).map_err(|_| ImageFaultKindV2::ByteBomb)?];
    decoder
        .read_image(&mut rgba8)
        .map_err(|_| ImageFaultKindV2::TruncatedData)?;
    Ok(ImageRecordV2 {
        ordinal: case.ordinal,
        width,
        height,
        stride: width * 4,
        rgba8,
        gamma_scaled,
        profile_fingerprint,
        orientation,
    })
}
