use super::candidates::{raqote_detects, tiny_skia_detects};
use super::types::{
    CPU_PIXEL_LIMIT_V2, CPU_SCALE_V2, CpuCaseV2, CpuCommandV2, CpuFaultKindV2, CpuFaultV2,
    CpuResultV2, CpuSamplingV2,
};

pub(crate) fn cpu_faults_v2() -> Vec<CpuFaultV2> {
    [
        CpuFaultKindV2::ZeroDimension,
        CpuFaultKindV2::PixelLimit,
        CpuFaultKindV2::InvalidImageStride,
        CpuFaultKindV2::NonFiniteTransform,
        CpuFaultKindV2::UnsupportedSampling,
    ]
    .into_iter()
    .map(|kind| CpuFaultV2 {
        kind,
        literal: detects(kind),
        tiny_skia: tiny_skia_detects(kind),
        raqote: raqote_detects(kind),
    })
    .collect()
}

pub(super) fn preflight_cases(cases: &[CpuCaseV2]) -> CpuResultV2<()> {
    for case in cases {
        validate_dimensions(case.width, case.height)?;
        for command in &case.commands {
            match command {
                CpuCommandV2::SolidRect { transform, .. }
                | CpuCommandV2::SolidPolygon { transform, .. } => {
                    let coefficients = transform
                        .coefficients
                        .map(|value| value as f32 / CPU_SCALE_V2 as f32);
                    validate_transform(coefficients)?;
                }
                CpuCommandV2::Image {
                    width,
                    height,
                    stride,
                    premultiplied_rgba,
                    sampling,
                    ..
                } => {
                    validate_image(*width, *height, *stride, premultiplied_rgba.len())?;
                    validate_sampling(*sampling)?;
                }
                CpuCommandV2::LinearGradient { .. } => {}
            }
        }
    }
    Ok(())
}

pub(super) fn detects(kind: CpuFaultKindV2) -> bool {
    match kind {
        CpuFaultKindV2::ZeroDimension => validate_dimensions(0, 8).is_err(),
        CpuFaultKindV2::PixelLimit => validate_dimensions(65, 65).is_err(),
        CpuFaultKindV2::InvalidImageStride => validate_image(2, 2, 7, 16).is_err(),
        CpuFaultKindV2::NonFiniteTransform => {
            validate_transform([f32::NAN, 0.0, 0.0, 1.0, 0.0, 0.0]).is_err()
        }
        CpuFaultKindV2::UnsupportedSampling => validate_sampling(CpuSamplingV2::Bilinear).is_err(),
    }
}

fn validate_dimensions(width: u32, height: u32) -> CpuResultV2<()> {
    if width == 0 || height == 0 {
        return Err(CpuFaultKindV2::ZeroDimension);
    }
    let pixels = width as usize * height as usize;
    if pixels > CPU_PIXEL_LIMIT_V2 {
        return Err(CpuFaultKindV2::PixelLimit);
    }
    Ok(())
}

fn validate_image(width: u32, height: u32, stride: u32, byte_len: usize) -> CpuResultV2<()> {
    let expected_stride = width
        .checked_mul(4)
        .ok_or(CpuFaultKindV2::InvalidImageStride)?;
    let expected_len = stride
        .checked_mul(height)
        .map(|value| value as usize)
        .ok_or(CpuFaultKindV2::InvalidImageStride)?;
    if stride != expected_stride || byte_len != expected_len {
        return Err(CpuFaultKindV2::InvalidImageStride);
    }
    Ok(())
}

fn validate_transform(coefficients: [f32; 6]) -> CpuResultV2<()> {
    if coefficients.into_iter().all(f32::is_finite) {
        Ok(())
    } else {
        Err(CpuFaultKindV2::NonFiniteTransform)
    }
}

fn validate_sampling(sampling: CpuSamplingV2) -> CpuResultV2<()> {
    if sampling == CpuSamplingV2::Nearest {
        Ok(())
    } else {
        Err(CpuFaultKindV2::UnsupportedSampling)
    }
}
