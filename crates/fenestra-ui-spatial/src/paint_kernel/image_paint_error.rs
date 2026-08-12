use crate::vocabulary::{SpatialAxisV2, SpatialExtentV2};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PaintP5Field {
    SourceX,
    SourceY,
    SourceWidth,
    SourceHeight,
    DestinationX,
    DestinationY,
    DestinationWidth,
    DestinationHeight,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PaintP5ImageKind {
    EmptySource,
    SourceOutOfBounds,
    NegativeDestinationExtent(SpatialExtentV2),
    EmptyDestination,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PaintP5ErrorKind {
    ScalarOutOfDomain,
    InvalidImage(PaintP5ImageKind),
    LocalBoundsOutOfDomain(SpatialAxisV2),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PaintP5Location {
    Paint { index: u32, field: PaintP5Field },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PaintP5Error {
    kind: PaintP5ErrorKind,
    location: PaintP5Location,
}

impl PaintP5Error {
    pub(super) const fn new(kind: PaintP5ErrorKind, index: u32, field: PaintP5Field) -> Self {
        Self {
            kind,
            location: PaintP5Location::Paint { index, field },
        }
    }

    pub(crate) const fn kind(self) -> PaintP5ErrorKind {
        self.kind
    }

    pub(crate) const fn location(self) -> PaintP5Location {
        self.location
    }
}
