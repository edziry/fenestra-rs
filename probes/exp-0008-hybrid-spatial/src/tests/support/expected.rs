use crate::baseline::{
    CaseKindV2, CorpusObligationV2 as O, CorpusOperationV2 as Op, PlacementModeV2 as P, QuerySetV2,
};

pub(crate) const CASE_NAMES: [&str; 14] = [
    "all-layout",
    "all-free",
    "free-to-layout",
    "layout-to-free",
    "mixed-siblings",
    "transparent-wrapper",
    "split-geometry",
    "transformed-clip",
    "polygon-path",
    "rich-paint",
    "anchor-forward",
    "zero-extent",
    "runtime-mutation",
    "runtime-rollback",
];

pub(crate) const SECTION_NAMES: [&str; 10] = [
    "receipt",
    "mapping",
    "source",
    "geometry",
    "clips",
    "paints",
    "hits",
    "semantics",
    "queries",
    "raster",
];

pub(crate) const CONTROL_FAMILIES: [&str; 7] = [
    "metadata", "records", "fields", "queries", "raster", "faults", "codec",
];

pub(crate) const SPATIAL_LIMITS: [usize; 30] = [
    256, 1024, 256, 512, 1024, 512, 256, 256, 4096, 4096, 2048, 64, 32, 64, 64, 128, 192, 256,
    1024, 256, 32, 4096, 4_194_304, 32, 64, 64, 4096, 65_536, 192, 256,
];

pub(crate) const OBSERVATION_COUNTS: [usize; 14] = [2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 9, 1];

pub(crate) struct ExpectedCaseV2 {
    pub(crate) kind: CaseKindV2,
    pub(crate) placements: &'static [P],
    pub(crate) operations: &'static [Op],
    pub(crate) obligations: &'static [O],
    pub(crate) scalars: &'static [i64],
    pub(crate) query_set: QuerySetV2,
}

const S: i64 = 65_536;

