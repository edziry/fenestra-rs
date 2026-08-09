use std::fmt::Write;

use fenestra_ui_ir::prototype::{InputPolicy, PropertyValue};

use super::error::{ArtifactDecodeError, ArtifactDecodeErrorKind};

pub(crate) fn parse_u16(value: &str, line: u32) -> Result<u16, ArtifactDecodeError> {
    canonical_unsigned(value, line)?
        .parse()
        .map_err(|_| noncanonical(line))
}

pub(crate) fn parse_u32(value: &str, line: u32) -> Result<u32, ArtifactDecodeError> {
    canonical_unsigned(value, line)?
        .parse()
        .map_err(|_| noncanonical(line))
}

pub(crate) fn parse_u64(value: &str, line: u32) -> Result<u64, ArtifactDecodeError> {
    canonical_unsigned(value, line)?
        .parse()
        .map_err(|_| noncanonical(line))
}

pub(crate) fn parse_property_value(
    value: &str,
    line: u32,
) -> Result<PropertyValue, ArtifactDecodeError> {
    let (kind, payload) = value.split_once(':').ok_or_else(|| malformed(line))?;
    match kind {
        "bool" => match payload {
            "false" => Ok(PropertyValue::Bool(false)),
            "true" => Ok(PropertyValue::Bool(true)),
            _ => Err(malformed(line)),
        },
        "i32" => Ok(PropertyValue::ScalarI32(parse_i32(payload, line)?)),
        "rgba8" => Ok(PropertyValue::Rgba8(parse_rgba8(payload, line)?)),
        "input" => match payload {
            "accept" => Ok(PropertyValue::InputPolicy(InputPolicy::Accept)),
            "ignore" => Ok(PropertyValue::InputPolicy(InputPolicy::Ignore)),
            _ => Err(malformed(line)),
        },
        _ => Err(malformed(line)),
    }
}

pub(crate) fn property_value_shape(value: &str, line: u32) -> Result<(), ArtifactDecodeError> {
    let (kind, payload) = value.split_once(':').ok_or_else(|| malformed(line))?;
    match kind {
        "i32" | "rgba8" => Ok(()),
        "bool" if matches!(payload, "false" | "true") => Ok(()),
        "input" if matches!(payload, "accept" | "ignore") => Ok(()),
        _ => Err(malformed(line)),
    }
}

pub(crate) fn write_property_value(output: &mut String, value: &PropertyValue) {
    match value {
        PropertyValue::Bool(false) => output.push_str("bool:false"),
        PropertyValue::Bool(true) => output.push_str("bool:true"),
        PropertyValue::ScalarI32(value) => {
            let _ = write!(output, "i32:{value}");
        }
        PropertyValue::Rgba8(bytes) => {
            output.push_str("rgba8:");
            for byte in bytes {
                let _ = write!(output, "{byte:02x}");
            }
        }
        PropertyValue::InputPolicy(InputPolicy::Accept) => output.push_str("input:accept"),
        PropertyValue::InputPolicy(InputPolicy::Ignore) => output.push_str("input:ignore"),
    }
}

fn canonical_unsigned(value: &str, line: u32) -> Result<&str, ArtifactDecodeError> {
    if value.is_empty()
        || !value.bytes().all(|byte| byte.is_ascii_digit())
        || (value.len() > 1 && value.starts_with('0'))
    {
        return Err(noncanonical(line));
    }
    Ok(value)
}

fn parse_i32(value: &str, line: u32) -> Result<i32, ArtifactDecodeError> {
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

fn parse_rgba8(value: &str, line: u32) -> Result<[u8; 4], ArtifactDecodeError> {
    if value.len() != 8
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(noncanonical(line));
    }
    let mut decoded = [0_u8; 4];
    for (index, slot) in decoded.iter_mut().enumerate() {
        let start = index * 2;
        *slot = u8::from_str_radix(&value[start..start + 2], 16).map_err(|_| noncanonical(line))?;
    }
    Ok(decoded)
}

fn noncanonical(line: u32) -> ArtifactDecodeError {
    ArtifactDecodeError::at(ArtifactDecodeErrorKind::NonCanonicalValue, line)
}

fn malformed(line: u32) -> ArtifactDecodeError {
    ArtifactDecodeError::at(ArtifactDecodeErrorKind::MalformedRecord, line)
}
