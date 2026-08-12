pub(crate) const FIXED_ONE_V2: i64 = 65_536;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PointV2 {
    pub(crate) x: i64,
    pub(crate) y: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RectV2 {
    pub(crate) x: i64,
    pub(crate) y: i64,
    pub(crate) width: i64,
    pub(crate) height: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AffineV2 {
    pub(crate) values: [i64; 6],
    pub(crate) origin: PointV2,
}

impl AffineV2 {
    pub(crate) const IDENTITY: Self = Self {
        values: [FIXED_ONE_V2, 0, 0, FIXED_ONE_V2, 0, 0],
        origin: PointV2 { x: 0, y: 0 },
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AxisV2 {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AnchorComponentV2 {
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AnchorTargetV2 {
    Viewport,
    Parent,
    Node(u32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlacementInputV2 {
    Root,
    Layout {
        width: i32,
        height: i32,
        transform: AffineV2,
    },
    Free {
        width: i32,
        height: i32,
        self_anchor: [AnchorComponentV2; 2],
        target: AnchorTargetV2,
        target_anchor: [AnchorComponentV2; 2],
        offset: PointV2,
        transform: AffineV2,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NodeInputV2 {
    pub(crate) key: u32,
    pub(crate) path: Option<String>,
    pub(crate) parent: Option<u32>,
    pub(crate) placement: PlacementInputV2,
    pub(crate) axis: AxisV2,
    pub(crate) padding: [i32; 4],
    pub(crate) gap: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PathVerbInputV2 {
    Move(PointV2),
    Line(PointV2),
    Quadratic(PointV2, PointV2),
    Cubic(PointV2, PointV2, PointV2),
    Close,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PathInputV2 {
    pub(crate) key: u32,
    pub(crate) owner: u32,
    pub(crate) verbs: Vec<PathVerbInputV2>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ShapeGeometryInputV2 {
    Rect(RectV2),
    Circle { center: PointV2, radius: i64 },
    Polygon { points: Vec<PointV2> },
    Path { path: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ShapeInputV2 {
    pub(crate) key: u32,
    pub(crate) owner: u32,
    pub(crate) geometry: ShapeGeometryInputV2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FillRuleV2 {
    NonZero,
    EvenOdd,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CoverageInputV2 {
    Fill { shape: u32, rule: FillRuleV2 },
    RoundStroke { shape: u32, width: i64 },
}

impl CoverageInputV2 {
    pub(crate) const fn shape(self) -> u32 {
        match self {
            Self::Fill { shape, .. } | Self::RoundStroke { shape, .. } => shape,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ClipInputV2 {
    pub(crate) key: u32,
    pub(crate) owner: u32,
    pub(crate) parent: Option<u32>,
    pub(crate) shape: u32,
    pub(crate) rule: FillRuleV2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GradientStopInputV2 {
    pub(crate) offset: u16,
    pub(crate) color: [u8; 4],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BrushInputV2 {
    Solid {
        key: u32,
        color: [u8; 4],
    },
    Linear {
        key: u32,
        stops: Vec<GradientStopInputV2>,
        start: PointV2,
        end: PointV2,
    },
}

impl BrushInputV2 {
    pub(crate) const fn key(&self) -> u32 {
        match self {
            Self::Solid { key, .. } | Self::Linear { key, .. } => *key,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ImageInputV2 {
    pub(crate) key: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) stride: u32,
    pub(crate) bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PaintContentInputV2 {
    Coverage {
        coverage: CoverageInputV2,
        brush: u32,
        opacity: u8,
        clip: Option<u32>,
    },
    Image {
        image: u32,
        source: RectV2,
        destination: RectV2,
        opacity: u8,
        clip: Option<u32>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PaintInputV2 {
    pub(crate) owner: u32,
    pub(crate) item: u32,
    pub(crate) content: PaintContentInputV2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HitInputV2 {
    pub(crate) owner: u32,
    pub(crate) item: u32,
    pub(crate) coverage: CoverageInputV2,
    pub(crate) clip: Option<u32>,
    pub(crate) accepts: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SemanticInputV2 {
    pub(crate) owner: u32,
    pub(crate) item: u32,
    pub(crate) shape: u32,
    pub(crate) rule: FillRuleV2,
    pub(crate) clip: Option<u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ReceiptInputV2 {
    pub(crate) generation: Option<u64>,
    pub(crate) mutation_count: u64,
    pub(crate) invalidation: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SceneInputV2 {
    pub(crate) viewport: (u32, u32),
    pub(crate) receipt: ReceiptInputV2,
    pub(crate) nodes: Vec<NodeInputV2>,
    pub(crate) paths: Vec<PathInputV2>,
    pub(crate) shapes: Vec<ShapeInputV2>,
    pub(crate) clips: Vec<ClipInputV2>,
    pub(crate) brushes: Vec<BrushInputV2>,
    pub(crate) images: Vec<ImageInputV2>,
    pub(crate) paints: Vec<PaintInputV2>,
    pub(crate) hits: Vec<HitInputV2>,
    pub(crate) semantics: Vec<SemanticInputV2>,
    pub(crate) queries: Vec<PointV2>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LiteralObservationInputV2 {
    pub(crate) step: u8,
    pub(crate) scene: SceneInputV2,
}
