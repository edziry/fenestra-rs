use tiny_skia::{
    Color, FillRule, GradientStop, IntSize, LinearGradient, Mask, Paint, PathBuilder, Pixmap,
    PixmapPaint, Point, Rect, SpreadMode, Transform,
};

use super::super::faults::{detects as detects_fault, preflight_cases};
use super::super::types::{
    CPU_SCALE_V2 as S, CpuCandidateV2, CpuCaseV2, CpuCommandV2, CpuFaultKindV2, CpuRasterV2,
    CpuResultV2, CpuRunV2, CpuTransformV2,
};

pub(crate) fn run(cases: &[CpuCaseV2]) -> CpuResultV2<CpuRunV2> {
    preflight_cases(cases)?;
    let cases = cases
        .iter()
        .map(render_case)
        .collect::<CpuResultV2<Vec<_>>>()?;
    Ok(CpuRunV2 {
        candidate: Some(CpuCandidateV2::TinySkia),
        cases,
    })
}

pub(crate) fn detects(kind: CpuFaultKindV2) -> bool {
    detects_fault(kind)
}

fn render_case(case: &CpuCaseV2) -> CpuResultV2<CpuRasterV2> {
    let mut pixmap = Pixmap::new(case.width, case.height).ok_or(CpuFaultKindV2::PixelLimit)?;
    for command in &case.commands {
        render_command(case, command, &mut pixmap)?;
    }
    Ok(CpuRasterV2 {
        ordinal: case.ordinal,
        width: case.width,
        height: case.height,
        bytes: pixmap.take(),
    })
}

fn render_command(
    case: &CpuCaseV2,
    command: &CpuCommandV2,
    pixmap: &mut Pixmap,
) -> CpuResultV2<()> {
    match command {
        CpuCommandV2::SolidRect {
            rect,
            color,
            transform,
            clip,
        } => {
            let mut paint = solid_paint(*color);
            paint.anti_alias = false;
            let mask = clip.map(|bounds| clip_mask(case, bounds));
            pixmap.fill_rect(
                to_rect(*rect),
                &paint,
                to_transform(*transform),
                mask.as_ref(),
            );
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
            let path = builder.finish().ok_or(CpuFaultKindV2::PixelLimit)?;
            let mut paint = solid_paint(*color);
            paint.anti_alias = false;
            let mask = clip.map(|bounds| clip_mask(case, bounds));
            pixmap.fill_path(
                &path,
                &paint,
                FillRule::Winding,
                to_transform(*transform),
                mask.as_ref(),
            );
        }
        CpuCommandV2::LinearGradient {
            rect,
            start,
            end,
            colors,
        } => {
            let stops = vec![
                GradientStop::new(0.0, to_color(colors[0])),
                GradientStop::new(1.0, to_color(colors[1])),
            ];
            let shader = LinearGradient::new(
                Point::from_xy(scalar(start[0]), scalar(start[1])),
                Point::from_xy(scalar(end[0]), scalar(end[1])),
                stops,
                SpreadMode::Pad,
                Transform::identity(),
            )
            .ok_or(CpuFaultKindV2::PixelLimit)?;
            let paint = Paint {
                shader,
                anti_alias: false,
                ..Paint::default()
            };
            pixmap.fill_rect(to_rect(*rect), &paint, Transform::identity(), None);
        }
        CpuCommandV2::Image {
            origin,
            width,
            height,
            premultiplied_rgba,
            ..
        } => {
            let size = IntSize::from_wh(*width, *height).ok_or(CpuFaultKindV2::ZeroDimension)?;
            let source = Pixmap::from_vec(premultiplied_rgba.clone(), size)
                .ok_or(CpuFaultKindV2::InvalidImageStride)?;
            pixmap.draw_pixmap(
                origin[0] / S,
                origin[1] / S,
                source.as_ref(),
                &PixmapPaint::default(),
                Transform::identity(),
                None,
            );
        }
    }
    Ok(())
}

fn solid_paint(color: [u8; 4]) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.set_color_rgba8(color[0], color[1], color[2], color[3]);
    paint
}

fn to_color(color: [u8; 4]) -> Color {
    Color::from_rgba8(color[0], color[1], color[2], color[3])
}

fn to_rect(rect: [i32; 4]) -> Rect {
    Rect::from_ltrb(
        scalar(rect[0]),
        scalar(rect[1]),
        scalar(rect[2]),
        scalar(rect[3]),
    )
    .expect("registered nonempty CPU rectangle")
}

fn to_transform(transform: CpuTransformV2) -> Transform {
    let [a, b, c, d, tx, ty] = transform.coefficients;
    Transform::from_row(
        scalar(a),
        scalar(b),
        scalar(c),
        scalar(d),
        scalar(tx),
        scalar(ty),
    )
}

fn clip_mask(case: &CpuCaseV2, clip: [i32; 4]) -> Mask {
    let mut data = vec![0; case.width as usize * case.height as usize];
    for y in 0..case.height {
        for x in 0..case.width {
            let point = [x as i32 * S + S / 2, y as i32 * S + S / 2];
            if point[0] >= clip[0]
                && point[0] < clip[2]
                && point[1] >= clip[1]
                && point[1] < clip[3]
            {
                data[y as usize * case.width as usize + x as usize] = 255;
            }
        }
    }
    let size = IntSize::from_wh(case.width, case.height).expect("registered CPU mask size");
    Mask::from_vec(data, size).expect("registered CPU mask length")
}

fn scalar(value: i32) -> f32 {
    value as f32 / S as f32
}
