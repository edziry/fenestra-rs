use super::types::{
    CPU_SCALE_V2 as S, CpuCaseV2, CpuCommandV2 as Command, CpuObligationV2 as Obligation,
    CpuSamplingV2, CpuTransformV2,
};

pub(crate) fn cpu_cases_v2() -> Vec<CpuCaseV2> {
    vec![
        case(
            0,
            "opaque-control",
            vec![solid_rect([1, 1, 7, 7], [220, 40, 20, 255], None)],
            vec![Obligation::OpaqueControl, Obligation::Shape],
        ),
        case(
            1,
            "alpha-source-over",
            vec![
                solid_rect([1, 1, 6, 6], [20, 80, 220, 255], None),
                solid_rect([3, 2, 8, 7], [240, 160, 20, 128], None),
            ],
            vec![Obligation::RichReference, Obligation::Alpha],
        ),
        case(
            2,
            "transformed-polygon",
            vec![Command::SolidPolygon {
                points: vec![point(0, 0), point(5, 1), point(2, 6)],
                color: [40, 210, 100, 255],
                transform: CpuTransformV2 {
                    coefficients: [S, 0, 0, S, 2 * S, S],
                },
                clip: None,
            }],
            vec![Obligation::Transform],
        ),
        case(
            3,
            "rectangular-clip",
            vec![solid_rect(
                [0, 0, 8, 8],
                [180, 30, 210, 255],
                Some([2, 1, 7, 6]),
            )],
            vec![Obligation::Clip],
        ),
        case(
            4,
            "linear-gradient",
            vec![Command::LinearGradient {
                rect: fixed_rect([0, 1, 8, 7]),
                start: point(0, 0),
                end: point(8, 0),
                colors: [[240, 20, 20, 255], [20, 40, 240, 128]],
            }],
            vec![Obligation::Gradient],
        ),
        case(
            5,
            "nearest-image",
            vec![Command::Image {
                origin: point(3, 3),
                width: 2,
                height: 2,
                stride: 8,
                premultiplied_rgba: vec![
                    255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
                ],
                sampling: CpuSamplingV2::Nearest,
            }],
            vec![Obligation::Image],
        ),
    ]
}

fn case(
    ordinal: u8,
    name: &'static str,
    commands: Vec<Command>,
    obligations: Vec<Obligation>,
) -> CpuCaseV2 {
    CpuCaseV2 {
        ordinal,
        name,
        width: 8,
        height: 8,
        commands,
        obligations,
    }
}

fn solid_rect(rect: [i32; 4], color: [u8; 4], clip: Option<[i32; 4]>) -> Command {
    Command::SolidRect {
        rect: fixed_rect(rect),
        color,
        transform: CpuTransformV2::IDENTITY,
        clip: clip.map(fixed_rect),
    }
}

fn fixed_rect(rect: [i32; 4]) -> [i32; 4] {
    [rect[0] * S, rect[1] * S, rect[2] * S, rect[3] * S]
}

const fn point(x: i32, y: i32) -> [i32; 2] {
    [x * S, y * S]
}
