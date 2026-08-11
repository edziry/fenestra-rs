//! Record-major Paint P2 brush preparation.

use super::brush_structure::BrushStructureProof;
use super::paint_p2_mapping::map_paint_p2_error;
use super::validated_shapes::ShapeLocalBoundsInput;
use crate::brush::{SpatialBrushContentV2, SpatialRgba8V2};
use crate::limits::SpatialLimitKindV2;
use crate::paint_kernel::{PreparedGradientP2, prepare_gradient_p2, prepare_solid_p2};
use crate::resolve_error::SpatialResolveErrorV2;

enum PreparedBrush {
    Solid(SpatialRgba8V2),
    LinearGradient(PreparedGradientP2),
}

pub(super) struct PreparedBrushesProof<'a> {
    structure: BrushStructureProof<'a>,
    brushes: Vec<PreparedBrush>,
}

impl<'a> PreparedBrushesProof<'a> {
    pub(super) fn input(&self) -> crate::aggregate_input::SpatialInputV2<'a> {
        self.structure.input()
    }

    pub(super) fn limits(&self) -> crate::limits::SpatialLimitsV2 {
        self.structure.limits()
    }

    pub(super) fn dependency_islands(
        &self,
    ) -> impl Iterator<Item = super::islands::preflight::DependencyIslandInput<'_>> + '_ {
        self.structure.dependency_islands()
    }

    pub(super) fn validated_paths(&self) -> &[crate::geometry_kernel::ValidatedPathK1<'a>] {
        self.structure.validated_paths()
    }

    pub(super) fn shape_local_bounds_inputs(
        &self,
    ) -> impl Iterator<Item = ShapeLocalBoundsInput<'a>> + '_ {
        self.structure.shape_local_bounds_inputs()
    }
}

pub(super) fn prepare_prepared_brushes<'a>(
    structure: BrushStructureProof<'a>,
) -> Result<PreparedBrushesProof<'a>, SpatialResolveErrorV2> {
    let brushes = structure.input().resources().brushes();
    let maximum_stops = structure
        .limits()
        .limit(SpatialLimitKindV2::GradientStopsPerBrush);
    let mut gradient = 0_usize;
    let mut prepared = Vec::with_capacity(brushes.len());

    for (index, brush) in brushes.iter().copied().enumerate() {
        let ordinal = u32::try_from(index).expect("phase one validated the brush row capacity");
        let brush = match brush.content() {
            SpatialBrushContentV2::Solid { color } => PreparedBrush::Solid(prepare_solid_p2(color)),
            SpatialBrushContentV2::LinearGradient {
                stop_start,
                stop_length,
                start,
                end,
            } => {
                let stops = structure.gradient_stops(gradient, ordinal);
                let proof = prepare_gradient_p2(
                    ordinal,
                    stop_start,
                    stop_length,
                    start,
                    end,
                    stops,
                    maximum_stops,
                )
                .map_err(map_paint_p2_error)?;
                gradient += 1;
                PreparedBrush::LinearGradient(proof)
            }
        };
        prepared.push(brush);
    }

    Ok(PreparedBrushesProof {
        structure,
        brushes: prepared,
    })
}

#[cfg(test)]
impl PreparedBrushesProof<'_> {
    pub(super) fn prepared_brush_facts(
        &self,
    ) -> Vec<(u32, crate::brush::SpatialBrushKindV2, usize)> {
        self.brushes
            .iter()
            .enumerate()
            .map(|(index, brush)| {
                let (kind, stop_count) = match brush {
                    PreparedBrush::Solid(_) => (crate::brush::SpatialBrushKindV2::Solid, 0),
                    PreparedBrush::LinearGradient(proof) => (
                        crate::brush::SpatialBrushKindV2::LinearGradient,
                        proof.facts().2.len(),
                    ),
                };
                (
                    u32::try_from(index).expect("phase one validated the brush row capacity"),
                    kind,
                    stop_count,
                )
            })
            .collect()
    }

    pub(super) fn prepared_solid_color(&self, brush: u32) -> SpatialRgba8V2 {
        match self
            .brushes
            .get(brush as usize)
            .expect("prepared brush facts use a trusted brush ordinal")
        {
            PreparedBrush::Solid(color) => *color,
            PreparedBrush::LinearGradient(_) => {
                panic!("prepared solid facts require a solid brush")
            }
        }
    }

    pub(super) fn prepared_gradient_facts(
        &self,
        brush: u32,
    ) -> (
        crate::model::SpatialPointV2,
        crate::model::SpatialPointV2,
        Vec<(u16, SpatialRgba8V2)>,
    ) {
        match self
            .brushes
            .get(brush as usize)
            .expect("prepared brush facts use a trusted brush ordinal")
        {
            PreparedBrush::LinearGradient(proof) => proof.facts(),
            PreparedBrush::Solid(_) => {
                panic!("prepared gradient facts require a linear-gradient brush")
            }
        }
    }

    pub(super) fn gradient_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.structure.gradient_range_facts()
    }

    pub(super) fn validated_shape_facts(
        &self,
    ) -> Vec<(u32, crate::shape::SpatialShapeKindV2, usize)> {
        self.structure.validated_shape_facts()
    }

    pub(super) fn polygon_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.structure.polygon_range_facts()
    }

    pub(super) fn validated_path_facts(&self) -> Vec<(u32, usize, usize)> {
        self.structure.validated_path_facts()
    }

    pub(super) fn subpath_total(&self) -> usize {
        self.structure.subpath_total()
    }

    pub(super) fn path_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.structure.path_range_facts()
    }

    pub(super) fn prepared_island_facts(&self) -> Vec<(u32, Vec<u32>)> {
        self.structure.prepared_island_facts()
    }
}
