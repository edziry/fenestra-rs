//! Shared Geometry K3 and Paint P5 local-bound preparation.

use super::flattened_paths::FlattenedPathsProof;
use super::geometry_k3_mapping::map_geometry_k3_error;
use super::paint_p5_mapping::map_paint_p5_error;
use super::validated_hit_items::HitLocalBoundsInput;
use super::validated_paint_items::PaintLocalBoundsInput;
use super::validated_shapes::ShapeLocalBoundsInput;
use crate::aabb::SpatialAabbV2;
use crate::aggregate_input::SpatialInputV2;
use crate::geometry_kernel::{
    DerivedLocalBoundsK3, GeometryK1StrokeSource, derive_circle_bounds_k3, derive_path_bounds_k3,
    derive_polygon_bounds_k3, derive_rect_bounds_k3, fill_bounds_k3, stroke_bounds_k3,
};
use crate::limits::SpatialLimitsV2;
use crate::paint_kernel::{ValidatedImagePaintP5, finish_image_paint_bounds_after_item_phase_p5};
use crate::resolve_error::SpatialResolveErrorV2;

#[cfg(test)]
mod facts;

enum PaintLocalBounds<'a> {
    Coverage(SpatialAabbV2),
    Image(ValidatedImagePaintP5<'a>),
}

impl PaintLocalBounds<'_> {
    const fn local_bounds(&self) -> SpatialAabbV2 {
        match self {
            Self::Coverage(bounds) => *bounds,
            Self::Image(image) => image.local_bounds(),
        }
    }
}

pub(super) struct LocalBoundsProof<'a> {
    flattened: FlattenedPathsProof<'a>,
    shapes: Vec<DerivedLocalBoundsK3>,
    paints: Vec<PaintLocalBounds<'a>>,
    hits: Vec<SpatialAabbV2>,
}

impl<'a> LocalBoundsProof<'a> {
    pub(super) fn input(&self) -> SpatialInputV2<'a> {
        self.flattened.input()
    }

    pub(super) fn limits(&self) -> SpatialLimitsV2 {
        self.flattened.limits()
    }

    pub(super) fn dependency_islands(
        &self,
    ) -> impl Iterator<Item = super::islands::preflight::DependencyIslandInput<'_>> + '_ {
        self.flattened.dependency_islands()
    }
}

pub(super) fn prepare_local_bounds<'a>(
    flattened: FlattenedPathsProof<'a>,
) -> Result<LocalBoundsProof<'a>, SpatialResolveErrorV2> {
    let mut shape_bounds = Vec::with_capacity(flattened.input().geometry().shapes().len());
    for (index, input) in flattened.shape_local_bounds_inputs().enumerate() {
        let ordinal = trusted_ordinal(index, "shape");
        let bounds = match input {
            ShapeLocalBoundsInput::Rect(rect) => {
                derive_rect_bounds_k3(ordinal, rect).map_err(map_geometry_k3_error)?
            }
            ShapeLocalBoundsInput::Circle(circle) => {
                derive_circle_bounds_k3(ordinal, circle).map_err(map_geometry_k3_error)?
            }
            ShapeLocalBoundsInput::Polygon(polygon) => derive_polygon_bounds_k3(polygon),
            ShapeLocalBoundsInput::Path(path) => {
                let path = flattened
                    .validated_paths()
                    .get(trusted_reference(path.get(), "path"))
                    .copied()
                    .expect("shape structure validated every path reference");
                derive_path_bounds_k3(path)
            }
        };
        shape_bounds.push(bounds);
    }

    let mut paint_bounds = Vec::with_capacity(flattened.input().items().paint_items().len());
    for (index, input) in flattened.paint_local_bounds_inputs().enumerate() {
        let ordinal = trusted_ordinal(index, "paint");
        let bounds = match input {
            PaintLocalBoundsInput::Fill { shape } => PaintLocalBounds::Coverage(fill_bounds_k3(
                trusted_shape_bounds(&shape_bounds, shape),
            )),
            PaintLocalBoundsInput::RoundStroke { shape, stroke } => PaintLocalBounds::Coverage(
                stroke_bounds_k3(
                    trusted_shape_bounds(&shape_bounds, shape),
                    GeometryK1StrokeSource::Paint { index: ordinal },
                    stroke,
                )
                .map_err(map_geometry_k3_error)?,
            ),
            PaintLocalBoundsInput::Image { preclip } => PaintLocalBounds::Image(
                finish_image_paint_bounds_after_item_phase_p5(preclip)
                    .map_err(map_paint_p5_error)?,
            ),
        };
        paint_bounds.push(bounds);
    }

    let mut hit_bounds = Vec::with_capacity(flattened.input().items().hit_items().len());
    for (index, input) in flattened.hit_local_bounds_inputs().enumerate() {
        let ordinal = trusted_ordinal(index, "hit");
        let bounds = match input {
            HitLocalBoundsInput::Fill { shape } => {
                fill_bounds_k3(trusted_shape_bounds(&shape_bounds, shape))
            }
            HitLocalBoundsInput::RoundStroke { shape, stroke } => stroke_bounds_k3(
                trusted_shape_bounds(&shape_bounds, shape),
                GeometryK1StrokeSource::Hit { index: ordinal },
                stroke,
            )
            .map_err(map_geometry_k3_error)?,
        };
        hit_bounds.push(bounds);
    }

    Ok(LocalBoundsProof {
        flattened,
        shapes: shape_bounds,
        paints: paint_bounds,
        hits: hit_bounds,
    })
}

fn trusted_shape_bounds(bounds: &[DerivedLocalBoundsK3], shape: u32) -> &DerivedLocalBoundsK3 {
    bounds
        .get(trusted_reference(shape, "shape"))
        .expect("item validation retained only existing shape references")
}

fn trusted_reference(index: u32, table: &str) -> usize {
    usize::try_from(index).unwrap_or_else(|_| panic!("validated {table} reference fits usize"))
}

fn trusted_ordinal(index: usize, table: &str) -> u32 {
    u32::try_from(index).unwrap_or_else(|_| panic!("phase one validated the {table} row capacity"))
}
