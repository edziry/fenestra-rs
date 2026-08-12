use super::coverage::coverage_contains;
use super::hit::clip_contains;
use super::numeric::{Point, SCALE, contains, inverse_point, round_ratio};
use super::scene::{Brush, PaintContent, Scene};
use super::types::{Projection, Raster};

const OFFSETS: [i64; 4] = [SCALE / 8, 3 * SCALE / 8, 5 * SCALE / 8, 7 * SCALE / 8];
const IMAGE: [u8; 16] = [255, 0, 0, 255, 0, 128, 0, 128, 0, 0, 64, 64, 0, 0, 0, 0];

pub fn raster(scene: &Scene, projection: &Projection, viewport: [i32; 2]) -> Raster {
    let width = u32::try_from(viewport[0]).expect("viewport width should be positive");
    let height = u32::try_from(viewport[1]).expect("viewport height should be positive");
    let mut bytes = Vec::with_capacity(
        usize::try_from(width * height * 4).expect("raster byte count should fit"),
    );
    for y in 0..height {
        for x in 0..width {
            let mut totals = [0_u32; 4];
            for offset_y in OFFSETS {
                for offset_x in OFFSETS {
                    let color = sample(
                        scene,
                        projection,
                        [
                            i64::from(x) * SCALE + offset_x,
                            i64::from(y) * SCALE + offset_y,
                        ],
                    );
                    for (total, channel) in totals.iter_mut().zip(color) {
                        *total += u32::from(channel);
                    }
                }
            }
            bytes.extend(
                totals.map(|total| {
                    u8::try_from((total + 8) / 16).expect("averaged channel should fit")
                }),
            );
        }
    }
    Raster {
        width,
        height,
        stride: u64::from(width) * 4,
        bytes: bytes.into_boxed_slice(),
    }
}

fn sample(scene: &Scene, projection: &Projection, point: Point) -> [u8; 4] {
    let mut destination = [0; 4];
    for (index, paint) in scene.paints.iter().enumerate() {
        let clip = match paint.content {
            PaintContent::Coverage { clip, .. } | PaintContent::Image { clip } => clip,
        };
        if let Some(clip) = clip
            && !clip_contains(scene, projection, clip, point)
        {
            continue;
        }
        let row = &projection.paints[index];
        if !contains(row.aabb, point) {
            continue;
        }
        let local = inverse_point(row.affine, point);
        let source = match &paint.content {
            PaintContent::Coverage {
                coverage,
                brush,
                opacity,
                ..
            } => {
                if !coverage_contains(scene, *coverage, paint.bounds, local) {
                    continue;
                }
                apply_opacity(
                    brush_sample(&scene.brushes[*brush as usize], local),
                    *opacity,
                )
            }
            PaintContent::Image { .. } => {
                let Some(color) = image_sample(local) else {
                    continue;
                };
                color
            }
        };
        destination = source_over(source, destination);
    }
    destination
}

fn brush_sample(brush: &Brush, point: Point) -> [u8; 4] {
    match brush {
        Brush::Solid(color) => normalize(*color),
        Brush::Gradient { start, end, stops } => {
            let dx = i128::from(end[0] - start[0]);
            let dy = i128::from(end[1] - start[1]);
            let relative_x = i128::from(point[0] - start[0]);
            let relative_y = i128::from(point[1] - start[1]);
            let denominator = dx * dx + dy * dy;
            let numerator = (relative_x * dx + relative_y * dy) * i128::from(u16::MAX);
            let parameter =
                u16::try_from(round_ratio(numerator, denominator).clamp(0, i128::from(u16::MAX)))
                    .expect("gradient parameter should fit");
            let mut lower = 0;
            for (index, stop) in stops.iter().enumerate().skip(1) {
                if stop.0 > parameter {
                    break;
                }
                lower = index;
            }
            let lower_stop = (stops[lower].0, normalize(stops[lower].1));
            let Some(upper) = stops.get(lower + 1) else {
                return lower_stop.1;
            };
            let upper_stop = (upper.0, normalize(upper.1));
            let local = parameter - lower_stop.0;
            let span = upper_stop.0 - lower_stop.0;
            std::array::from_fn(|channel| {
                let lower = i128::from(lower_stop.1[channel]);
                let difference = i128::from(upper_stop.1[channel]) - lower;
                u8::try_from(lower + round_ratio(difference * i128::from(local), i128::from(span)))
                    .expect("interpolated channel should fit")
            })
        }
    }
}

fn image_sample(point: Point) -> Option<[u8; 4]> {
    let near = 2 * SCALE;
    let far = 10 * SCALE;
    if point[0] < near || point[0] >= far || point[1] < near || point[1] >= far {
        return None;
    }
    let source_x = u32::try_from(i128::from(point[0] - near) * 2 / i128::from(8 * SCALE))
        .expect("source x should fit");
    let source_y = u32::try_from(i128::from(point[1] - near) * 2 / i128::from(8 * SCALE))
        .expect("source y should fit");
    let offset = usize::try_from(source_y * 8 + source_x * 4).expect("image offset should fit");
    Some(apply_opacity(
        [
            IMAGE[offset],
            IMAGE[offset + 1],
            IMAGE[offset + 2],
            IMAGE[offset + 3],
        ],
        192,
    ))
}

fn normalize(color: [u8; 4]) -> [u8; 4] {
    [
        scale_byte(color[0], color[3]),
        scale_byte(color[1], color[3]),
        scale_byte(color[2], color[3]),
        color[3],
    ]
}

fn apply_opacity(color: [u8; 4], opacity: u8) -> [u8; 4] {
    color.map(|channel| scale_byte(channel, opacity))
}

fn source_over(source: [u8; 4], destination: [u8; 4]) -> [u8; 4] {
    let inverse = u8::MAX - source[3];
    std::array::from_fn(|channel| {
        let output =
            u16::from(source[channel]) + u16::from(scale_byte(destination[channel], inverse));
        u8::try_from(output).expect("source-over channel should fit")
    })
}

fn scale_byte(channel: u8, factor: u8) -> u8 {
    ((u16::from(channel) * u16::from(factor) + 127) / 255) as u8
}
