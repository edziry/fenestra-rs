//! Dense brush keys and trusted gradient-stop ranges.

use std::ops::Range;

use super::make_resolve_error;
use super::validated_shapes::{ShapeLocalBoundsInput, ValidatedShapesProof};
use crate::brush::{SpatialBrushContentV2, SpatialGradientStopV2};
use crate::content_diagnostic::{SpatialKeyedContentTableV2, SpatialPayloadTableV2};
use crate::content_error::SpatialContentErrorKindV2;
use crate::error::SpatialErrorLocationV2;
use crate::geometry_field::SpatialBrushFieldV2;
use crate::resolve_error::{SpatialResolveErrorKindV2, SpatialResolveErrorV2};

struct GradientRange {
    brush: u32,
    stops: Range<usize>,
}

pub(super) struct BrushStructureProof<'a> {
    shapes: ValidatedShapesProof<'a>,
    gradient_ranges: Vec<GradientRange>,
}

impl<'a> BrushStructureProof<'a> {
    pub(super) fn input(&self) -> crate::aggregate_input::SpatialInputV2<'a> {
        self.shapes.input()
    }

    pub(super) fn limits(&self) -> crate::limits::SpatialLimitsV2 {
        self.shapes.limits()
    }

    pub(super) fn validated_paths(&self) -> &[crate::geometry_kernel::ValidatedPathK1<'a>] {
        self.shapes.validated_paths()
    }

    pub(super) fn shape_local_bounds_inputs(
        &self,
    ) -> impl Iterator<Item = ShapeLocalBoundsInput<'a>> + '_ {
        self.shapes.shape_local_bounds_inputs()
    }

    pub(super) fn gradient_stops(
        &self,
        gradient: usize,
        brush: u32,
    ) -> &'a [SpatialGradientStopV2] {
        let range = self
            .gradient_ranges
            .get(gradient)
            .expect("phase seven supplied a trusted gradient ordinal");
        assert_eq!(
            range.brush, brush,
            "trusted gradient ranges remain aligned with brush order"
        );
        &self.input().resources().gradient_stops()[range.stops.clone()]
    }
}

pub(super) fn prepare_brush_structure<'a>(
    shapes: ValidatedShapesProof<'a>,
) -> Result<BrushStructureProof<'a>, SpatialResolveErrorV2> {
    let resources = shapes.input().resources();
    let brushes = resources.brushes();
    let stop_count = resources.gradient_stops().len() as u128;

    for (index, brush) in brushes.iter().copied().enumerate() {
        let ordinal = trusted_brush_ordinal(index);
        if brush.key().get() != ordinal {
            return Err(content_error(
                SpatialContentErrorKindV2::NonDenseKey(SpatialKeyedContentTableV2::Brush),
                brush_location(ordinal, SpatialBrushFieldV2::Key),
            ));
        }
    }

    let mut cursor = 0_u128;
    let mut gradient_ranges = Vec::new();
    for (index, brush) in brushes.iter().copied().enumerate() {
        let ordinal = trusted_brush_ordinal(index);
        match brush.content() {
            SpatialBrushContentV2::Solid { .. } => {}
            SpatialBrushContentV2::LinearGradient {
                stop_start,
                stop_length,
                ..
            } => {
                let end = validate_gradient_stop_range(
                    ordinal,
                    cursor,
                    stop_start,
                    stop_length,
                    stop_count,
                )?;
                gradient_ranges.push(GradientRange {
                    brush: ordinal,
                    stops: trusted_stop_index(cursor)..trusted_stop_index(end),
                });
                cursor = end;
            }
        }
    }

    if cursor != stop_count {
        return Err(invalid_gradient_range(SpatialErrorLocationV2::Input));
    }

    Ok(BrushStructureProof {
        shapes,
        gradient_ranges,
    })
}

pub(super) fn validate_gradient_stop_range(
    brush: u32,
    cursor: u128,
    stop_start: u32,
    stop_length: u32,
    stop_count: u128,
) -> Result<u128, SpatialResolveErrorV2> {
    let start = u128::from(stop_start);
    if start != cursor {
        return Err(invalid_gradient_range(brush_location(
            brush,
            SpatialBrushFieldV2::GradientStopStart,
        )));
    }

    let end = start + u128::from(stop_length);
    if end > stop_count {
        return Err(invalid_gradient_range(brush_location(
            brush,
            SpatialBrushFieldV2::GradientStopLength,
        )));
    }

    Ok(end)
}

fn trusted_brush_ordinal(index: usize) -> u32 {
    u32::try_from(index).expect("phase one validated the brush row capacity")
}

fn trusted_stop_index(index: u128) -> usize {
    usize::try_from(index).expect("a trusted gradient range fits the payload table")
}

const fn brush_location(index: u32, field: SpatialBrushFieldV2) -> SpatialErrorLocationV2 {
    SpatialErrorLocationV2::Brush { index, field }
}

fn invalid_gradient_range(location: SpatialErrorLocationV2) -> SpatialResolveErrorV2 {
    content_error(
        SpatialContentErrorKindV2::InvalidRange(SpatialPayloadTableV2::GradientStop),
        location,
    )
}

fn content_error(
    kind: SpatialContentErrorKindV2,
    location: SpatialErrorLocationV2,
) -> SpatialResolveErrorV2 {
    make_resolve_error(SpatialResolveErrorKindV2::Content(kind), location)
}

#[cfg(test)]
impl BrushStructureProof<'_> {
    pub(super) fn gradient_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.gradient_ranges
            .iter()
            .map(|range| {
                (
                    range.brush,
                    range.stops.start as u128,
                    range.stops.end as u128,
                )
            })
            .collect()
    }

    pub(super) fn validated_shape_facts(
        &self,
    ) -> Vec<(u32, crate::shape::SpatialShapeKindV2, usize)> {
        self.shapes.validated_shape_facts()
    }

    pub(super) fn polygon_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.shapes.polygon_range_facts()
    }

    pub(super) fn validated_path_facts(&self) -> Vec<(u32, usize, usize)> {
        self.shapes.validated_path_facts()
    }

    pub(super) fn subpath_total(&self) -> usize {
        self.shapes.subpath_total()
    }

    pub(super) fn path_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.shapes.path_range_facts()
    }

    pub(super) fn prepared_island_facts(&self) -> Vec<(u32, Vec<u32>)> {
        self.shapes.prepared_island_facts()
    }
}
