use super::faults::preflight_cases;
use super::types::{
    CPU_SCALE_V2 as S, CpuCaseV2, CpuCommandV2, CpuRasterV2, CpuResultV2, CpuRunV2, CpuTransformV2,
};

pub(crate) fn literal_cpu_run_v2(cases: &[CpuCaseV2]) -> CpuResultV2<CpuRunV2> {
    preflight_cases(cases)?;
    Ok(CpuRunV2::literal(cases.iter().map(render_case).collect()))
}

fn render_case(case: &CpuCaseV2) -> CpuRasterV2 {
    let mut bytes = vec![0; case.width as usize * case.height as usize * 4];
    for command in &case.commands {
        render_command(case, command, &mut bytes);
    }
    CpuRasterV2 {
        ordinal: case.ordinal,
        width: case.width,
        height: case.height,
        bytes,
    }
}

fn render_command(case: &CpuCaseV2, command: &CpuCommandV2, bytes: &mut [u8]) {
    for y in 0..case.height {
        for x in 0..case.width {
            let scene = [x as i32 * S + S / 2, y as i32 * S + S / 2];
            if let Some(source) = sample(command, scene) {
                let offset = (y as usize * case.width as usize + x as usize) * 4;
                blend_source_over(&mut bytes[offset..offset + 4], source);
            }
        }
    }
}

fn sample(command: &CpuCommandV2, scene: [i32; 2]) -> Option<[u8; 4]> {
    match command {
        CpuCommandV2::SolidRect {
            rect,
            color,
            transform,
            clip,
        } => {
            let local = clipped_local(scene, *clip, *transform)?;
            contains_rect(*rect, local).then(|| premultiply(*color))
        }
        CpuCommandV2::SolidPolygon {
            points,
            color,
            transform,
            clip,
        } => {
            let local = clipped_local(scene, *clip, *transform)?;
            point_in_polygon(points, local).then(|| premultiply(*color))
        }
        CpuCommandV2::LinearGradient {
            rect,
            start,
            end,
            colors,
        } => contains_rect(*rect, scene).then(|| {
            let dx = i64::from(end[0] - start[0]);
            let dy = i64::from(end[1] - start[1]);
            let numerator =
                i64::from(scene[0] - start[0]) * dx + i64::from(scene[1] - start[1]) * dy;
            let denominator = dx * dx + dy * dy;
            let t = ((numerator * i64::from(S)) / denominator).clamp(0, i64::from(S));
            let mut color = [0; 4];
            for channel in 0..4 {
                color[channel] = interpolate(colors[0][channel], colors[1][channel], t as i32);
            }
            premultiply(color)
        }),
        CpuCommandV2::Image {
            origin,
            width,
            height,
            stride,
            premultiplied_rgba,
            ..
        } => {
            let local_x = scene[0] - origin[0];
            let local_y = scene[1] - origin[1];
            if local_x < 0 || local_y < 0 {
                return None;
            }
            let x = (local_x / S) as u32;
            let y = (local_y / S) as u32;
            if x >= *width || y >= *height {
                return None;
            }
            let offset = y as usize * *stride as usize + x as usize * 4;
            Some([
                premultiplied_rgba[offset],
                premultiplied_rgba[offset + 1],
                premultiplied_rgba[offset + 2],
                premultiplied_rgba[offset + 3],
            ])
        }
    }
}

fn clipped_local(
    scene: [i32; 2],
    clip: Option<[i32; 4]>,
    transform: CpuTransformV2,
) -> Option<[i32; 2]> {
    if clip.is_some_and(|bounds| !contains_rect(bounds, scene)) {
        return None;
    }
    inverse(transform, scene)
}

fn inverse(transform: CpuTransformV2, scene: [i32; 2]) -> Option<[i32; 2]> {
    let [a, b, c, d, tx, ty] = transform.coefficients;
    let determinant = i64::from(a) * i64::from(d) - i64::from(b) * i64::from(c);
    if determinant == 0 {
        return None;
    }
    let x = i64::from(scene[0] - tx);
    let y = i64::from(scene[1] - ty);
    Some([
        ((i64::from(d) * x - i64::from(c) * y) * i64::from(S) / determinant) as i32,
        ((-i64::from(b) * x + i64::from(a) * y) * i64::from(S) / determinant) as i32,
    ])
}

fn contains_rect(rect: [i32; 4], point: [i32; 2]) -> bool {
    point[0] >= rect[0] && point[0] < rect[2] && point[1] >= rect[1] && point[1] < rect[3]
}

fn point_in_polygon(points: &[[i32; 2]], point: [i32; 2]) -> bool {
    let mut inside = false;
    for index in 0..points.len() {
        let first = points[index];
        let second = points[(index + 1) % points.len()];
        if (first[1] > point[1]) != (second[1] > point[1]) {
            let crossing = i64::from(second[0] - first[0]) * i64::from(point[1] - first[1]);
            let height = i64::from(second[1] - first[1]);
            let at_right = if height > 0 {
                crossing > i64::from(point[0] - first[0]) * height
            } else {
                crossing < i64::from(point[0] - first[0]) * height
            };
            inside ^= at_right;
        }
    }
    inside
}

fn interpolate(first: u8, second: u8, t: i32) -> u8 {
    ((i32::from(first) * (S - t) + i32::from(second) * t + S / 2) / S) as u8
}

fn premultiply(color: [u8; 4]) -> [u8; 4] {
    let alpha = color[3];
    [
        mul_div_255(color[0], alpha),
        mul_div_255(color[1], alpha),
        mul_div_255(color[2], alpha),
        alpha,
    ]
}

fn blend_source_over(destination: &mut [u8], source: [u8; 4]) {
    let inverse_alpha = 255 - source[3];
    for channel in 0..3 {
        destination[channel] =
            source[channel].saturating_add(mul_div_255(destination[channel], inverse_alpha));
    }
    destination[3] = source[3].saturating_add(mul_div_255(destination[3], inverse_alpha));
}

fn mul_div_255(first: u8, second: u8) -> u8 {
    ((u16::from(first) * u16::from(second) + 127) / 255) as u8
}
