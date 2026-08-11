//! Dense image keys and record-major Paint P4 validation.

use super::make_resolve_error;
use super::paint_p4_mapping::map_image_p4_error;
use super::prepared_brushes::PreparedBrushesProof;
use super::validated_shapes::ShapeLocalBoundsInput;
use crate::content_diagnostic::SpatialKeyedContentTableV2;
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::SpatialImageFieldV2;
use crate::limits::SpatialLimitKindV2;
use crate::paint_kernel::{ValidatedImageP4, prepare_image_p4};
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

pub(super) struct ValidatedImagesProof<'a> {
    brushes: PreparedBrushesProof<'a>,
    images: Vec<ValidatedImageP4<'a>>,
    accepted_pixels: usize,
}

impl<'a> ValidatedImagesProof<'a> {
    pub(super) fn input(&self) -> crate::aggregate_input::SpatialInputV2<'a> {
        self.brushes.input()
    }

    pub(super) fn limits(&self) -> crate::limits::SpatialLimitsV2 {
        self.brushes.limits()
    }

    pub(super) fn validated_paths(&self) -> &[crate::geometry_kernel::ValidatedPathK1<'a>] {
        self.brushes.validated_paths()
    }

    pub(super) fn shape_local_bounds_inputs(
        &self,
    ) -> impl Iterator<Item = ShapeLocalBoundsInput<'a>> + '_ {
        self.brushes.shape_local_bounds_inputs()
    }

    pub(super) fn validated_image(&self, index: u32) -> Option<ValidatedImageP4<'a>> {
        self.images.get(index as usize).copied()
    }
}

pub(super) fn prepare_validated_images<'a>(
    brushes: PreparedBrushesProof<'a>,
) -> Result<ValidatedImagesProof<'a>, SpatialResolveErrorV2> {
    let input = brushes.input();
    let images: &'a [crate::image::SpatialImageV2] = input.resources().images();

    for (index, image) in images.iter().enumerate() {
        let ordinal = trusted_image_ordinal(index);
        if image.key().get() != ordinal {
            return Err(content_error(
                SpatialContentErrorKindV2::NonDenseKey(SpatialKeyedContentTableV2::Image),
                image_location(ordinal, SpatialImageFieldV2::Key),
            ));
        }
    }

    let limits = brushes.limits();
    let maximum_edge = limits.limit(SpatialLimitKindV2::ImageEdge);
    let maximum_pixels = limits.limit(SpatialLimitKindV2::ImagePixelsTotal);
    let mut accepted_pixels = 0_usize;
    let mut validated = Vec::with_capacity(images.len());
    for image in images {
        validated.push(
            prepare_image_p4(image, &mut accepted_pixels, maximum_edge, maximum_pixels)
                .map_err(map_image_p4_error)?,
        );
    }

    Ok(ValidatedImagesProof {
        brushes,
        images: validated,
        accepted_pixels,
    })
}

fn trusted_image_ordinal(index: usize) -> u32 {
    u32::try_from(index).expect("phase one validated the image row capacity")
}

const fn image_location(index: u32, field: SpatialImageFieldV2) -> SpatialErrorLocationV2 {
    SpatialErrorLocationV2::Image { index, field }
}

fn content_error(
    kind: SpatialContentErrorKindV2,
    location: SpatialErrorLocationV2,
) -> SpatialResolveErrorV2 {
    make_resolve_error(SpatialResolveErrorKindV2::Content(kind), location)
}

#[cfg(test)]
impl ValidatedImagesProof<'_> {
    pub(super) fn validated_image_facts(&self) -> Vec<(u32, u32, u32, u32, Vec<u8>)> {
        self.images
            .iter()
            .enumerate()
            .map(|(index, image)| {
                let (width, height, stride, bytes) = image.facts();
                (trusted_image_ordinal(index), width, height, stride, bytes)
            })
            .collect()
    }

    pub(super) fn accepted_pixel_total(&self) -> u128 {
        self.accepted_pixels as u128
    }

    pub(super) fn prepared_brush_facts(
        &self,
    ) -> Vec<(u32, crate::brush::SpatialBrushKindV2, usize)> {
        self.brushes.prepared_brush_facts()
    }

    pub(super) fn gradient_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.brushes.gradient_range_facts()
    }

    pub(super) fn validated_shape_facts(
        &self,
    ) -> Vec<(u32, crate::shape::SpatialShapeKindV2, usize)> {
        self.brushes.validated_shape_facts()
    }

    pub(super) fn polygon_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.brushes.polygon_range_facts()
    }

    pub(super) fn validated_path_facts(&self) -> Vec<(u32, usize, usize)> {
        self.brushes.validated_path_facts()
    }

    pub(super) fn subpath_total(&self) -> usize {
        self.brushes.subpath_total()
    }

    pub(super) fn path_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.brushes.path_range_facts()
    }

    pub(super) fn prepared_island_facts(&self) -> Vec<(u32, Vec<u32>)> {
        self.brushes.prepared_island_facts()
    }
}
