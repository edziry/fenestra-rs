use super::png_bytes::{PngMetadataV2, rgba8_png};
use super::types::{
    ImageCaseV2, ImageObligationV2 as Obligation, ImageOrientationV2, ImageRecordV2,
    profile_fingerprint,
};

const ICC_PROFILE: &[u8] = b"Fenestra bounded sRGB profile v2";
const ROTATE_90_EXIF: [u8; 26] = [
    b'I', b'I', 42, 0, 8, 0, 0, 0, 1, 0, 0x12, 0x01, 3, 0, 1, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0,
];

pub(crate) fn image_cases_v2() -> Vec<ImageCaseV2> {
    vec![
        image_case(
            0,
            "rgba-control",
            1,
            1,
            vec![10, 20, 30, 255],
            None,
            None,
            None,
            vec![
                Obligation::Dimensions,
                Obligation::Stride,
                Obligation::Rgba8,
            ],
        ),
        image_case(
            1,
            "alpha-gamma",
            2,
            2,
            vec![
                200, 20, 40, 128, 20, 180, 60, 255, 30, 50, 220, 64, 255, 255, 255, 0,
            ],
            Some(45_455),
            None,
            None,
            vec![Obligation::Alpha, Obligation::Gamma],
        ),
        image_case(
            2,
            "icc-profile",
            1,
            1,
            vec![90, 110, 130, 255],
            None,
            Some(ICC_PROFILE),
            None,
            vec![Obligation::Profile],
        ),
        image_case(
            3,
            "exif-orientation",
            2,
            1,
            vec![240, 10, 20, 255, 20, 40, 230, 255],
            None,
            None,
            Some(&ROTATE_90_EXIF),
            vec![Obligation::Orientation],
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
fn image_case(
    ordinal: u8,
    name: &'static str,
    width: u32,
    height: u32,
    rgba8: Vec<u8>,
    gamma_scaled: Option<u32>,
    icc_profile: Option<&[u8]>,
    exif: Option<&[u8]>,
    obligations: Vec<Obligation>,
) -> ImageCaseV2 {
    let png_bytes = rgba8_png(
        width,
        height,
        &rgba8,
        PngMetadataV2 {
            gamma_scaled,
            icc_profile,
            exif,
        },
    );
    let expected = ImageRecordV2 {
        ordinal,
        width,
        height,
        stride: width * 4,
        rgba8,
        gamma_scaled,
        profile_fingerprint: icc_profile.map(profile_fingerprint),
        orientation: if exif.is_some() {
            ImageOrientationV2::Rotate90
        } else {
            ImageOrientationV2::None
        },
    };
    ImageCaseV2 {
        ordinal,
        name,
        width,
        height,
        png_bytes,
        expected,
        obligations,
    }
}
