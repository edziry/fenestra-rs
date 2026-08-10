use fenestra_ui_ir::prototype::{PropertyId, PropertyValue, ValidatedStyleProgram};
use fenestra_ui_layout::prototype::LayoutEngineV1;

use crate::logical_tree::NodeId;

use super::{HeadlessProjectionErrorKind, HeadlessProjectionSpec, HeadlessSurface};

/// One logical point in the provisional headless projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HeadlessPoint {
    x: i32,
    y: i32,
}

impl HeadlessPoint {
    /// Creates a logical point.
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Returns the logical horizontal coordinate.
    #[must_use]
    pub const fn x(self) -> i32 {
        self.x
    }

    /// Returns the logical vertical coordinate.
    #[must_use]
    pub const fn y(self) -> i32 {
        self.y
    }
}

/// One absolute logical rectangle in the provisional headless projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HeadlessRect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl HeadlessRect {
    /// Creates an absolute logical rectangle.
    #[must_use]
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns the absolute horizontal origin.
    #[must_use]
    pub const fn x(self) -> i32 {
        self.x
    }

    /// Returns the absolute vertical origin.
    #[must_use]
    pub const fn y(self) -> i32 {
        self.y
    }

    /// Returns the logical width.
    #[must_use]
    pub const fn width(self) -> i32 {
        self.width
    }

    /// Returns the logical height.
    #[must_use]
    pub const fn height(self) -> i32 {
        self.height
    }

    pub(super) const fn is_non_empty(self) -> bool {
        self.width > 0 && self.height > 0
    }

    pub(super) fn contains(self, point: HeadlessPoint) -> bool {
        if !self.is_non_empty() {
            return false;
        }
        let right = self.x.checked_add(self.width);
        let bottom = self.y.checked_add(self.height);
        right.is_some_and(|right| self.x <= point.x() && point.x() < right)
            && bottom.is_some_and(|bottom| self.y <= point.y() && point.y() < bottom)
    }
}

/// Closed role vocabulary for the provisional headless semantic projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessSemanticRole {
    /// One activatable control.
    Control,
}

/// Closed action vocabulary for the provisional headless semantic projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeadlessSemanticAction {
    /// Activate the semantic control.
    Activate,
}

#[derive(Clone, Copy)]
pub(crate) struct HeadlessProjectionFailure {
    kind: HeadlessProjectionErrorKind,
    cause: Option<(NodeId, PropertyId)>,
}

impl HeadlessProjectionFailure {
    pub(crate) const fn new(kind: HeadlessProjectionErrorKind) -> Self {
        Self { kind, cause: None }
    }

    pub(crate) const fn negative(node: NodeId, property: PropertyId) -> Self {
        Self {
            kind: HeadlessProjectionErrorKind::NegativeGeometry,
            cause: Some((node, property)),
        }
    }

    pub(crate) const fn kind(self) -> HeadlessProjectionErrorKind {
        self.kind
    }

    pub(crate) const fn cause(self) -> Option<(NodeId, PropertyId)> {
        self.cause
    }
}

pub(crate) struct HeadlessRuntimeConfig {
    pub(super) style: ValidatedStyleProgram,
    pub(super) spec: HeadlessProjectionSpec,
    pub(super) layout_engine: Box<dyn LayoutEngineV1>,
}

#[derive(Clone)]
pub(crate) struct HeadlessProjectionState {
    pub(super) surface: HeadlessSurface,
    pub(super) computed_styles: Vec<ComputedStyleRecord>,
    pub(super) geometry: Vec<GeometryRecord>,
    pub(super) semantics: Vec<SemanticRecord>,
    pub(super) hit_regions: Vec<HitRegionRecord>,
    pub(super) scene_rectangles: Vec<SceneRectangleRecord>,
}

#[derive(Clone)]
pub(super) struct ComputedStyleRecord {
    pub(super) node: NodeId,
    pub(super) properties: Vec<ComputedProperty>,
}

#[derive(Clone)]
pub(super) struct ComputedProperty {
    pub(super) id: PropertyId,
    pub(super) value: PropertyValue,
}

#[derive(Clone)]
pub(super) struct GeometryRecord {
    pub(super) node: NodeId,
    pub(super) bounds: HeadlessRect,
    pub(super) clip: HeadlessRect,
    pub(super) effective_visible: bool,
}

#[derive(Clone)]
pub(super) struct SemanticRecord {
    pub(super) node: NodeId,
    pub(super) role: HeadlessSemanticRole,
    pub(super) label: u32,
    pub(super) action: HeadlessSemanticAction,
}

#[derive(Clone)]
pub(super) struct HitRegionRecord {
    pub(super) node: NodeId,
    pub(super) clip: HeadlessRect,
}

#[derive(Clone)]
pub(super) struct SceneRectangleRecord {
    pub(super) node: NodeId,
    pub(super) rectangle: HeadlessRect,
    pub(super) color: [u8; 4],
}
