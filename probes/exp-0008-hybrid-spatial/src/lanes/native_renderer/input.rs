use super::types::{
    NATIVE_SCALE_V2 as S, NativeCaseV2, NativeCommandV2 as Command,
    NativeObligationV2 as Obligation, NativeTransformV2,
};

pub(crate) fn native_cases_v2() -> Vec<NativeCaseV2> {
    vec![
        NativeCaseV2 {
            ordinal: 0,
            name: "rich-scene",
            width: 32,
            height: 24,
            commands: vec![
                Command::SolidRect {
                    rect: rect(1, 1, 15, 12),
                    color: [220, 40, 30, 180],
                    transform: NativeTransformV2 {
                        coefficients: [S, 0, 0, S, 2 * S, S],
                    },
                },
                Command::GradientRect {
                    rect: rect(4, 3, 22, 16),
                    start: point(4, 3),
                    end: point(22, 3),
                    colors: [[20, 80, 220, 255], [220, 180, 20, 128]],
                },
                Command::ClipRect {
                    clip: rect(8, 4, 28, 20),
                    rect: rect(5, 2, 31, 23),
                    color: [50, 210, 100, 255],
                },
                Command::Image {
                    origin: point(20, 12),
                    width: 2,
                    height: 2,
                    rgba8: vec![
                        255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 128,
                    ],
                },
            ],
            obligations: vec![
                Obligation::Scene,
                Obligation::Shape,
                Obligation::Alpha,
                Obligation::Transform,
                Obligation::Clip,
                Obligation::Gradient,
                Obligation::Image,
                Obligation::PainterOrder,
            ],
        },
        NativeCaseV2 {
            ordinal: 1,
            name: "resized-retained-scene",
            width: 64,
            height: 48,
            commands: vec![Command::SolidRect {
                rect: rect(0, 0, 64, 48),
                color: [12, 24, 36, 255],
                transform: NativeTransformV2::IDENTITY,
            }],
            obligations: vec![
                Obligation::Resize,
                Obligation::ResourceLifetime,
                Obligation::TargetQualification,
            ],
        },
    ]
}

const fn point(x: i32, y: i32) -> [i32; 2] {
    [x * S, y * S]
}

const fn rect(x0: i32, y0: i32, x1: i32, y1: i32) -> [i32; 4] {
    [x0 * S, y0 * S, x1 * S, y1 * S]
}
