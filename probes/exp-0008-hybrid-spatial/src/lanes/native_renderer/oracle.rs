use super::faults::preflight_cases;
use super::types::{NativeCaseV2, NativeCommandV2, NativeRecordV2, NativeResultV2, NativeRunV2};

pub(crate) fn literal_native_run_v2(cases: &[NativeCaseV2]) -> NativeResultV2<NativeRunV2> {
    preflight_cases(cases)?;
    Ok(NativeRunV2::literal(
        cases.iter().map(record_from_case).collect(),
    ))
}

fn record_from_case(case: &NativeCaseV2) -> NativeRecordV2 {
    let mut shapes = 0;
    let mut clips = 0;
    let mut images = 0;
    let mut painter_digest = fold(14_695_981_039_346_656_037, case.ordinal);
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
        painter_digest = fold(painter_digest, tag);
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

const fn fold(hash: u64, value: u8) -> u64 {
    (hash ^ value as u64).wrapping_mul(1_099_511_628_211)
}
