/// Dense pass-local key for one version-2 spatial node.
///
/// ```compile_fail,E0616
/// use fenestra_ui_spatial::prototype::SpatialNodeKeyV2;
///
/// let key = SpatialNodeKeyV2::new(1);
/// let _ = key.0;
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SpatialNodeKeyV2(u32);

impl SpatialNodeKeyV2 {
    /// Creates a pass-local spatial key.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the numeric pass-local key.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Present logical viewport metadata for one spatial computation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialViewportV2 {
    width: i32,
    height: i32,
}

impl SpatialViewportV2 {
    /// Creates logical viewport metadata.
    #[must_use]
    pub const fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    /// Returns the logical viewport width.
    #[must_use]
    pub const fn width(self) -> i32 {
        self.width
    }

    /// Returns the logical viewport height.
    #[must_use]
    pub const fn height(self) -> i32 {
        self.height
    }
}

/// Raw signed fixed-point storage for one version-2 spatial scalar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialScalarV2(i64);

impl SpatialScalarV2 {
    /// Creates an unvalidated raw scalar.
    #[must_use]
    pub const fn new(raw: i64) -> Self {
        Self(raw)
    }

    /// Returns the raw fixed-point storage.
    #[must_use]
    pub const fn raw(self) -> i64 {
        self.0
    }
}

/// One raw point in a local spatial coordinate system.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialPointV2 {
    x: SpatialScalarV2,
    y: SpatialScalarV2,
}

impl SpatialPointV2 {
    /// Creates a raw point.
    #[must_use]
    pub const fn new(x: SpatialScalarV2, y: SpatialScalarV2) -> Self {
        Self { x, y }
    }

    /// Returns the horizontal component.
    #[must_use]
    pub const fn x(self) -> SpatialScalarV2 {
        self.x
    }

    /// Returns the vertical component.
    #[must_use]
    pub const fn y(self) -> SpatialScalarV2 {
        self.y
    }
}

/// One raw translation offset used by free placement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialOffsetV2 {
    x: SpatialScalarV2,
    y: SpatialScalarV2,
}

impl SpatialOffsetV2 {
    /// Creates a raw placement offset.
    #[must_use]
    pub const fn new(x: SpatialScalarV2, y: SpatialScalarV2) -> Self {
        Self { x, y }
    }

    /// Returns the horizontal offset.
    #[must_use]
    pub const fn x(self) -> SpatialScalarV2 {
        self.x
    }

    /// Returns the vertical offset.
    #[must_use]
    pub const fn y(self) -> SpatialScalarV2 {
        self.y
    }
}

/// Raw two-dimensional affine coefficients in column-vector order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Affine2V2 {
    a: SpatialScalarV2,
    b: SpatialScalarV2,
    c: SpatialScalarV2,
    d: SpatialScalarV2,
    tx: SpatialScalarV2,
    ty: SpatialScalarV2,
}

impl Affine2V2 {
    /// Creates six raw affine coefficients.
    #[must_use]
    pub const fn new(
        a: SpatialScalarV2,
        b: SpatialScalarV2,
        c: SpatialScalarV2,
        d: SpatialScalarV2,
        tx: SpatialScalarV2,
        ty: SpatialScalarV2,
    ) -> Self {
        Self { a, b, c, d, tx, ty }
    }

    /// Returns the first linear coefficient.
    #[must_use]
    pub const fn a(self) -> SpatialScalarV2 {
        self.a
    }

    /// Returns the second linear coefficient.
    #[must_use]
    pub const fn b(self) -> SpatialScalarV2 {
        self.b
    }

    /// Returns the third linear coefficient.
    #[must_use]
    pub const fn c(self) -> SpatialScalarV2 {
        self.c
    }

    /// Returns the fourth linear coefficient.
    #[must_use]
    pub const fn d(self) -> SpatialScalarV2 {
        self.d
    }

    /// Returns the horizontal translation coefficient.
    #[must_use]
    pub const fn tx(self) -> SpatialScalarV2 {
        self.tx
    }

    /// Returns the vertical translation coefficient.
    #[must_use]
    pub const fn ty(self) -> SpatialScalarV2 {
        self.ty
    }
}

/// Raw local affine transform and its local origin.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialLocalTransformV2 {
    affine: Affine2V2,
    origin: SpatialPointV2,
}

impl SpatialLocalTransformV2 {
    /// Creates one raw local transform.
    #[must_use]
    pub const fn new(affine: Affine2V2, origin: SpatialPointV2) -> Self {
        Self { affine, origin }
    }

    /// Returns the raw affine coefficients.
    #[must_use]
    pub const fn affine(self) -> Affine2V2 {
        self.affine
    }

    /// Returns the local transform origin.
    #[must_use]
    pub const fn origin(self) -> SpatialPointV2 {
        self.origin
    }
}

/// Closed anchor component on one axis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialAnchorComponentV2 {
    /// Selects the near edge.
    Start,
    /// Selects the center.
    Center,
    /// Selects the far edge.
    End,
}

impl SpatialAnchorComponentV2 {
    /// Every anchor component in deterministic vocabulary order.
    pub const ALL: [Self; 3] = [Self::Start, Self::Center, Self::End];
}

/// Two-dimensional anchor components.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpatialAnchorV2 {
    horizontal: SpatialAnchorComponentV2,
    vertical: SpatialAnchorComponentV2,
}

impl SpatialAnchorV2 {
    /// Creates a two-dimensional anchor.
    #[must_use]
    pub const fn new(
        horizontal: SpatialAnchorComponentV2,
        vertical: SpatialAnchorComponentV2,
    ) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    /// Returns the horizontal anchor component.
    #[must_use]
    pub const fn horizontal(self) -> SpatialAnchorComponentV2 {
        self.horizontal
    }

    /// Returns the vertical anchor component.
    #[must_use]
    pub const fn vertical(self) -> SpatialAnchorComponentV2 {
        self.vertical
    }
}

/// Closed kind vocabulary for free-placement anchor targets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialAnchorTargetKindV2 {
    /// Targets the viewport.
    Viewport,
    /// Targets the spatial parent.
    Parent,
    /// Targets another spatial node.
    Node,
}

impl SpatialAnchorTargetKindV2 {
    /// Every anchor target kind in deterministic vocabulary order.
    pub const ALL: [Self; 3] = [Self::Viewport, Self::Parent, Self::Node];
}

/// Authored target for one free placement.
///
/// Payload enums intentionally have no fieldless `ALL` array.
///
/// ```compile_fail,E0599
/// use fenestra_ui_spatial::prototype::SpatialAnchorTargetV2;
///
/// let _ = SpatialAnchorTargetV2::ALL;
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialAnchorTargetV2 {
    /// Targets the present logical viewport.
    Viewport,
    /// Targets the node's spatial parent.
    Parent,
    /// Targets one supplied spatial node key.
    Node(SpatialNodeKeyV2),
}
