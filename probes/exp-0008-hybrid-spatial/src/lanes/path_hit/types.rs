pub(crate) const PATH_SCALE_V2: i32 = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum PathHitCandidateV2 {
    Kurbo,
    Lyon,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PathHitOutcomeV2 {
    Pass,
    Adapt,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PathHitCandidateRegistrationV2 {
    pub(crate) kind: PathHitCandidateV2,
    pub(crate) name: &'static str,
    pub(crate) version: &'static str,
    pub(crate) features: &'static str,
    pub(crate) outcome: PathHitOutcomeV2,
    pub(crate) reason: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum PathHitObligationV2 {
    Convex,
    Concave,
    Holes,
    NonZero,
    EvenOdd,
    SelfIntersection,
    Degenerate,
    Quadratic,
    Cubic,
    Fill,
    RoundStroke,
    Clip,
    AabbMiss,
    ReversePainter,
}

impl PathHitObligationV2 {
    pub(crate) const ALL: [Self; 14] = [
        Self::Convex,
        Self::Concave,
        Self::Holes,
        Self::NonZero,
        Self::EvenOdd,
        Self::SelfIntersection,
        Self::Degenerate,
        Self::Quadratic,
        Self::Cubic,
        Self::Fill,
        Self::RoundStroke,
        Self::Clip,
        Self::AabbMiss,
        Self::ReversePainter,
    ];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PathVerbV2 {
    Move([i32; 2]),
    Line([i32; 2]),
    Quadratic([i32; 2], [i32; 2]),
    Cubic([i32; 2], [i32; 2], [i32; 2]),
    Close,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FillRuleV2 {
    NonZero,
    EvenOdd,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PathCoverageV2 {
    Fill(FillRuleV2),
    RoundStroke { width: i32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PathLayerV2 {
    pub(crate) verbs: Vec<PathVerbV2>,
    pub(crate) coverage: PathCoverageV2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PathQueryV2 {
    pub(crate) point: [i32; 2],
    pub(crate) nonrectangular_aabb_miss: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PathHitCaseV2 {
    pub(crate) ordinal: u8,
    pub(crate) name: &'static str,
    pub(crate) layers: Vec<PathLayerV2>,
    pub(crate) clip: Option<PathLayerV2>,
    pub(crate) queries: Vec<PathQueryV2>,
    pub(crate) obligations: Vec<PathHitObligationV2>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PathHitRecordV2 {
    pub(crate) case: u8,
    pub(crate) query: u8,
    pub(crate) layer_hits: Vec<bool>,
    pub(crate) topmost: Option<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PathHitRunV2 {
    pub(crate) records: Vec<PathHitRecordV2>,
    pub(crate) triangle_witnesses: usize,
    pub(crate) reverse_painter_queries: usize,
    pub(crate) nonrectangular_aabb_misses: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PathHitFaultKindV2 {
    MissingMove,
    OpenFillSubpath,
    NonFiniteCoordinate,
    TessellationLimit,
    InvalidStrokeWidth,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PathHitFaultV2 {
    pub(crate) kind: PathHitFaultKindV2,
    pub(crate) literal: bool,
    pub(crate) kurbo: bool,
    pub(crate) lyon: bool,
}

pub(crate) type PathHitResultV2<T> = Result<T, PathHitFaultKindV2>;
