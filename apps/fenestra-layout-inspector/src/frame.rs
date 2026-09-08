use fenestra_ui_spatial::prototype::SpatialViewportV2;

/// Deterministic observation of one application frame.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InspectorFrame {
    pub(super) generation: u64,
    pub(super) viewport: SpatialViewportV2,
    pub(super) node_count: usize,
    pub(super) keyed_keys: Box<[u64]>,
    pub(super) image_count: usize,
    pub(super) paint_count: usize,
    pub(super) hit_count: usize,
    pub(super) semantic_count: usize,
    pub(super) raster_bytes: usize,
    pub(super) has_hover: bool,
    pub(super) has_selection: bool,
}

/// Bounded RGBA8 reference pixels for native presentation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InspectorRaster {
    pub(super) viewport: SpatialViewportV2,
    pub(super) bytes: Box<[u8]>,
}

impl InspectorRaster {
    /// Returns the logical viewport represented by these pixels.
    #[must_use]
    pub const fn viewport(&self) -> SpatialViewportV2 {
        self.viewport
    }

    /// Returns premultiplied RGBA8 pixels in row-major order.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl InspectorFrame {
    /// Returns the committed runtime generation.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// Returns the logical viewport used by this frame.
    #[must_use]
    pub const fn viewport(&self) -> SpatialViewportV2 {
        self.viewport
    }

    /// Returns the number of live logical nodes.
    #[must_use]
    pub const fn node_count(&self) -> usize {
        self.node_count
    }

    /// Returns keyed tile keys in committed order.
    #[must_use]
    pub fn keyed_keys(&self) -> &[u64] {
        &self.keyed_keys
    }

    /// Returns the number of authored image resources.
    #[must_use]
    pub const fn image_count(&self) -> usize {
        self.image_count
    }

    /// Returns the number of resolved paint items.
    #[must_use]
    pub const fn paint_count(&self) -> usize {
        self.paint_count
    }

    /// Returns the number of resolved hit items.
    #[must_use]
    pub const fn hit_count(&self) -> usize {
        self.hit_count
    }

    /// Returns the number of resolved semantic items.
    #[must_use]
    pub const fn semantic_count(&self) -> usize {
        self.semantic_count
    }

    /// Returns the size of the reference raster in bytes.
    #[must_use]
    pub const fn raster_bytes(&self) -> usize {
        self.raster_bytes
    }

    /// Reports whether a pointer hit is currently hovered.
    #[must_use]
    pub const fn has_hover(&self) -> bool {
        self.has_hover
    }

    /// Reports whether a node is selected.
    #[must_use]
    pub const fn has_selection(&self) -> bool {
        self.has_selection
    }
}
