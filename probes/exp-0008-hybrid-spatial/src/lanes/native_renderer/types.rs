pub(crate) const NATIVE_SCALE_V2: i32 = 256;
pub(crate) const NATIVE_RESOURCE_LIMIT_V2: usize = 16_384;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum NativeCandidateV2 {
    Vello,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeOutcomeV2 {
    Pass,
    Adapt,
    Stop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeCandidateRegistrationV2 {
    pub(crate) kind: NativeCandidateV2,
    pub(crate) name: &'static str,
    pub(crate) version: &'static str,
    pub(crate) renderer_features: &'static str,
    pub(crate) gpu_version: &'static str,
    pub(crate) gpu_features: &'static str,
    pub(crate) targets: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum NativeObligationV2 {
    Scene,
    Shape,
    Alpha,
    Transform,
    Clip,
    Gradient,
    Image,
    PainterOrder,
    Resize,
    ResourceLifetime,
    TargetQualification,
}

impl NativeObligationV2 {
    pub(crate) const ALL: [Self; 11] = [
        Self::Scene,
        Self::Shape,
        Self::Alpha,
        Self::Transform,
        Self::Clip,
        Self::Gradient,
        Self::Image,
        Self::PainterOrder,
        Self::Resize,
        Self::ResourceLifetime,
        Self::TargetQualification,
    ];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeTransformV2 {
    pub(crate) coefficients: [i32; 6],
}

impl NativeTransformV2 {
    pub(crate) const IDENTITY: Self = Self {
        coefficients: [NATIVE_SCALE_V2, 0, 0, NATIVE_SCALE_V2, 0, 0],
    };
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NativeCommandV2 {
    SolidRect {
        rect: [i32; 4],
        color: [u8; 4],
        transform: NativeTransformV2,
    },
    GradientRect {
        rect: [i32; 4],
        start: [i32; 2],
        end: [i32; 2],
        colors: [[u8; 4]; 2],
    },
    ClipRect {
        clip: [i32; 4],
        rect: [i32; 4],
        color: [u8; 4],
    },
    Image {
        origin: [i32; 2],
        width: u32,
        height: u32,
        rgba8: Vec<u8>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NativeCaseV2 {
    pub(crate) ordinal: u8,
    pub(crate) name: &'static str,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) commands: Vec<NativeCommandV2>,
    pub(crate) obligations: Vec<NativeObligationV2>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeRecordV2 {
    pub(crate) ordinal: u8,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) commands: u32,
    pub(crate) shapes: u32,
    pub(crate) clips: u32,
    pub(crate) images: u32,
    pub(crate) painter_digest: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NativeRunV2 {
    pub(crate) candidate: Option<NativeCandidateV2>,
    pub(crate) records: Vec<NativeRecordV2>,
    pub(crate) scene_fingerprint: u64,
    pub(crate) encoded_scene_bytes: usize,
    pub(crate) used_vello_scene: bool,
    pub(crate) executed_gpu: bool,
}

impl NativeRunV2 {
    pub(crate) fn literal(records: Vec<NativeRecordV2>) -> Self {
        Self {
            candidate: None,
            records,
            scene_fingerprint: 0,
            encoded_scene_bytes: 0,
            used_vello_scene: false,
            executed_gpu: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeMismatchV2 {
    pub(crate) record: u8,
    pub(crate) field: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeClassificationV2 {
    pub(crate) candidate: NativeCandidateV2,
    pub(crate) outcome: NativeOutcomeV2,
    pub(crate) reason: &'static str,
    pub(crate) first_mismatch: Option<NativeMismatchV2>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeFaultKindV2 {
    MissingCapability,
    ZeroSurface,
    ResourceLimit,
    AdapterUnavailable,
    SurfaceLost,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeFaultV2 {
    pub(crate) kind: NativeFaultKindV2,
    pub(crate) literal: bool,
    pub(crate) vello: bool,
}

pub(crate) type NativeResultV2<T> = Result<T, NativeFaultKindV2>;
