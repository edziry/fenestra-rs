#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PaintP4Channel {
    R,
    G,
    B,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PaintP4Field {
    Width,
    Height,
    Stride,
    ByteLength,
    Pixel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PaintP4LimitKind {
    ImageEdge,
    ImagePixelsTotal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PaintP4ImageKind {
    ZeroExtent,
    StrideMismatch,
    LengthMismatch,
    InvalidPremultipliedPixel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PaintP4ErrorKind {
    LimitExceeded(PaintP4LimitKind),
    InvalidImage(PaintP4ImageKind),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PaintP4Location {
    Image {
        index: u32,
        field: PaintP4Field,
    },
    ImagePixel {
        image: u32,
        pixel: u128,
        channel: PaintP4Channel,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PaintP4Error {
    kind: PaintP4ErrorKind,
    location: PaintP4Location,
    observed: Option<u128>,
    maximum: Option<u128>,
}

impl PaintP4Error {
    pub(super) const fn new(kind: PaintP4ErrorKind, location: PaintP4Location) -> Self {
        Self {
            kind,
            location,
            observed: None,
            maximum: None,
        }
    }

    pub(super) const fn limit(
        limit: PaintP4LimitKind,
        location: PaintP4Location,
        observed: u128,
        maximum: u128,
    ) -> Self {
        Self {
            kind: PaintP4ErrorKind::LimitExceeded(limit),
            location,
            observed: Some(observed),
            maximum: Some(maximum),
        }
    }

    pub(crate) const fn kind(self) -> PaintP4ErrorKind {
        self.kind
    }

    pub(crate) const fn location(self) -> PaintP4Location {
        self.location
    }

    pub(crate) const fn observed(self) -> Option<u128> {
        self.observed
    }

    pub(crate) const fn maximum(self) -> Option<u128> {
        self.maximum
    }
}

#[cfg(test)]
pub(crate) const fn test_p4_pixel_error(
    image: u32,
    pixel: u128,
    channel: PaintP4Channel,
) -> PaintP4Error {
    PaintP4Error::new(
        PaintP4ErrorKind::InvalidImage(PaintP4ImageKind::InvalidPremultipliedPixel),
        PaintP4Location::ImagePixel {
            image,
            pixel,
            channel,
        },
    )
}
