use crate::baseline::literal_types::{BrushInputV2, FIXED_ONE_V2, PaintContentInputV2, PointV2};

use super::coverage::{clip_allows, coverage_contains};
use super::numeric::{inverse_point, round_ratio};
use super::types::{ResolvedPaint, ScenePlan};

const OFFSETS: [i64; 4] = [
    FIXED_ONE_V2 / 8,
    3 * FIXED_ONE_V2 / 8,
    5 * FIXED_ONE_V2 / 8,
    7 * FIXED_ONE_V2 / 8,
];

pub(super) fn raster(plan: &ScenePlan<'_>) -> Vec<u8> {
    let width = plan.scene.viewport.0;
    let height = plan.scene.viewport.1;
    let mut output = Vec::with_capacity(width as usize * height as usize * 4);
    for y in 0..height {
        for x in 0..width {
            let mut total = [0_u32; 4];
            for offset_y in OFFSETS {
                for offset_x in OFFSETS {
                    let color = sample_scene(
                        plan,
                        PointV2 {
                            x: i64::from(x) * FIXED_ONE_V2 + offset_x,
                            y: i64::from(y) * FIXED_ONE_V2 + offset_y,
                        },
                    );
                    for (channel, value) in total.iter_mut().zip(color) {
                        *channel += u32::from(value);
                    }
                }
            }
            output.extend(total.map(|value| ((value + 8) / 16) as u8));
        }
    }
    output
}

fn sample_scene(plan: &ScenePlan<'_>, scene_point: PointV2) -> [u8; 4] {
    let mut destination = [0; 4];
    for (input, resolved) in plan.scene.paints.iter().zip(&plan.paints) {
        let clip = match &input.content {
            PaintContentInputV2::Coverage { clip, .. }
            | PaintContentInputV2::Image { clip, .. } => *clip,
        };
        if !clip_allows(plan, clip, scene_point) || !resolved.world_bounds.contains(scene_point) {
            continue;
        }
        let Some(local) = inverse_point(plan.worlds[input.owner as usize], scene_point) else {
            continue;
        };
        let Some(source) = sample_paint(plan, resolved, local) else {
            continue;
        };
        destination = source_over(source, destination);
    }
    destination
}

fn sample_paint(
    plan: &ScenePlan<'_>,
    resolved: &ResolvedPaint<'_>,
    local: PointV2,
) -> Option<[u8; 4]> {
    match resolved.input {
        PaintContentInputV2::Coverage {
            coverage,
            brush,
            opacity,
            ..
        } => {
            if !coverage_contains(plan, *coverage, resolved.local_bounds, local) {
                return None;
            }
            let color = match &plan.scene.brushes[*brush as usize] {
                BrushInputV2::Solid { color, .. } => normalize(*color),
                BrushInputV2::Linear {
                    stops, start, end, ..
                } => gradient(stops, *start, *end, local),
            };
            Some(opacity_color(color, *opacity))
        }
        PaintContentInputV2::Image {
            image,
            source,
            destination,
            opacity,
            ..
        } => {
            if local.x < resolved.local_bounds.edges[0]
                || local.x >= resolved.local_bounds.edges[2]
                || local.y < resolved.local_bounds.edges[1]
                || local.y >= resolved.local_bounds.edges[3]
            {
                return None;
            }
            let image = &plan.scene.images[*image as usize];
            let source_x = (source.x / FIXED_ONE_V2) as u32
                + ((local.x - destination.x) as i128 * (source.width / FIXED_ONE_V2) as i128
                    / destination.width as i128) as u32;
            let source_y = (source.y / FIXED_ONE_V2) as u32
                + ((local.y - destination.y) as i128 * (source.height / FIXED_ONE_V2) as i128
                    / destination.height as i128) as u32;
            let index = source_y as usize * image.stride as usize + source_x as usize * 4;
            Some(opacity_color(
                [
                    image.bytes[index],
                    image.bytes[index + 1],
                    image.bytes[index + 2],
                    image.bytes[index + 3],
                ],
                *opacity,
            ))
        }
    }
}

fn gradient(
    stops: &[crate::baseline::literal_types::GradientStopInputV2],
    start: PointV2,
    end: PointV2,
    point: PointV2,
) -> [u8; 4] {
    let dx = end.x as i128 - start.x as i128;
    let dy = end.y as i128 - start.y as i128;
    let relative_x = point.x as i128 - start.x as i128;
    let relative_y = point.y as i128 - start.y as i128;
    let parameter = round_ratio(
        (relative_x * dx + relative_y * dy) * u16::MAX as i128,
        dx * dx + dy * dy,
    )
    .clamp(0, u16::MAX as i128) as u16;
    let mut lower = 0;
    for (index, stop) in stops.iter().enumerate().skip(1) {
        if stop.offset > parameter {
            break;
        }
        lower = index;
    }
    if lower + 1 == stops.len() {
        return normalize(stops[lower].color);
    }
    let low = stops[lower];
    let high = stops[lower + 1];
    let local = parameter - low.offset;
    let span = high.offset - low.offset;
    let low_color = normalize(low.color);
    let high_color = normalize(high.color);
    std::array::from_fn(|index| {
        let difference = i128::from(high_color[index]) - i128::from(low_color[index]);
        (i128::from(low_color[index])
            + round_ratio(difference * i128::from(local), i128::from(span))) as u8
    })
}

fn normalize(color: [u8; 4]) -> [u8; 4] {
    [
        scale(color[0], color[3]),
        scale(color[1], color[3]),
        scale(color[2], color[3]),
        color[3],
    ]
}

fn opacity_color(color: [u8; 4], opacity: u8) -> [u8; 4] {
    color.map(|channel| scale(channel, opacity))
}

fn source_over(source: [u8; 4], destination: [u8; 4]) -> [u8; 4] {
    let inverse = u8::MAX - source[3];
    std::array::from_fn(|index| source[index] + scale(destination[index], inverse))
}

fn scale(channel: u8, factor: u8) -> u8 {
    ((u16::from(channel) * u16::from(factor) + 127) / 255) as u8
}
