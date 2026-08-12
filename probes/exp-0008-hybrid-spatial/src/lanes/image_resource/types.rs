#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum ImageCandidateV2 {
    Png,
    Image,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ImageOutcomeV2 {
    Pass,
    Adapt,
    Stop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ImageCandidateRegistrationV2 {
    pub(crate) kind: ImageCandidateV2,
    pub(crate) name: &'static str,
    pub(crate) version: &'static str,
    pub(crate) features: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum ImageObligationV2 {
    Dimensions,
    Stride,
    Rgba8,
    Alpha,
    Gamma,
    Profile,
    Orientation,
}

impl ImageObligationV2 {
    pub(crate) const ALL: [Self; 7] = [
        Self::Dimensions,
        Self::Stride,
        Self::Rgba8,
        Self::Alpha,
        Self::Gamma,
        Self::Profile,
        Self::Orientation,
    ];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ImageOrientationV2 {
    None,
    Rotate90,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ImageRecordV2 {
    pub(crate) ordinal: u8,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) stride: u32,
    pub(crate) rgba8: Vec<u8>,
    pub(crate) gamma_scaled: Option<u32>,
    pub(crate) profile_fingerprint: Option<u64>,
    pub(crate) orientation: ImageOrientationV2,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ImageCaseV2 {
    pub(crate) ordinal: u8,
    pub(crate) name: &'static str,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) png_bytes: Vec<u8>,
    pub(crate) expected: ImageRecordV2,
    pub(crate) obligations: Vec<ImageObligationV2>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ImageRunV2 {
    pub(crate) candidate: Option<ImageCandidateV2>,
    pub(crate) used_orientation_adapter: bool,
    pub(crate) records: Vec<ImageRecordV2>,
}

impl ImageRunV2 {
    pub(crate) fn literal(records: Vec<ImageRecordV2>) -> Self {
        Self {
            candidate: None,
            used_orientation_adapter: false,
            records,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ImageMismatchV2 {
    pub(crate) record: u8,
    pub(crate) field: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ImageClassificationV2 {
    pub(crate) candidate: ImageCandidateV2,
    pub(crate) outcome: ImageOutcomeV2,
    pub(crate) reason: &'static str,
    pub(crate) first_mismatch: Option<ImageMismatchV2>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ImageFaultKindV2 {
    MalformedSignature,
    DimensionLimit,
    StrideOverflow,
    ByteBomb,
    UnsupportedColor,
    TruncatedData,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ImageFaultV2 {
    pub(crate) kind: ImageFaultKindV2,
    pub(crate) literal: bool,
    pub(crate) png: bool,
    pub(crate) image: bool,
}

pub(crate) fn profile_fingerprint(bytes: &[u8]) -> u64 {
    bytes.iter().fold(14_695_981_039_346_656_037, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(1_099_511_628_211)
    })
}

pub(crate) fn orientation_from_exif(bytes: &[u8]) -> ImageOrientationV2 {
    if bytes.len() >= 20
        && &bytes[..2] == b"II"
        && u16::from_le_bytes([bytes[2], bytes[3]]) == 42
        && u16::from_le_bytes([bytes[8], bytes[9]]) > 0
        && u16::from_le_bytes([bytes[10], bytes[11]]) == 0x0112
        && u16::from_le_bytes([bytes[18], bytes[19]]) == 6
    {
        ImageOrientationV2::Rotate90
    } else {
        ImageOrientationV2::None
    }
}

pub(crate) type ImageResultV2<T> = Result<T, ImageFaultKindV2>;
