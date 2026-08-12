use crate::ids::SpatialImageSymbolV2;
use crate::source::SourceSpan;

use super::SpatialFieldV2;

/// Program-local decoded image declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpatialImageDeclarationV2 {
    symbol: SpatialFieldV2<SpatialImageSymbolV2>,
    width: SpatialFieldV2<u32>,
    height: SpatialFieldV2<u32>,
    stride: SpatialFieldV2<u32>,
    bytes: Box<[u8]>,
    span: SourceSpan,
}

impl SpatialImageDeclarationV2 {
    /// Creates a program-local image declaration.
    #[must_use]
    pub fn new(
        symbol: SpatialFieldV2<SpatialImageSymbolV2>,
        width: SpatialFieldV2<u32>,
        height: SpatialFieldV2<u32>,
        stride: SpatialFieldV2<u32>,
        bytes: Box<[u8]>,
        span: SourceSpan,
    ) -> Self {
        Self {
            symbol,
            width,
            height,
            stride,
            bytes,
            span,
        }
    }

    /// Returns the program-local image symbol.
    #[must_use]
    pub const fn symbol(&self) -> SpatialFieldV2<SpatialImageSymbolV2> {
        self.symbol
    }

    /// Returns the image width in pixels.
    #[must_use]
    pub const fn width(&self) -> SpatialFieldV2<u32> {
        self.width
    }

    /// Returns the image height in pixels.
    #[must_use]
    pub const fn height(&self) -> SpatialFieldV2<u32> {
        self.height
    }

    /// Returns the image row stride in bytes.
    #[must_use]
    pub const fn stride(&self) -> SpatialFieldV2<u32> {
        self.stride
    }

    /// Returns the decoded image bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the image declaration span.
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        self.span
    }
}
