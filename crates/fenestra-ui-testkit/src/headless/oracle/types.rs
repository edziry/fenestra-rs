use fenestra_ui_ir::prototype::InputPolicy;
use fenestra_ui_runtime::prototype::{
    HeadlessRect, HeadlessSemanticAction, HeadlessSemanticRole, HeadlessSurface, RuntimeGeneration,
};

use crate::semantic::NodePathV1;

/// One identity-free computed-style record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedHeadlessComputedStyleV1 {
    pub(super) path: NodePathV1,
    pub(super) width: i32,
    pub(super) height: i32,
    pub(super) color: [u8; 4],
    pub(super) visible: bool,
    pub(super) input: InputPolicy,
}

impl NormalizedHeadlessComputedStyleV1 {
    /// Returns the semantic node address.
    #[must_use]
    pub const fn path(&self) -> &NodePathV1 {
        &self.path
    }

    /// Returns the materialized width.
    #[must_use]
    pub const fn width(&self) -> i32 {
        self.width
    }

    /// Returns the materialized height.
    #[must_use]
    pub const fn height(&self) -> i32 {
        self.height
    }

    /// Returns the materialized color.
    #[must_use]
    pub const fn color(&self) -> [u8; 4] {
        self.color
    }

    /// Returns the local visibility value.
    #[must_use]
    pub const fn visible(&self) -> bool {
        self.visible
    }

    /// Returns the local input policy.
    #[must_use]
    pub const fn input(&self) -> InputPolicy {
        self.input
    }
}

/// One identity-free absolute geometry record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedHeadlessGeometryV1 {
    pub(super) path: NodePathV1,
    pub(super) bounds: HeadlessRect,
    pub(super) clip: HeadlessRect,
}

impl NormalizedHeadlessGeometryV1 {
    /// Returns the semantic node address.
    #[must_use]
    pub const fn path(&self) -> &NodePathV1 {
        &self.path
    }

    /// Returns the unclipped absolute bounds.
    #[must_use]
    pub const fn bounds(&self) -> HeadlessRect {
        self.bounds
    }

    /// Returns the effective clip.
    #[must_use]
    pub const fn clip(&self) -> HeadlessRect {
        self.clip
    }
}

/// One identity-free semantic record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedHeadlessSemanticV1 {
    pub(super) path: NodePathV1,
    pub(super) role: HeadlessSemanticRole,
    pub(super) label: u32,
    pub(super) action: HeadlessSemanticAction,
}

impl NormalizedHeadlessSemanticV1 {
    /// Returns the semantic node address.
    #[must_use]
    pub const fn path(&self) -> &NodePathV1 {
        &self.path
    }

    /// Returns the closed role.
    #[must_use]
    pub const fn role(&self) -> HeadlessSemanticRole {
        self.role
    }

    /// Returns the fixed label symbol.
    #[must_use]
    pub const fn label(&self) -> u32 {
        self.label
    }

    /// Returns the closed action.
    #[must_use]
    pub const fn action(&self) -> HeadlessSemanticAction {
        self.action
    }
}

/// One identity-free ordered hit region.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedHeadlessHitRegionV1 {
    pub(super) path: NodePathV1,
    pub(super) clip: HeadlessRect,
}

impl NormalizedHeadlessHitRegionV1 {
    /// Returns the semantic node address.
    #[must_use]
    pub const fn path(&self) -> &NodePathV1 {
        &self.path
    }

    /// Returns the effective hit clip.
    #[must_use]
    pub const fn clip(&self) -> HeadlessRect {
        self.clip
    }
}

/// One identity-free authored-order scene rectangle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedHeadlessSceneRectangleV1 {
    pub(super) path: NodePathV1,
    pub(super) rectangle: HeadlessRect,
    pub(super) color: [u8; 4],
}

impl NormalizedHeadlessSceneRectangleV1 {
    /// Returns the semantic node address.
    #[must_use]
    pub const fn path(&self) -> &NodePathV1 {
        &self.path
    }

    /// Returns the clipped scene rectangle.
    #[must_use]
    pub const fn rectangle(&self) -> HeadlessRect {
        self.rectangle
    }

    /// Returns the materialized scene color.
    #[must_use]
    pub const fn color(&self) -> [u8; 4] {
        self.color
    }
}

/// Complete generation-free normalized headless projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedHeadlessProjectionV1 {
    pub(super) surface: HeadlessSurface,
    pub(super) computed_styles: Vec<NormalizedHeadlessComputedStyleV1>,
    pub(super) geometries: Vec<NormalizedHeadlessGeometryV1>,
    pub(super) semantics: Vec<NormalizedHeadlessSemanticV1>,
    pub(super) hit_regions: Vec<NormalizedHeadlessHitRegionV1>,
    pub(super) scene_rectangles: Vec<NormalizedHeadlessSceneRectangleV1>,
}

impl NormalizedHeadlessProjectionV1 {
    /// Returns the logical surface.
    #[must_use]
    pub const fn surface(&self) -> HeadlessSurface {
        self.surface
    }

