#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GeometryK1Field {
    Kind,
    VerbLength,
    ToX,
    ToY,
    ControlX,
    ControlY,
    Control1X,
    Control1Y,
    Control2X,
    Control2Y,
    RectX,
    RectY,
    RectWidth,
    RectHeight,
    CircleCenterX,
    CircleCenterY,
    CircleRadius,
    PolygonPointLength,
    X,
    Y,
    StrokeWidth,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GeometryK1LimitKind {
    PathSubpathsTotal,
    PolygonPointsPerShape,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GeometryK1PathGrammarKind {
    Empty,
    FirstNotMove,
    EmptySubpath,
    DrawingWithoutSubpath,
    CloseWithoutSegment,
    TrailingMove,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GeometryK1ShapeKind {
    NegativeExtent,
    NegativeRadius,
    PolygonTooShort,
    PolygonRepeatedFirst,
    PolygonAdjacentEqual,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GeometryK1StrokeKind {
    NegativeWidth,
    ZeroWidth,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GeometryK1ErrorKind {
    ScalarOutOfDomain,
    InvalidPathGrammar(GeometryK1PathGrammarKind),
    InvalidShape(GeometryK1ShapeKind),
    InvalidStroke(GeometryK1StrokeKind),
    LimitExceeded(GeometryK1LimitKind),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GeometryK1Location {
    Path {
        index: u32,
        field: GeometryK1Field,
    },
    PathVerb {
        path: u32,
        verb: u32,
        field: GeometryK1Field,
    },
    Shape {
        index: u32,
        field: GeometryK1Field,
    },
    PolygonPoint {
        shape: u32,
        point: u32,
        field: GeometryK1Field,
    },
    Paint {
        index: u32,
        field: GeometryK1Field,
    },
    Hit {
        index: u32,
        field: GeometryK1Field,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GeometryK1Error {
    kind: GeometryK1ErrorKind,
    location: GeometryK1Location,
    observed: Option<u128>,
    maximum: Option<u128>,
}

impl GeometryK1Error {
    pub(crate) const fn new(kind: GeometryK1ErrorKind, location: GeometryK1Location) -> Self {
        Self {
            kind,
            location,
            observed: None,
            maximum: None,
        }
    }

    pub(crate) const fn limit(
        kind: GeometryK1LimitKind,
        location: GeometryK1Location,
        observed: u128,
        maximum: u128,
    ) -> Self {
        Self {
            kind: GeometryK1ErrorKind::LimitExceeded(kind),
            location,
            observed: Some(observed),
            maximum: Some(maximum),
        }
    }

    pub(crate) const fn kind(self) -> GeometryK1ErrorKind {
        self.kind
    }

    pub(crate) const fn location(self) -> GeometryK1Location {
        self.location
    }

    pub(crate) const fn observed(self) -> Option<u128> {
        self.observed
    }

    pub(crate) const fn maximum(self) -> Option<u128> {
        self.maximum
    }
}
