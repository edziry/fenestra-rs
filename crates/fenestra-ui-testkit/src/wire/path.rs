use std::fmt::Write;

use crate::semantic::{FragmentPathV1, NodePathV1, PathSegmentV1};

use super::error::{ArtifactDecodeError, ArtifactDecodeErrorKind};
use super::primitive::{parse_u16, parse_u64};

pub(crate) fn parse_node_path(value: &str, line: u32) -> Result<NodePathV1, ArtifactDecodeError> {
    let mut path = NodePathV1::root();
    if value == "root" {
        return Ok(path);
    }
    let remainder = value
        .strip_prefix("root/")
        .ok_or_else(|| noncanonical(line))?;
    if remainder.is_empty() {
        return Err(noncanonical(line));
    }
    for segment in remainder.split('/') {
        let fields: Vec<_> = segment.split(':').collect();
        match fields.as_slice() {
            ["s", slot] => path = path.static_child(parse_u16(slot, line)?),
            ["m", slot, key] => {
                path = path.member(parse_u16(slot, line)?, parse_u64(key, line)?);
            }
            _ => return Err(noncanonical(line)),
        }
    }
    Ok(path)
}

pub(crate) fn parse_fragment_path(
    value: &str,
    line: u32,
) -> Result<FragmentPathV1, ArtifactDecodeError> {
    let (owner, region) = value.rsplit_once("/r:").ok_or_else(|| noncanonical(line))?;
    if owner.is_empty() || region.is_empty() || region.contains(['/', ':']) {
        return Err(noncanonical(line));
    }
    Ok(FragmentPathV1::new(
        parse_node_path(owner, line)?,
        parse_u16(region, line)?,
    ))
}

pub(crate) fn write_node_path(output: &mut String, path: &NodePathV1) {
    output.push_str("root");
    for segment in path.segments() {
        match segment {
            PathSegmentV1::Static { authored_slot } => {
                let _ = write!(output, "/s:{authored_slot}");
            }
            PathSegmentV1::Member { region_slot, key } => {
                let _ = write!(output, "/m:{region_slot}:{key}");
            }
        }
    }
}

pub(crate) fn write_fragment_path(output: &mut String, path: &FragmentPathV1) {
    write_node_path(output, path.owner());
    let _ = write!(output, "/r:{}", path.region_slot());
}

fn noncanonical(line: u32) -> ArtifactDecodeError {
    ArtifactDecodeError::at(ArtifactDecodeErrorKind::NonCanonicalValue, line)
}
