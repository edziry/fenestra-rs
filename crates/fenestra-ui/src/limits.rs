/// Inclusive application resource bounds independent of conformance fixtures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    pub(crate) max_nodes: usize,
    pub(crate) max_depth: usize,
    pub(crate) max_pixels: usize,
}

impl Limits {
    /// Sets element count, root-inclusive element depth, and viewport pixel count.
    #[must_use]
    pub const fn new(max_nodes: usize, max_depth: usize, max_pixels: usize) -> Self {
        Self {
            max_nodes,
            max_depth,
            max_pixels,
        }
    }

    /// Returns the maximum number of elements in a view.
    #[must_use]
    pub const fn max_nodes(self) -> usize {
        self.max_nodes
    }

    /// Returns the maximum element depth, counting the view root as depth one.
    #[must_use]
    pub const fn max_depth(self) -> usize {
        self.max_depth
    }

    /// Returns the maximum pixel count in a viewport or rendered image.
    #[must_use]
    pub const fn max_pixels(self) -> usize {
        self.max_pixels
    }
}

impl Default for Limits {
    fn default() -> Self {
        Self::new(1024, 64, 4_194_304)
    }
}
