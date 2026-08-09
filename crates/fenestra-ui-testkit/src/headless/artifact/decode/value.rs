use crate::semantic::NodePathV1;

use super::super::error::{
    HeadlessArtifactDecodeErrorKindV1 as ErrorKind, HeadlessArtifactDecodeErrorV1,
};

pub(super) fn parse_u16(value: &str, line: u32) -> Result<u16, HeadlessArtifactDecodeErrorV1> {
    canonical_unsigned(value, line)?
        .parse()
        .map_err(|_| noncanonical(line))
}

pub(super) fn parse_u32(value: &str, line: u32) -> Result<u32, HeadlessArtifactDecodeErrorV1> {
    canonical_unsigned(value, line)?
        .parse()
        .map_err(|_| noncanonical(line))
}

pub(super) fn parse_u64(value: &str, line: u32) -> Result<u64, HeadlessArtifactDecodeErrorV1> {
    canonical_unsigned(value, line)?
        .parse()
        .map_err(|_| noncanonical(line))
}

pub(super) fn parse_usize(value: &str, line: u32) -> Result<usize, HeadlessArtifactDecodeErrorV1> {
    usize::try_from(parse_u64(value, line)?).map_err(|_| noncanonical(line))
}

pub(super) fn parse_i32(value: &str, line: u32) -> Result<i32, HeadlessArtifactDecodeErrorV1> {
    let digits = value.strip_prefix('-').unwrap_or(value);
    if value.starts_with('+')
        || digits.is_empty()
        || !digits.bytes().all(|byte| byte.is_ascii_digit())
        || (digits.len() > 1 && digits.starts_with('0'))
        || value == "-0"
    {
        return Err(noncanonical(line));
    }
    value.parse().map_err(|_| noncanonical(line))
}

pub(super) fn parse_optional_u64(
    value: &str,
    line: u32,
) -> Result<Option<u64>, HeadlessArtifactDecodeErrorV1> {
    if value == "-" {
        Ok(None)
    } else {
        parse_u64(value, line).map(Some)
    }
}

pub(super) fn parse_bool(value: &str, line: u32) -> Result<bool, HeadlessArtifactDecodeErrorV1> {
    match value {
        "false" => Ok(false),
        "true" => Ok(true),
        _ => Err(malformed(line)),
    }
}

pub(super) fn parse_rgba(value: &str, line: u32) -> Result<[u8; 4], HeadlessArtifactDecodeErrorV1> {
    let payload = value
        .strip_prefix("rgba8:")
        .ok_or_else(|| malformed(line))?;
    if payload.len() != 8
        || !payload
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(noncanonical(line));
    }
    let mut output = [0_u8; 4];
    for (index, byte) in output.iter_mut().enumerate() {
        let start = index * 2;
        *byte =
            u8::from_str_radix(&payload[start..start + 2], 16).map_err(|_| noncanonical(line))?;
    }
    Ok(output)
}

pub(super) fn inspect_node_path(
    value: &str,
    line: u32,
) -> Result<usize, HeadlessArtifactDecodeErrorV1> {
    if value == "root" {
        return Ok(0);
    }
    let remainder = value
        .strip_prefix("root/")
        .ok_or_else(|| noncanonical(line))?;
    if remainder.is_empty() {
        return Err(noncanonical(line));
    }
    let mut depth = 0_usize;
    for segment in remainder.split('/') {
        inspect_segment(segment, line)?;
        depth = depth.checked_add(1).ok_or_else(|| noncanonical(line))?;
    }
    Ok(depth)
}

pub(super) fn parse_node_path(
    value: &str,
    line: u32,
) -> Result<NodePathV1, HeadlessArtifactDecodeErrorV1> {
    let _ = inspect_node_path(value, line)?;
    let mut path = NodePathV1::root();
    if value == "root" {
        return Ok(path);
    }
    let remainder = value
        .strip_prefix("root/")
        .ok_or_else(|| noncanonical(line))?;
    for segment in remainder.split('/') {
        let mut fields = segment.split(':');
        match (fields.next(), fields.next(), fields.next(), fields.next()) {
            (Some("s"), Some(slot), None, None) => {
                path = path.static_child(parse_u16(slot, line)?);
            }
            (Some("m"), Some(slot), Some(key), None) => {
                path = path.member(parse_u16(slot, line)?, parse_u64(key, line)?);
            }
            _ => return Err(noncanonical(line)),
        }
    }
    Ok(path)
}

pub(super) fn parse_submission(
    value: &str,
    line: u32,
) -> Result<Option<(u64, u64)>, HeadlessArtifactDecodeErrorV1> {
    if value == "-" {
        return Ok(None);
    }
    let (epoch, token) = value.split_once(':').ok_or_else(|| malformed(line))?;
    if token.contains(':') {
        return Err(malformed(line));
    }
    Ok(Some((parse_u64(epoch, line)?, parse_u64(token, line)?)))
}

fn inspect_segment(segment: &str, line: u32) -> Result<(), HeadlessArtifactDecodeErrorV1> {
    let mut fields = segment.split(':');
    match (fields.next(), fields.next(), fields.next(), fields.next()) {
        (Some("s"), Some(slot), None, None) => {
            let _ = parse_u16(slot, line)?;
            Ok(())
        }
        (Some("m"), Some(slot), Some(key), None) => {
            let _ = parse_u16(slot, line)?;
            let _ = parse_u64(key, line)?;
            Ok(())
        }
        _ => Err(noncanonical(line)),
    }
}

fn canonical_unsigned(value: &str, line: u32) -> Result<&str, HeadlessArtifactDecodeErrorV1> {
    if value.is_empty()
        || !value.bytes().all(|byte| byte.is_ascii_digit())
        || (value.len() > 1 && value.starts_with('0'))
    {
        return Err(noncanonical(line));
    }
    Ok(value)
}

fn noncanonical(line: u32) -> HeadlessArtifactDecodeErrorV1 {
    HeadlessArtifactDecodeErrorV1::at(ErrorKind::NonCanonicalValue, line)
}

fn malformed(line: u32) -> HeadlessArtifactDecodeErrorV1 {
    HeadlessArtifactDecodeErrorV1::at(ErrorKind::MalformedRecord, line)
}
