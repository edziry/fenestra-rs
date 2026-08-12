pub(crate) const CPU_SCALE_V2: i32 = 256;
pub(crate) const CPU_PIXEL_LIMIT_V2: usize = 4_096;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum CpuCandidateV2 {
    TinySkia,
    Raqote,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CpuOutcomeV2 {
    Pass,
    Adapt,
    Stop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CpuCandidateRegistrationV2 {
    pub(crate) kind: CpuCandidateV2,
    pub(crate) name: &'static str,
    pub(crate) version: &'static str,
    pub(crate) features: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum CpuObligationV2 {
    OpaqueControl,
    RichReference,
    Shape,
    Alpha,
    Transform,
    Clip,
    Gradient,
    Image,
}

impl CpuObligationV2 {
    pub(crate) const ALL: [Self; 8] = [
        Self::OpaqueControl,
        Self::RichReference,
        Self::Shape,
        Self::Alpha,
        Self::Transform,
        Self::Clip,
        Self::Gradient,
        Self::Image,
    ];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CpuTransformV2 {
    pub(crate) coefficients: [i32; 6],
}

impl CpuTransformV2 {
    pub(crate) const IDENTITY: Self = Self {
        coefficients: [CPU_SCALE_V2, 0, 0, CPU_SCALE_V2, 0, 0],
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CpuSamplingV2 {
    Nearest,
    Bilinear,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CpuCommandV2 {
    SolidRect {
        rect: [i32; 4],
        color: [u8; 4],
        transform: CpuTransformV2,
        clip: Option<[i32; 4]>,
    },
    SolidPolygon {
        points: Vec<[i32; 2]>,
        color: [u8; 4],
        transform: CpuTransformV2,
        clip: Option<[i32; 4]>,
    },
    LinearGradient {
        rect: [i32; 4],
        start: [i32; 2],
        end: [i32; 2],
        colors: [[u8; 4]; 2],
    },
    Image {
        origin: [i32; 2],
        width: u32,
        height: u32,
        stride: u32,
        premultiplied_rgba: Vec<u8>,
        sampling: CpuSamplingV2,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CpuCaseV2 {
    pub(crate) ordinal: u8,
    pub(crate) name: &'static str,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) commands: Vec<CpuCommandV2>,
    pub(crate) obligations: Vec<CpuObligationV2>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CpuRasterV2 {
    pub(crate) ordinal: u8,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CpuRunV2 {
    pub(crate) candidate: Option<CpuCandidateV2>,
    pub(crate) cases: Vec<CpuRasterV2>,
}

impl CpuRunV2 {
    pub(crate) fn literal(cases: Vec<CpuRasterV2>) -> Self {
        Self {
            candidate: None,
            cases,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CpuMismatchV2 {
    pub(crate) case: u8,
    pub(crate) byte: usize,
    pub(crate) expected: u8,
    pub(crate) observed: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CpuClassificationV2 {
    pub(crate) candidate: CpuCandidateV2,
    pub(crate) outcome: CpuOutcomeV2,
    pub(crate) reason: &'static str,
    pub(crate) first_mismatch: Option<CpuMismatchV2>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CpuFaultKindV2 {
    ZeroDimension,
    PixelLimit,
    InvalidImageStride,
    NonFiniteTransform,
    UnsupportedSampling,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CpuFaultV2 {
    pub(crate) kind: CpuFaultKindV2,
    pub(crate) literal: bool,
    pub(crate) tiny_skia: bool,
    pub(crate) raqote: bool,
}

pub(crate) type CpuResultV2<T> = Result<T, CpuFaultKindV2>;
