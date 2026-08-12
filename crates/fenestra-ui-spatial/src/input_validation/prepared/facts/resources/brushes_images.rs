use super::super::super::PreparedSpatialV2;
use super::super::super::model::PreparedBrushContent;
use super::ordinal;
use crate::brush::{SpatialBrushKindV2, SpatialRgba8V2};
use crate::model::SpatialPointV2;

impl PreparedSpatialV2 {
    pub(in crate::input_validation) fn gradient_range_facts(&self) -> Vec<(u32, u128, u128)> {
        self.state
            .brushes
            .iter()
            .enumerate()
            .filter_map(|(index, brush)| {
                brush
                    .gradient_range
                    .as_ref()
                    .map(|range| (ordinal(index), range.start as u128, range.end as u128))
            })
            .collect()
    }

    pub(in crate::input_validation) fn prepared_brush_facts(
        &self,
    ) -> Vec<(u32, SpatialBrushKindV2, usize)> {
        self.state
            .brushes
            .iter()
            .enumerate()
            .map(|(index, brush)| match &brush.content {
                PreparedBrushContent::Solid(_) => (ordinal(index), SpatialBrushKindV2::Solid, 0),
                PreparedBrushContent::LinearGradient { stops, .. } => (
                    ordinal(index),
                    SpatialBrushKindV2::LinearGradient,
                    stops.len(),
                ),
            })
            .collect()
    }

    pub(in crate::input_validation) fn prepared_solid_color(&self, brush: u32) -> SpatialRgba8V2 {
        match &self.state.brushes[brush as usize].content {
            PreparedBrushContent::Solid(color) => *color,
            PreparedBrushContent::LinearGradient { .. } => {
                panic!("prepared solid facts require a solid brush")
            }
        }
    }

    pub(in crate::input_validation) fn prepared_gradient_facts(
        &self,
        brush: u32,
    ) -> (SpatialPointV2, SpatialPointV2, Vec<(u16, SpatialRgba8V2)>) {
        match &self.state.brushes[brush as usize].content {
            PreparedBrushContent::LinearGradient { start, end, stops } => {
                (*start, *end, stops.to_vec())
            }
            PreparedBrushContent::Solid(_) => {
                panic!("prepared gradient facts require a gradient brush")
            }
        }
    }

    pub(in crate::input_validation) fn image_plan_facts(&self) -> Vec<(u32, u32, u32, u32)> {
        self.state
            .images
            .iter()
            .enumerate()
            .map(|(index, image)| (ordinal(index), image.width, image.height, image.stride))
            .collect()
    }
}
