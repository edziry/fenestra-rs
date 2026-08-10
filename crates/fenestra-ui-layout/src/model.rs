use crate::vocabulary::LayoutAxisV1;

/// Dense pass-local key for one version-1 layout node.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LayoutNodeKeyV1(u32);

impl LayoutNodeKeyV1 {
    /// Creates a pass-local key.
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

/// Raw minimum, preferred, and maximum values for one logical extent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayoutDimensionV1 {
    minimum: i32,
    preferred: i32,
    maximum: i32,
}

impl LayoutDimensionV1 {
    /// Creates one raw dimension constraint.
    #[must_use]
    pub const fn new(minimum: i32, preferred: i32, maximum: i32) -> Self {
        Self {
            minimum,
            preferred,
            maximum,
        }
    }

    /// Returns the inclusive minimum.
    #[must_use]
    pub const fn minimum(self) -> i32 {
        self.minimum
    }

    /// Returns the authored preference.
    #[must_use]
    pub const fn preferred(self) -> i32 {
        self.preferred
    }

    /// Returns the inclusive maximum.
    #[must_use]
    pub const fn maximum(self) -> i32 {
        self.maximum
    }

    pub(crate) const fn resolved(self) -> i32 {
        if self.preferred < self.minimum {
            self.minimum
        } else if self.preferred > self.maximum {
            self.maximum
        } else {
            self.preferred
        }
    }
}

/// Raw border-box padding for one layout node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayoutPaddingV1 {
    left: i32,
    right: i32,
    top: i32,
    bottom: i32,
}

impl LayoutPaddingV1 {
    /// Creates explicit padding on every side.
    #[must_use]
    pub const fn new(left: i32, right: i32, top: i32, bottom: i32) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }

    /// Returns the left padding.
    #[must_use]
    pub const fn left(self) -> i32 {
        self.left
    }

    /// Returns the right padding.
    #[must_use]
    pub const fn right(self) -> i32 {
        self.right
    }

    /// Returns the top padding.
    #[must_use]
    pub const fn top(self) -> i32 {
        self.top
    }

    /// Returns the bottom padding.
    #[must_use]
    pub const fn bottom(self) -> i32 {
        self.bottom
    }
}

/// Raw version-1 stack style for one node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayoutStyleV1 {
    axis: LayoutAxisV1,
    width: LayoutDimensionV1,
    height: LayoutDimensionV1,
    padding: LayoutPaddingV1,
    gap: i32,
}

impl LayoutStyleV1 {
    /// Creates a complete raw stack style.
    #[must_use]
    pub const fn new(
        axis: LayoutAxisV1,
        width: LayoutDimensionV1,
        height: LayoutDimensionV1,
        padding: LayoutPaddingV1,
        gap: i32,
    ) -> Self {
        Self {
            axis,
            width,
            height,
            padding,
            gap,
        }
    }

    /// Returns the child stack direction.
    #[must_use]
    pub const fn axis(self) -> LayoutAxisV1 {
        self.axis
    }

    /// Returns the width constraint.
    #[must_use]
    pub const fn width(self) -> LayoutDimensionV1 {
        self.width
    }

    /// Returns the height constraint.
    #[must_use]
    pub const fn height(self) -> LayoutDimensionV1 {
        self.height
    }

    /// Returns the border-box padding.
    #[must_use]
    pub const fn padding(self) -> LayoutPaddingV1 {
        self.padding
    }

    /// Returns the inter-child gap.
    #[must_use]
    pub const fn gap(self) -> i32 {
        self.gap
    }
}

/// One raw node in authored preorder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayoutNodeV1 {
    key: LayoutNodeKeyV1,
    parent: Option<LayoutNodeKeyV1>,
    style: LayoutStyleV1,
}

impl LayoutNodeV1 {
    /// Creates one raw authored-preorder node.
    #[must_use]
    pub const fn new(
        key: LayoutNodeKeyV1,
        parent: Option<LayoutNodeKeyV1>,
        style: LayoutStyleV1,
    ) -> Self {
        Self { key, parent, style }
    }

    /// Returns the pass-local key.
    #[must_use]
    pub const fn key(self) -> LayoutNodeKeyV1 {
        self.key
    }

    /// Returns the parent key, or `None` for an authored root.
    #[must_use]
    pub const fn parent(self) -> Option<LayoutNodeKeyV1> {
        self.parent
    }

    /// Returns the raw stack style.
    #[must_use]
    pub const fn style(self) -> LayoutStyleV1 {
        self.style
    }
}

/// Present logical viewport metadata for one computation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayoutViewportV1 {
    width: i32,
    height: i32,
}

impl LayoutViewportV1 {
    /// Creates a raw logical viewport, including possible zero extents.
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

/// Borrowed raw input for one layout computation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayoutInputV1<'a> {
    viewport: LayoutViewportV1,
    nodes: &'a [LayoutNodeV1],
}

impl<'a> LayoutInputV1<'a> {
    /// Creates a borrowed raw layout input.
    #[must_use]
    pub const fn new(viewport: LayoutViewportV1, nodes: &'a [LayoutNodeV1]) -> Self {
        Self { viewport, nodes }
    }

    /// Returns the present logical viewport.
    #[must_use]
    pub const fn viewport(self) -> LayoutViewportV1 {
        self.viewport
    }

    /// Returns nodes in authored preorder.
    #[must_use]
    pub const fn nodes(self) -> &'a [LayoutNodeV1] {
        self.nodes
    }
}

/// One absolute logical rectangle returned by a layout engine.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayoutRectV1 {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl LayoutRectV1 {
    /// Creates one raw absolute logical rectangle.
    #[must_use]
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns the horizontal origin.
    #[must_use]
    pub const fn x(self) -> i32 {
        self.x
    }

    /// Returns the vertical origin.
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
}

/// One key-associated layout result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayoutRecordV1 {
    key: LayoutNodeKeyV1,
    bounds: LayoutRectV1,
}

impl LayoutRecordV1 {
    /// Creates one raw output record.
    #[must_use]
    pub const fn new(key: LayoutNodeKeyV1, bounds: LayoutRectV1) -> Self {
        Self { key, bounds }
    }

    /// Returns the pass-local input key.
    #[must_use]
    pub const fn key(self) -> LayoutNodeKeyV1 {
        self.key
    }

    /// Returns the absolute border-box bounds.
    #[must_use]
    pub const fn bounds(self) -> LayoutRectV1 {
        self.bounds
    }
}

/// Owned raw output returned by one layout engine call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayoutOutputV1 {
    records: Vec<LayoutRecordV1>,
}

impl LayoutOutputV1 {
    /// Creates an owned raw output for boundary validation.
    #[must_use]
    pub const fn new(records: Vec<LayoutRecordV1>) -> Self {
        Self { records }
    }

    /// Returns records in engine-provided order.
    #[must_use]
    pub fn records(&self) -> &[LayoutRecordV1] {
        &self.records
    }
}
