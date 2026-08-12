//! Deterministic CPU reference raster over accepted snapshot projection.

use super::super::model::{PreparedBrushContent, PreparedPaintContent, PreparedSpatialState};
use super::SpatialResolvedSnapshotV2;
use super::hit::output_bounds_contains;
use crate::brush::SpatialRgba8V2;
use crate::model::{SpatialPointV2, SpatialScalarV2};
use crate::paint_kernel::image_sample::sample_image_fields_p6;
use crate::paint_kernel::sample::sample_gradient_fields_p3;
use crate::paint_kernel::{apply_opacity_p1, source_over_p1};
use crate::reference_raster::{
    ReferenceRasterErrorV2, ReferenceRasterLimitKindV2, ReferenceRasterLimitsV2, ReferenceRasterV2,
};

const SAMPLE_OFFSETS: [i64; 4] = [
    SpatialScalarV2::SCALE / 8,
    3 * SpatialScalarV2::SCALE / 8,
    5 * SpatialScalarV2::SCALE / 8,
    7 * SpatialScalarV2::SCALE / 8,
];

impl SpatialResolvedSnapshotV2 {
    /// Renders one deterministic packed premultiplied RGBA8 reference raster.
    #[must_use = "reference raster errors must be handled"]
    pub fn rasterize_reference(
        &self,
        limits: ReferenceRasterLimitsV2,
    ) -> Result<ReferenceRasterV2, ReferenceRasterErrorV2> {
        let viewport = self.viewport();
        let width =
            u32::try_from(viewport.width()).expect("resolved viewport width is nonnegative");
        let height =
            u32::try_from(viewport.height()).expect("resolved viewport height is nonnegative");
        let observed = u128::from(width) * u128::from(height);
        let allocation_maximum = (isize::MAX as usize) / 4;
        let maximum = limits
            .limit(ReferenceRasterLimitKindV2::Pixels)
            .min(allocation_maximum);
        if observed > maximum as u128 {
            return Err(ReferenceRasterErrorV2::limit_exceeded(
                observed,
                maximum as u128,
            ));
        }

        let byte_length = usize::try_from(observed * 4)
            .expect("successful raster preflight makes byte length representable");
        let mut bytes = vec![0; byte_length].into_boxed_slice();
        if width == 0 || height == 0 {
            return Ok(ReferenceRasterV2::from_bytes(width, height, bytes));
        }
        let mut byte_index = 0_usize;
        for y in 0..height {
            for x in 0..width {
                let color = self.rasterize_pixel(x, y);
                bytes[byte_index] = color.r();
                bytes[byte_index + 1] = color.g();
                bytes[byte_index + 2] = color.b();
                bytes[byte_index + 3] = color.a();
                byte_index += 4;
            }
        }
        Ok(ReferenceRasterV2::from_bytes(width, height, bytes))
    }

    fn rasterize_pixel(&self, x: u32, y: u32) -> SpatialRgba8V2 {
        let base_x = i64::from(x) * SpatialScalarV2::SCALE;
        let base_y = i64::from(y) * SpatialScalarV2::SCALE;
        let mut total = ChannelTotals::ZERO;
        for offset_y in SAMPLE_OFFSETS {
            for offset_x in SAMPLE_OFFSETS {
                let scene_point = SpatialPointV2::new(
                    SpatialScalarV2::new(base_x + offset_x),
                    SpatialScalarV2::new(base_y + offset_y),
                );
                total.add(self.rasterize_sample(scene_point));
            }
        }
        total.average()
    }

    fn rasterize_sample(&self, scene_point: SpatialPointV2) -> SpatialRgba8V2 {
        let state = &self.prepared.state;
        let mut destination = SpatialRgba8V2::new(0, 0, 0, 0);
        for (row, paint) in self.paints.iter().zip(state.paints.iter()) {
            let terminal = paint_clip(&paint.content);
            if let Some(terminal) = terminal
                && !self.clip_chain_contains(state, terminal, scene_point)
            {
                continue;
            }
            if !output_bounds_contains(row.world_aabb(), scene_point) {
                continue;
            }
            let Some(local_point) = row.world_from_local().inverse_point(scene_point) else {
                continue;
            };
            let Some(source) = self.sample_paint(state, paint, local_point) else {
                continue;
            };
            destination = source_over_p1(source, destination);
        }
        destination
    }

    fn sample_paint(
        &self,
        state: &PreparedSpatialState,
        paint: &super::super::model::PreparedPaintPlan,
        local_point: SpatialPointV2,
    ) -> Option<SpatialRgba8V2> {
        match &paint.content {
            PreparedPaintContent::Coverage {
                coverage,
                brush,
                opacity,
                ..
            } => {
                if !self.coverage_contains(state, coverage, paint.local_bounds, local_point) {
                    return None;
                }
                let color = match &state.brushes[*brush as usize].content {
                    PreparedBrushContent::Solid(color) => *color,
                    PreparedBrushContent::LinearGradient { start, end, stops } => {
                        sample_gradient_fields_p3(*start, *end, stops, local_point)
                    }
                };
                Some(apply_opacity_p1(color, *opacity))
            }
            PreparedPaintContent::Image {
                image,
                source,
                destination,
                opacity,
                ..
            } => {
                let plan = &state.images[*image as usize];
                let input = self.prepared.source.as_input();
                let bytes = input.resources().images()[*image as usize].bytes();
                sample_image_fields_p6(
                    plan.stride,
                    bytes,
                    *source,
                    *destination,
                    *opacity,
                    paint.local_bounds,
                    local_point,
                )
            }
        }
    }
}

const fn paint_clip(content: &PreparedPaintContent) -> Option<u32> {
    match content {
        PreparedPaintContent::Coverage { clip, .. } | PreparedPaintContent::Image { clip, .. } => {
            *clip
        }
    }
}

struct ChannelTotals {
    red: u32,
    green: u32,
    blue: u32,
    alpha: u32,
}

impl ChannelTotals {
    const ZERO: Self = Self {
        red: 0,
        green: 0,
        blue: 0,
        alpha: 0,
    };

    fn add(&mut self, color: SpatialRgba8V2) {
        self.red += u32::from(color.r());
        self.green += u32::from(color.g());
        self.blue += u32::from(color.b());
        self.alpha += u32::from(color.a());
    }

    fn average(self) -> SpatialRgba8V2 {
        SpatialRgba8V2::new(
            average_channel(self.red),
            average_channel(self.green),
            average_channel(self.blue),
            average_channel(self.alpha),
        )
    }
}

fn average_channel(total: u32) -> u8 {
    u8::try_from((total + 8) / 16).expect("the average of byte channels remains a byte")
}