pub(crate) const CASES: [ExpectedCaseV2; 14] = [
    ExpectedCaseV2 {
        kind: CaseKindV2::Direct,
        placements: &[P::Root, P::Layout, P::Layout, P::Layout],
        operations: &[Op::Direct, Op::Resize],
        obligations: &[O::AllLayout, O::NestedDescendant, O::Resize],
        scalars: &[192 * S, 128 * S, 4 * S, 2 * S, 32 * S, 20 * S, 48 * S],
        query_set: QuerySetV2::DirectComplete,
    },
    ExpectedCaseV2 {
        kind: CaseKindV2::Direct,
        placements: &[P::Root, P::Free, P::Free, P::Free],
        operations: &[Op::Direct, Op::Resize],
        obligations: &[O::AllFree, O::NestedDescendant, O::Resize],
        scalars: &[192 * S, 128 * S, 3 * S / 2, -S / 2, 36 * S, 24 * S],
        query_set: QuerySetV2::DirectComplete,
    },
    ExpectedCaseV2 {
        kind: CaseKindV2::Direct,
        placements: &[P::Root, P::Free, P::Layout, P::Layout],
        operations: &[Op::Direct, Op::ResizeHost],
        obligations: &[O::OuterFreeInnerLayout, O::NestedDescendant, O::Resize],
        scalars: &[192 * S, 128 * S, 12 * S, 8 * S, 80 * S, 56 * S, 3 * S],
        query_set: QuerySetV2::DirectComplete,
    },
    ExpectedCaseV2 {
        kind: CaseKindV2::Direct,
        placements: &[P::Root, P::Layout, P::Free, P::Layout],
        operations: &[Op::Direct, Op::ResizeHost],
        obligations: &[O::OuterLayoutInnerFree, O::NestedDescendant, O::Resize],
        scalars: &[192 * S, 128 * S, 52 * S, 36 * S, 5 * S, 7 * S, S],
        query_set: QuerySetV2::DirectComplete,
    },
    ExpectedCaseV2 {
        kind: CaseKindV2::Direct,
        placements: &[P::Root, P::Layout, P::Free],
        operations: &[Op::Direct],
        obligations: &[O::MixedSiblingModes, O::FreeConsumesNoLayout],
        scalars: &[192 * S, 128 * S, 40 * S, 20 * S, 41 * S, 3 * S],
        query_set: QuerySetV2::DirectComplete,
    },
    ExpectedCaseV2 {
        kind: CaseKindV2::Direct,
        placements: &[P::Root, P::Layout, P::Layout],
        operations: &[Op::Direct],
        obligations: &[O::TransparentParticipatesLayout, O::TransparentNoPaintHit],
        scalars: &[192 * S, 128 * S, 64 * S, 40 * S, 24 * S, 12 * S],
        query_set: QuerySetV2::DirectComplete,
    },
    ExpectedCaseV2 {
        kind: CaseKindV2::Direct,
        placements: &[P::Root, P::Layout],
        operations: &[Op::Direct],
        obligations: &[
            O::SplitLayoutExtent,
            O::PaintOverflow,
            O::CircularHit,
            O::IndependentSemantics,
        ],
        scalars: &[192 * S, 128 * S, 48 * S, 32 * S, -4 * S, 56 * S, 10 * S],
        query_set: QuerySetV2::DirectComplete,
    },
    ExpectedCaseV2 {
        kind: CaseKindV2::Direct,
        placements: &[P::Root, P::Free, P::Free, P::Free],
        operations: &[Op::Direct],
        obligations: &[O::ThreeLevelTransforms, O::TwoLinkClip],
        scalars: &[192 * S, 128 * S, 8 * S, 6 * S, 0, S, -S, 2 * S],
        query_set: QuerySetV2::DirectComplete,
    },
    ExpectedCaseV2 {
        kind: CaseKindV2::Direct,
        placements: &[P::Root, P::Free, P::Free],
        operations: &[Op::Direct],
        obligations: &[
            O::PolygonConcavity,
            O::AabbMiss,
            O::BothFillRules,
            O::TwoSubpaths,
            O::SelfIntersection,
            O::ZeroSegment,
            O::Quadratic,
            O::Cubic,
            O::Fill,
            O::RoundStroke,
        ],
        scalars: &[192 * S, 128 * S, 5 * S, 17 * S, S / 4, 3 * S / 2],
        query_set: QuerySetV2::DirectComplete,
    },
    ExpectedCaseV2 {
        kind: CaseKindV2::Direct,
        placements: &[P::Root, P::Free, P::Free, P::Free],
        operations: &[Op::Direct],
        obligations: &[
            O::Solid,
            O::PartialAlpha,
            O::LinearGradient,
            O::PremultipliedImage,
            O::SourceOver,
            O::ExplicitClip,
        ],
        scalars: &[192 * S, 128 * S, 16 * S, 12 * S, 255, 128, 64, 32],
        query_set: QuerySetV2::DirectComplete,
    },
    ExpectedCaseV2 {
        kind: CaseKindV2::Direct,
        placements: &[P::Root, P::Free, P::Free, P::Free],
        operations: &[Op::Direct, Op::DependencyCycleControl],
        obligations: &[
            O::LaterAnchor,
            O::ParentAnchor,
            O::ViewportAnchor,
            O::DependencyCycleControl,
        ],
        scalars: &[192 * S, 128 * S, S / 2, 9 * S, -3 * S, 28 * S],
        query_set: QuerySetV2::DirectComplete,
    },
    ExpectedCaseV2 {
        kind: CaseKindV2::Direct,
        placements: &[P::Root, P::Free, P::Free, P::Free],
        operations: &[Op::Direct],
        obligations: &[O::ZeroByN, O::NByZero, O::ZeroByZero],
        scalars: &[192 * S, 128 * S, 0, 19 * S, 23 * S, 0],
        query_set: QuerySetV2::DirectComplete,
    },
    ExpectedCaseV2 {
        kind: CaseKindV2::RuntimeMutation,
        placements: &[P::Root, P::Layout, P::Layout, P::Free, P::Layout],
        operations: &[
            Op::Init,
            Op::Resize,
            Op::SetSpanX,
            Op::SetTone,
            Op::SetPolicy,
            Op::Insert,
            Op::Move,
            Op::Update,
            Op::Remove,
        ],
        obligations: &[O::RuntimeNineSteps, O::KeyedMutation],
        scalars: &[192 * S, 128 * S, 224 * S, 160 * S, 20, 30, 8, 9],
        query_set: QuerySetV2::RuntimePixelCenters,
    },
    ExpectedCaseV2 {
        kind: CaseKindV2::RuntimeRollback,
        placements: &[P::Root, P::Layout, P::Layout, P::Free, P::Layout],
        operations: &[Op::SingularAttempt],
        obligations: &[O::SingularRollback, O::ExactRetainedState],
        scalars: &[0, 8, 9, 226, 227],
        query_set: QuerySetV2::RuntimePixelCenters,
    },
];
