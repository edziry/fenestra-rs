use super::candidates::vello_detects;
use super::types::{
    NATIVE_RESOURCE_LIMIT_V2, NativeCaseV2, NativeCommandV2, NativeFaultKindV2, NativeFaultV2,
    NativeResultV2,
};

pub(crate) fn native_faults_v2() -> Vec<NativeFaultV2> {
    [
        NativeFaultKindV2::MissingCapability,
        NativeFaultKindV2::ZeroSurface,
        NativeFaultKindV2::ResourceLimit,
        NativeFaultKindV2::AdapterUnavailable,
        NativeFaultKindV2::SurfaceLost,
    ]
    .into_iter()
    .map(|kind| NativeFaultV2 {
        kind,
        literal: detects(kind),
        vello: vello_detects(kind),
    })
    .collect()
}

pub(super) fn preflight_cases(cases: &[NativeCaseV2]) -> NativeResultV2<()> {
    validate_capabilities(true, true)?;
    for case in cases {
        validate_surface(case.width, case.height)?;
        let mut resource_bytes = 0_usize;
        for command in &case.commands {
            if let NativeCommandV2::Image {
                width,
                height,
                rgba8,
                ..
            } = command
            {
                let expected = *width as usize * *height as usize * 4;
                if rgba8.len() != expected {
                    return Err(NativeFaultKindV2::ResourceLimit);
                }
                resource_bytes = resource_bytes
                    .checked_add(rgba8.len())
                    .ok_or(NativeFaultKindV2::ResourceLimit)?;
            }
        }
        validate_resources(resource_bytes)?;
    }
    Ok(())
}

pub(super) fn detects(kind: NativeFaultKindV2) -> bool {
    match kind {
        NativeFaultKindV2::MissingCapability => validate_capabilities(false, true).is_err(),
        NativeFaultKindV2::ZeroSurface => validate_surface(0, 24).is_err(),
        NativeFaultKindV2::ResourceLimit => {
            validate_resources(NATIVE_RESOURCE_LIMIT_V2 + 1).is_err()
        }
        NativeFaultKindV2::AdapterUnavailable => adapter_status(false).is_err(),
        NativeFaultKindV2::SurfaceLost => surface_status(false).is_err(),
    }
}

fn validate_capabilities(vector: bool, images: bool) -> NativeResultV2<()> {
    if vector && images {
        Ok(())
    } else {
        Err(NativeFaultKindV2::MissingCapability)
    }
}

fn validate_surface(width: u32, height: u32) -> NativeResultV2<()> {
    if width == 0 || height == 0 {
        Err(NativeFaultKindV2::ZeroSurface)
    } else {
        Ok(())
    }
}

fn validate_resources(bytes: usize) -> NativeResultV2<()> {
    if bytes > NATIVE_RESOURCE_LIMIT_V2 {
        Err(NativeFaultKindV2::ResourceLimit)
    } else {
        Ok(())
    }
}

fn adapter_status(available: bool) -> NativeResultV2<()> {
    available
        .then_some(())
        .ok_or(NativeFaultKindV2::AdapterUnavailable)
}

fn surface_status(available: bool) -> NativeResultV2<()> {
    available
        .then_some(())
        .ok_or(NativeFaultKindV2::SurfaceLost)
}
