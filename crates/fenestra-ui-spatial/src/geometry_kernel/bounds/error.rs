use crate::vocabulary::SpatialAxisV2;

use super::super::error::GeometryK1Location;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GeometryK3ErrorKind {
    LocalBoundsOutOfDomain(SpatialAxisV2),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GeometryK3Error {
    kind: GeometryK3ErrorKind,
    location: GeometryK1Location,
}

impl GeometryK3Error {
    pub(super) const fn new(axis: SpatialAxisV2, location: GeometryK1Location) -> Self {
        Self {
            kind: GeometryK3ErrorKind::LocalBoundsOutOfDomain(axis),
            location,
        }
    }

    pub(crate) const fn kind(self) -> GeometryK3ErrorKind {
        self.kind
    }

    pub(crate) const fn location(self) -> GeometryK1Location {
        self.location
    }
}
