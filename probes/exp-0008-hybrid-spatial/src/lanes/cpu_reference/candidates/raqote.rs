use raqote::{
    AntialiasMode, Color, DrawOptions, DrawTarget, ExtendMode, FilterMode, Gradient, GradientStop,
    Image, IntPoint, IntRect, PathBuilder, Point, SolidSource, Source, Spread, Transform,
};

use super::super::faults::{detects as detects_fault, preflight_cases};
use super::super::types::{
    CPU_SCALE_V2 as S, CpuCandidateV2, CpuCaseV2, CpuCommandV2, CpuFaultKindV2, CpuRasterV2,
    CpuResultV2, CpuRunV2, CpuTransformV2,
};

pub(crate) fn run(cases: &[CpuCaseV2]) -> CpuResultV2<CpuRunV2> {
    preflight_cases(cases)?;
    Ok(CpuRunV2 {
        candidate: Some(CpuCandidateV2::Raqote),
        cases: cases.iter().map(render_case).collect(),
    })
}

pub(crate) fn detects(kind: CpuFaultKindV2) -> bool {
    detects_fault(kind)
}

fn render_case(case: &CpuCaseV2) -> CpuRasterV2 {
    let mut target = DrawTarget::new(case.width as i32, case.height as i32);
    for command in &case.commands {
        render_command(command, &mut target);
    }
    let mut bytes = Vec::with_capacity(case.width as usize * case.height as usize * 4);
    for pixel in target.into_vec() {
        bytes.extend_from_slice(&[
            ((pixel >> 16) & 0xff) as u8,
            ((pixel >> 8) & 0xff) as u8,
            (pixel & 0xff) as u8,
            ((pixel >> 24) & 0xff) as u8,
        ]);
    }
    CpuRasterV2 {
        ordinal: case.ordinal,
        width: case.width,
        height: case.height,
        bytes,
    }
}

fn render_command(command: &CpuCommandV2, target: &mut DrawTarget) {
    let options = options();
    match command {
        CpuCommandV2::SolidRect {
            rect,
            color,
            transform,
            clip,
        } => {
            push_clip(target, *clip);
            target.set_transform(&to_transform(*transform));
            target.fill_rect(
                scalar(rect[0]),
                scalar(rect[1]),
                scalar(rect[2] - rect[0]),
                scalar(rect[3] - rect[1]),
                &solid(*color),
                &options,
            );
            target.set_transform(&Transform::identity());
            pop_clip(target, *clip);
        }
        CpuCommandV2::SolidPolygon {
            points,
            color,
            transform,
            clip,
        } => {
            let mut builder = PathBuilder::new();
            builder.move_to(scalar(points[0][0]), scalar(points[0][1]));
            for point in &points[1..] {
                builder.line_to(scalar(point[0]), scalar(point[1]));
            }
            builder.close();
            push_clip(target, *clip);
            target.set_transform(&to_transform(*transform));
            target.fill(&builder.finish(), &solid(*color), &options);
            target.set_transform(&Transform::identity());
            pop_clip(target, *clip);
        }
        CpuCommandV2::LinearGradient {
            rect,
            start,
            end,
            colors,
        } => {
            let gradient = Gradient {
                stops: vec![
                    GradientStop {
                        position: 0.0,
                        color: to_color(colors[0]),
                    },
                    GradientStop {
                        position: 1.0,
                        color: to_color(colors[1]),
                    },
                ],
            };
            let source = Source::new_linear_gradient(
                gradient,
                Point::new(scalar(start[0]), scalar(start[1])),
                Point::new(scalar(end[0]), scalar(end[1])),
                Spread::Pad,
            );
            target.fill_rect(
                scalar(rect[0]),
                scalar(rect[1]),
                scalar(rect[2] - rect[0]),
                scalar(rect[3] - rect[1]),
                &source,
                &options,
            );
        }
        CpuCommandV2::Image {
            origin,
            width,
            height,
            premultiplied_rgba,
            ..
        } => {
            let pixels = rgba_to_argb(premultiplied_rgba);
            let image = Image {
                width: *width as i32,
                height: *height as i32,
                data: &pixels,
            };
            let x = scalar(origin[0]);
            let y = scalar(origin[1]);
            let source = Source::Image(
                image,
                ExtendMode::Pad,
                FilterMode::Nearest,
                Transform::translation(-x, -y),
            );
            target.fill_rect(x, y, *width as f32, *height as f32, &source, &options);
        }
    }
}

fn options() -> DrawOptions {
    DrawOptions {
        antialias: AntialiasMode::None,
        ..DrawOptions::default()
    }
}

fn solid(color: [u8; 4]) -> Source<'static> {
    Source::Solid(SolidSource::from_unpremultiplied_argb(
        color[3], color[0], color[1], color[2],
    ))
}

fn to_color(color: [u8; 4]) -> Color {
    Color::new(color[3], color[0], color[1], color[2])
}

fn to_transform(transform: CpuTransformV2) -> Transform {
    let [a, b, c, d, tx, ty] = transform.coefficients;
    Transform::new(
        scalar(a),
        scalar(b),
        scalar(c),
        scalar(d),
        scalar(tx),
        scalar(ty),
    )
}

fn push_clip(target: &mut DrawTarget, clip: Option<[i32; 4]>) {
    if let Some([x0, y0, x1, y1]) = clip {
        target.push_clip_rect(IntRect::new(
            IntPoint::new(x0 / S, y0 / S),
            IntPoint::new(x1 / S, y1 / S),
        ));
    }
}

fn pop_clip(target: &mut DrawTarget, clip: Option<[i32; 4]>) {
    if clip.is_some() {
        target.pop_clip();
    }
}

fn rgba_to_argb(bytes: &[u8]) -> Vec<u32> {
    bytes
        .chunks_exact(4)
        .map(|pixel| {
            (u32::from(pixel[3]) << 24)
                | (u32::from(pixel[0]) << 16)
                | (u32::from(pixel[1]) << 8)
                | u32::from(pixel[2])
        })
        .collect()
}

fn scalar(value: i32) -> f32 {
    value as f32 / S as f32
}
