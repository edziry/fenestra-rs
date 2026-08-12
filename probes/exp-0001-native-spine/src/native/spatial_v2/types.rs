use fenestra_ui_runtime::prototype::{RuntimeGeneration, SubmissionId};
use fenestra_ui_spatial::prototype::SpatialViewportV2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SpatialPhysicalExtentV2 {
    width: u32,
    height: u32,
}

impl SpatialPhysicalExtentV2 {
    pub(crate) const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub(crate) const fn width(self) -> u32 {
        self.width
    }

    pub(crate) const fn height(self) -> u32 {
        self.height
    }

    pub(crate) const fn is_zero(self) -> bool {
        self.width == 0 || self.height == 0
    }
}

#[derive(Clone, Copy)]
pub(crate) struct SpatialSurfaceTupleV2 {
    physical: SpatialPhysicalExtentV2,
    logical: SpatialViewportV2,
}

impl SpatialSurfaceTupleV2 {
    pub(crate) const fn new(physical: SpatialPhysicalExtentV2, logical: SpatialViewportV2) -> Self {
        Self { physical, logical }
    }

    pub(crate) const fn physical(self) -> SpatialPhysicalExtentV2 {
        self.physical
    }

    pub(crate) const fn logical(self) -> SpatialViewportV2 {
        self.logical
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SpatialPresentationLimitKindV2 {
    ReferencePixels,
    PhysicalWidth,
    PhysicalHeight,
    PhysicalPixels,
    PhysicalBytes,
}

#[derive(Clone, Copy)]
pub(crate) struct SpatialPresentationLimitsV2 {
    reference_pixels: usize,
    physical_width: u32,
    physical_height: u32,
    physical_pixels: usize,
    physical_bytes: usize,
}

impl SpatialPresentationLimitsV2 {
    pub(crate) const fn new(
        reference_pixels: usize,
        physical_width: u32,
        physical_height: u32,
        physical_pixels: usize,
        physical_bytes: usize,
    ) -> Self {
        Self {
            reference_pixels,
            physical_width,
            physical_height,
            physical_pixels,
            physical_bytes,
        }
    }

    pub(crate) const fn reference_pixels(self) -> usize {
        self.reference_pixels
    }

    pub(crate) const fn physical_width(self) -> u32 {
        self.physical_width
    }

    pub(crate) const fn physical_height(self) -> u32 {
        self.physical_height
    }

    pub(crate) const fn physical_pixels(self) -> usize {
        self.physical_pixels
    }

    pub(crate) const fn physical_bytes(self) -> usize {
        self.physical_bytes
    }
}

#[derive(Clone, Copy)]
pub(crate) struct SpatialRasterInputV2<'a> {
    width: u32,
    height: u32,
    stride: u64,
    bytes: &'a [u8],
}

impl<'a> SpatialRasterInputV2<'a> {
    pub(crate) const fn new(width: u32, height: u32, stride: u64, bytes: &'a [u8]) -> Self {
        Self {
            width,
            height,
            stride,
            bytes,
        }
    }

    pub(crate) const fn width(self) -> u32 {
        self.width
    }

    pub(crate) const fn height(self) -> u32 {
        self.height
    }

    pub(crate) const fn stride(self) -> u64 {
        self.stride
    }

    pub(crate) const fn bytes(self) -> &'a [u8] {
        self.bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SpatialPresentErrorKindV2 {
    ReferenceRaster,
    RasterMetadata,
    ZeroLogicalRaster,
    ViewportMismatch,
    LimitExceeded(SpatialPresentationLimitKindV2),
    Allocation,
    Presenter,
    PrePresent,
    Scheduler,
    Invariant,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SpatialPresentErrorV2 {
    kind: SpatialPresentErrorKindV2,
    accepted_submission: Option<SubmissionId>,
}

impl SpatialPresentErrorV2 {
    pub(crate) const fn new(
        kind: SpatialPresentErrorKindV2,
        accepted_submission: Option<SubmissionId>,
    ) -> Self {
        Self {
            kind,
            accepted_submission,
        }
    }

    pub(crate) const fn kind(self) -> SpatialPresentErrorKindV2 {
        self.kind
    }

    pub(crate) const fn accepted_submission(self) -> Option<SubmissionId> {
        self.accepted_submission
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SpatialPresentationReceiptV2 {
    generation: RuntimeGeneration,
    digest: u64,
}

impl SpatialPresentationReceiptV2 {
    pub(crate) const fn new(generation: RuntimeGeneration, digest: u64) -> Self {
        Self { generation, digest }
    }

    pub(crate) const fn generation(self) -> RuntimeGeneration {
        self.generation
    }

    pub(crate) const fn digest(self) -> u64 {
        self.digest
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SpatialPresentationOutcomeV2 {
    Suspended,
    Completed(SpatialPresentationReceiptV2),
}

impl SpatialPresentationOutcomeV2 {
    pub(crate) const fn completed(self) -> Option<SpatialPresentationReceiptV2> {
        match self {
            Self::Suspended => None,
            Self::Completed(receipt) => Some(receipt),
        }
    }
}

pub(crate) struct StagedSpatialPixelsV2 {
    physical: SpatialPhysicalExtentV2,
    pixels: Vec<u32>,
    digest: u64,
}

impl StagedSpatialPixelsV2 {
    pub(crate) fn new(physical: SpatialPhysicalExtentV2, pixels: Vec<u32>, digest: u64) -> Self {
        Self {
            physical,
            pixels,
            digest,
        }
    }

    pub(crate) const fn physical(&self) -> SpatialPhysicalExtentV2 {
        self.physical
    }

    pub(crate) fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    pub(crate) const fn digest(&self) -> u64 {
        self.digest
    }
}
