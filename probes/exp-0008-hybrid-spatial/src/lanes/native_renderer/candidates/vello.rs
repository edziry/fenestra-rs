use std::mem::size_of_val;

use vello::kurbo::{Affine, Rect};
use vello::peniko::{Color, Fill, Gradient, ImageAlphaType, ImageBrush, ImageData, ImageFormat};

use super::super::faults::{detects as detects_fault, preflight_cases};
use super::super::types::{
    NATIVE_SCALE_V2 as S, NativeCandidateV2, NativeCaseV2, NativeCommandV2, NativeFaultKindV2,
    NativeRecordV2, NativeResultV2, NativeRunV2, NativeTransformV2,
};

pub(crate) fn run(cases: &[NativeCaseV2]) -> NativeResultV2<NativeRunV2> {
    preflight_cases(cases)?;
    let backend_policy = wgpu::Backends::VULKAN | wgpu::Backends::DX12;
    debug_assert!(!backend_policy.is_empty());
    let mut scene_fingerprint = 14_695_981_039_346_656_037;
    let mut encoded_scene_bytes = 0;
    let mut records = Vec::with_capacity(cases.len());
    for case in cases {
        let mut scene = vello::Scene::new();
        for command in &case.commands {
            encode(command, &mut scene);
        }
        let encoding = scene.encoding();
        for tag in &encoding.path_tags {
            scene_fingerprint = fold_u32(scene_fingerprint, u32::from(tag.0));
        }
        for value in &encoding.path_data {
            scene_fingerprint = fold_u32(scene_fingerprint, *value);
        }
        for tag in &encoding.draw_tags {
            scene_fingerprint = fold_u32(scene_fingerprint, tag.0);
        }
        for value in &encoding.draw_data {
            scene_fingerprint = fold_u32(scene_fingerprint, *value);
        }
        for transform in &encoding.transforms {
            for value in transform.matrix.into_iter().chain(transform.translation) {
                scene_fingerprint = fold_u32(scene_fingerprint, value.to_bits());
            }
        }
        for style in &encoding.styles {
            scene_fingerprint = fold_u32(scene_fingerprint, style.flags_and_miter_limit);
            scene_fingerprint = fold_u32(scene_fingerprint, style.line_width.to_bits());
        }
        scene_fingerprint = fold_u32(scene_fingerprint, encoding.n_paths);
        scene_fingerprint = fold_u32(scene_fingerprint, encoding.n_path_segments);
        scene_fingerprint = fold_u32(scene_fingerprint, encoding.n_clips);
        scene_fingerprint = fold_u32(scene_fingerprint, encoding.resources.patches.len() as u32);
        scene_fingerprint = fold_u32(
            scene_fingerprint,
            encoding.resources.color_stops.len() as u32,
        );
        encoded_scene_bytes += size_of_val(encoding.path_tags.as_slice())
            + size_of_val(encoding.path_data.as_slice())
            + size_of_val(encoding.draw_tags.as_slice())
            + size_of_val(encoding.draw_data.as_slice())
            + size_of_val(encoding.transforms.as_slice())
            + size_of_val(encoding.styles.as_slice())
            + size_of_val(encoding.resources.patches.as_slice())
            + size_of_val(encoding.resources.color_stops.as_slice());
        records.push(record_from_case(case));
    }
    Ok(NativeRunV2 {
        candidate: Some(NativeCandidateV2::Vello),
        records,
        scene_fingerprint,
        encoded_scene_bytes,
        used_vello_scene: true,
        executed_gpu: false,
    })
}

pub(crate) fn detects(kind: NativeFaultKindV2) -> bool {
    detects_fault(kind)
}

fn encode(command: &NativeCommandV2, scene: &mut vello::Scene) {
    match command {
        NativeCommandV2::SolidRect {
            rect,
            color,
            transform,
        } => scene.fill(
            Fill::NonZero,
            affine(*transform),
            color_value(*color),
            None,
            &rect_value(*rect),
        ),
        NativeCommandV2::GradientRect {
            rect,
            start,
            end,
            colors,
        } => {
            let gradient = Gradient::new_linear(point(*start), point(*end))
                .with_stops([color_value(colors[0]), color_value(colors[1])]);
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &gradient,
                None,
                &rect_value(*rect),
            );
        }
        NativeCommandV2::ClipRect { clip, rect, color } => {
            scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &rect_value(*clip));
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                color_value(*color),
                None,
                &rect_value(*rect),
            );
            scene.pop_layer();
        }
        NativeCommandV2::Image {
            origin,
            width,
            height,
            rgba8,
        } => {
            let image = ImageBrush::new(ImageData {
                data: rgba8.clone().into(),
                format: ImageFormat::Rgba8,
                alpha_type: ImageAlphaType::Alpha,
                width: *width,
                height: *height,
            });
            scene.draw_image(
                &image,
                Affine::translate((scalar(origin[0]), scalar(origin[1]))),
            );
        }
    }
}

fn record_from_case(case: &NativeCaseV2) -> NativeRecordV2 {
    let mut shapes = 0;
    let mut clips = 0;
    let mut images = 0;
    let mut painter_digest = fold_byte(14_695_981_039_346_656_037, case.ordinal);
    for command in &case.commands {
        let tag = match command {
            NativeCommandV2::SolidRect { .. } => {
                shapes += 1;
                0
            }
            NativeCommandV2::GradientRect { .. } => {
                shapes += 1;
                1
            }
            NativeCommandV2::ClipRect { .. } => {
                shapes += 1;
                clips += 1;
                2
            }
            NativeCommandV2::Image { .. } => {
                images += 1;
                3
            }
        };
        painter_digest = fold_byte(painter_digest, tag);
    }
    NativeRecordV2 {
        ordinal: case.ordinal,
        width: case.width,
        height: case.height,
        commands: case.commands.len() as u32,
        shapes,
        clips,
        images,
        painter_digest,
    }
}

fn affine(transform: NativeTransformV2) -> Affine {
    Affine::new(transform.coefficients.map(scalar))
}

fn rect_value(rect: [i32; 4]) -> Rect {
    Rect::new(
        scalar(rect[0]),
        scalar(rect[1]),
        scalar(rect[2]),
        scalar(rect[3]),
    )
}

fn point(point: [i32; 2]) -> (f64, f64) {
    (scalar(point[0]), scalar(point[1]))
}

fn color_value(color: [u8; 4]) -> Color {
    Color::from_rgba8(color[0], color[1], color[2], color[3])
}

fn scalar(value: i32) -> f64 {
    f64::from(value) / f64::from(S)
}

const fn fold_byte(hash: u64, value: u8) -> u64 {
    (hash ^ value as u64).wrapping_mul(1_099_511_628_211)
}

fn fold_u32(mut hash: u64, value: u32) -> u64 {
    for byte in value.to_le_bytes() {
        hash = fold_byte(hash, byte);
    }
    hash
}