    /// Returns computed records in authored order.
    #[must_use]
    pub fn computed_styles(&self) -> &[NormalizedHeadlessComputedStyleV1] {
        &self.computed_styles
    }

    /// Returns geometry records in authored order.
    #[must_use]
    pub fn geometries(&self) -> &[NormalizedHeadlessGeometryV1] {
        &self.geometries
    }

    /// Returns semantic records in authored order.
    #[must_use]
    pub fn semantics(&self) -> &[NormalizedHeadlessSemanticV1] {
        &self.semantics
    }

    /// Returns hit regions in authored scene order.
    #[must_use]
    pub fn hit_regions(&self) -> &[NormalizedHeadlessHitRegionV1] {
        &self.hit_regions
    }

    /// Returns scene rectangles in authored order.
    #[must_use]
    pub fn scene_rectangles(&self) -> &[NormalizedHeadlessSceneRectangleV1] {
        &self.scene_rectangles
    }
}

/// Runtime generation carried separately from normalized projection identity.
pub struct ObservedHeadlessProjectionV1 {
    pub(super) generation: RuntimeGeneration,
    pub(super) projection: NormalizedHeadlessProjectionV1,
}

impl ObservedHeadlessProjectionV1 {
    /// Returns the committed runtime generation.
    #[must_use]
    pub const fn generation(&self) -> RuntimeGeneration {
        self.generation
    }

    /// Returns the identity-free projection.
    #[must_use]
    pub const fn projection(&self) -> &NormalizedHeadlessProjectionV1 {
        &self.projection
    }
}

/// Projection family selected by the first mismatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessMismatchKindV1 {
    /// Computed-style records.
    ComputedStyle,
    /// Geometry records.
    Geometry,
    /// Semantic records.
    Semantics,
    /// Hit regions.
    HitRegions,
    /// Scene rectangles.
    SceneRectangles,
}

impl HeadlessMismatchKindV1 {
    /// All mismatch families in deterministic priority order.
    pub const ALL: [Self; 5] = [
        Self::ComputedStyle,
        Self::Geometry,
        Self::Semantics,
        Self::HitRegions,
        Self::SceneRectangles,
    ];
}

/// First unequal field within one normalized projection record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessMismatchFieldV1 {
    /// Semantic path or ordered membership.
    Path,
    /// Computed width.
    Width,
    /// Computed height.
    Height,
    /// Computed or scene color.
    Color,
    /// Local visibility.
    Visible,
    /// Local input policy.
    Input,
    /// Unclipped geometry bounds.
    Bounds,
    /// Effective geometry or hit clip.
    Clip,
    /// Semantic role.
    Role,
    /// Semantic label.
    Label,
    /// Semantic action.
    Action,
    /// Scene rectangle.
    Rectangle,
}

/// Privacy-safe semantic location of a projection mismatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HeadlessMismatchLocationV1 {
    /// A record path from the expected or observed side.
    Path(NodePathV1),
    /// Neither side supplies a path at the differing index.
    End,
}

/// First deterministic difference between two normalized projections.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HeadlessMismatchV1 {
    pub(super) kind: HeadlessMismatchKindV1,
    pub(super) index: usize,
    pub(super) field: HeadlessMismatchFieldV1,
    pub(super) location: HeadlessMismatchLocationV1,
}

impl HeadlessMismatchV1 {
    /// Returns the mismatching projection family.
    #[must_use]
    pub const fn kind(&self) -> HeadlessMismatchKindV1 {
        self.kind
    }

    /// Returns the first differing ordered record index.
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Returns the first unequal field at that index.
    #[must_use]
    pub const fn field(&self) -> HeadlessMismatchFieldV1 {
        self.field
    }

    /// Returns the privacy-safe semantic location.
    #[must_use]
    pub const fn location(&self) -> &HeadlessMismatchLocationV1 {
        &self.location
    }
}

/// Registered test-only defect adapters for projection comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessProjectionFaultV1 {
    /// Perturbs multiple computed-style fields.
    ComputedStyle,
    /// Perturbs geometry order.
    GeometryOrder,
    /// Perturbs semantic membership.
    SemanticMembership,
    /// Perturbs hit-region order.
    HitOrder,
    /// Perturbs multiple scene fields.
    SceneOutput,
}

impl HeadlessProjectionFaultV1 {
    /// All registered faults in mismatch-family priority order.
    pub const ALL: [Self; 5] = [
        Self::ComputedStyle,
        Self::GeometryOrder,
        Self::SemanticMembership,
        Self::HitOrder,
        Self::SceneOutput,
    ];
}

pub(super) const fn rect(x: i32, y: i32, width: i32, height: i32) -> HeadlessRect {
    HeadlessRect::new(x, y, width, height)
}

pub(super) type ProjectionRect = HeadlessRect;

pub(super) const fn semantic_role() -> HeadlessSemanticRole {
    HeadlessSemanticRole::Control
}

pub(super) const fn semantic_action() -> HeadlessSemanticAction {
    HeadlessSemanticAction::Activate
}
