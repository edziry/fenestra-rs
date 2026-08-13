use crate::GpuTargetV1;

use super::InteractiveArtifactErrorKindV1;

pub(super) fn fields<'a>(
    line: Option<&'a str>,
    record: &str,
) -> Result<Vec<(&'a str, &'a str)>, InteractiveArtifactErrorKindV1> {
    let mut parts = line
        .ok_or(InteractiveArtifactErrorKindV1::Grammar)?
        .split('|');
    if parts.next() != Some(record) {
        return Err(InteractiveArtifactErrorKindV1::Grammar);
    }
    let mut output: Vec<(&str, &str)> = Vec::new();
    for part in parts {
        let (key, value) = part
            .split_once('=')
            .ok_or(InteractiveArtifactErrorKindV1::Grammar)?;
        if key.is_empty()
            || value.is_empty()
            || value.contains('=')
            || output.iter().any(|field| field.0 == key)
        {
            return Err(InteractiveArtifactErrorKindV1::Grammar);
        }
        output.push((key, value));
    }
    Ok(output)
}

pub(super) fn exact_keys(
    fields: &[(&str, &str)],
    expected: &[&str],
) -> Result<(), InteractiveArtifactErrorKindV1> {
    if fields.len() != expected.len()
        || fields
            .iter()
            .zip(expected)
            .any(|(field, expected)| field.0 != *expected)
    {
        return Err(InteractiveArtifactErrorKindV1::Grammar);
    }
    Ok(())
}

pub(super) fn parse_target(value: &str) -> Result<GpuTargetV1, InteractiveArtifactErrorKindV1> {
    match value {
        "windows-dx12" => Ok(GpuTargetV1::WindowsDx12),
        "linux-vulkan" => Ok(GpuTargetV1::LinuxVulkan),
        _ => Err(InteractiveArtifactErrorKindV1::Grammar),
    }
}

pub(super) fn parse_u32(value: &str) -> Result<u32, InteractiveArtifactErrorKindV1> {
    value
        .parse()
        .map_err(|_| InteractiveArtifactErrorKindV1::Grammar)
}

pub(super) fn parse_u64(value: &str) -> Result<u64, InteractiveArtifactErrorKindV1> {
    value
        .parse()
        .map_err(|_| InteractiveArtifactErrorKindV1::Grammar)
}

pub(super) fn parse_extent(value: &str) -> Result<(u32, u32), InteractiveArtifactErrorKindV1> {
    let (width, height) = value
        .split_once('x')
        .ok_or(InteractiveArtifactErrorKindV1::Grammar)?;
    let extent = (parse_u32(width)?, parse_u32(height)?);
    if extent.0 == 0 || extent.1 == 0 {
        return Err(InteractiveArtifactErrorKindV1::Protocol);
    }
    Ok(extent)
}

pub(super) fn parse_digest(value: &str) -> Result<u64, InteractiveArtifactErrorKindV1> {
    if value.len() != 16 || value.bytes().any(|byte| !byte.is_ascii_hexdigit()) {
        return Err(InteractiveArtifactErrorKindV1::Grammar);
    }
    u64::from_str_radix(value, 16).map_err(|_| InteractiveArtifactErrorKindV1::Grammar)
}

pub(super) fn require_hex(value: &str) -> Result<(), InteractiveArtifactErrorKindV1> {
    if !value.len().is_multiple_of(2) || value.bytes().any(|byte| !byte.is_ascii_hexdigit()) {
        return Err(InteractiveArtifactErrorKindV1::Grammar);
    }
    Ok(())
}
