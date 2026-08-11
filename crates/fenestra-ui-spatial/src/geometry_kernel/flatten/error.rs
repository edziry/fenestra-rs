use super::super::error::{GeometryK1Field, GeometryK1Location};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GeometryK2LimitKind {
    FlattenedSegmentsPerPath,
    FlattenedSegmentsTotal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GeometryK2ErrorKind {
    LimitExceeded(GeometryK2LimitKind),
    NonFlatAtMaximumDepth,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GeometryK2Error {
    kind: GeometryK2ErrorKind,
    location: GeometryK1Location,
    observed: Option<usize>,
    maximum: Option<usize>,
}

impl GeometryK2Error {
    pub(super) const fn nonflat(path: u32, source_verb: u32) -> Self {
        Self {
            kind: GeometryK2ErrorKind::NonFlatAtMaximumDepth,
            location: source_location(path, source_verb),
            observed: None,
            maximum: None,
        }
    }

    pub(super) const fn limit(
        kind: GeometryK2LimitKind,
        path: u32,
        source_verb: u32,
        observed: usize,
        maximum: usize,
    ) -> Self {
        Self {
            kind: GeometryK2ErrorKind::LimitExceeded(kind),
            location: source_location(path, source_verb),
            observed: Some(observed),
            maximum: Some(maximum),
        }
    }

    pub(crate) const fn kind(self) -> GeometryK2ErrorKind {
        self.kind
    }

    pub(crate) const fn location(self) -> GeometryK1Location {
        self.location
    }

    pub(crate) const fn observed(self) -> Option<usize> {
        self.observed
    }

    pub(crate) const fn maximum(self) -> Option<usize> {
        self.maximum
    }
}

const fn source_location(path: u32, verb: u32) -> GeometryK1Location {
    GeometryK1Location::PathVerb {
        path,
        verb,
        field: GeometryK1Field::Kind,
    }
}
