#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PaintP2Field {
    GradientStopLength,
    GradientStartX,
    GradientStartY,
    GradientEndX,
    GradientEndY,
    Offset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PaintP2LimitKind {
    GradientStopsPerBrush,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PaintP2GradientKind {
    CoincidentEndpoints,
    TooFewStops,
    FirstOffset,
    LastOffset,
    DecreasingOffset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PaintP2ErrorKind {
    LimitExceeded(PaintP2LimitKind),
    ScalarOutOfDomain,
    InvalidGradient(PaintP2GradientKind),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PaintP2Location {
    Brush {
        index: u32,
        field: PaintP2Field,
    },
    GradientStop {
        brush: u32,
        stop: u32,
        field: PaintP2Field,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PaintP2Error {
    kind: PaintP2ErrorKind,
    location: PaintP2Location,
    observed: Option<usize>,
    maximum: Option<usize>,
}

impl PaintP2Error {
    pub(super) const fn new(kind: PaintP2ErrorKind, location: PaintP2Location) -> Self {
        Self {
            kind,
            location,
            observed: None,
            maximum: None,
        }
    }

    pub(super) const fn limit(location: PaintP2Location, observed: usize, maximum: usize) -> Self {
        Self {
            kind: PaintP2ErrorKind::LimitExceeded(PaintP2LimitKind::GradientStopsPerBrush),
            location,
            observed: Some(observed),
            maximum: Some(maximum),
        }
    }

    pub(crate) const fn kind(self) -> PaintP2ErrorKind {
        self.kind
    }

    pub(crate) const fn location(self) -> PaintP2Location {
        self.location
    }

    pub(crate) const fn observed(self) -> Option<usize> {
        self.observed
    }

    pub(crate) const fn maximum(self) -> Option<usize> {
        self.maximum
    }
}
