use super::super::error::{ArtifactDecodeError, ArtifactDecodeErrorKind, SectionKind, VersionKind};
use super::super::primitive::property_value_shape;
use super::super::scan::ScannedLine;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum MarkerRoleV1 {
    Singleton,
    Begin,
    End,
}

#[derive(Clone, Copy)]
pub(super) enum RecordKindV1 {
    Marker {
        section: SectionKind,
        role: MarkerRoleV1,
    },
    Case(CaseRecordRoleV1),
    Trace,
}

#[derive(Clone, Copy)]
pub(super) enum CaseRecordRoleV1 {
    Transaction { explicitly_empty: bool },
    Operation,
}

pub(super) fn classify_line_v1(
    line: &ScannedLine<'_>,
) -> Result<RecordKindV1, ArtifactDecodeError> {
    let fields = line.text.split('|').collect::<Vec<_>>();
    match fields.first().copied() {
        Some("fenestra-oracle-failure") => classify_header(&fields, line.number),
        Some("versions") => classify_versions(&fields, line.number),
        Some("fixture") => match fields.as_slice() {
            ["fixture", "runtime-oracle", _, _, _, _, _] => Ok(singleton(SectionKind::Fixture)),
            _ => Err(malformed(line.number)),
        },
        Some("replay") if fields.len() == 7 => Ok(singleton(SectionKind::Replay)),
        Some("generator") if fields.len() == 4 => Ok(singleton(SectionKind::Generator)),
        Some("seed") if fields.len() == 2 => Ok(singleton(SectionKind::Seed)),
        Some("original-begin") if fields.len() == 4 => Ok(begin_marker(SectionKind::Original)),
        Some("original-end") if fields.len() == 1 => Ok(end_marker(SectionKind::Original)),
        Some("fault") => match fields.as_slice() {
            ["fault", "omit-move", _] => Ok(singleton(SectionKind::Fault)),
            _ => Err(malformed(line.number)),
        },
        Some("failure") => classify_failure(&fields, line.number),
        Some("reducer") => match fields.as_slice() {
            ["reducer", _, _, "fixed-point" | "budget-exhausted"] => {
                Ok(singleton(SectionKind::Reducer))
            }
            _ => Err(malformed(line.number)),
        },
        Some("minimized-begin") if fields.len() == 4 => Ok(begin_marker(SectionKind::Minimized)),
        Some("minimized-end") if fields.len() == 1 => Ok(end_marker(SectionKind::Minimized)),
        Some("trace-begin") if fields.len() == 3 => Ok(begin_marker(SectionKind::Trace)),
        Some("trace-end") if fields.len() == 1 => Ok(end_marker(SectionKind::Trace)),
        Some("end") if fields.len() == 1 => Ok(singleton(SectionKind::End)),
        Some("tx") if fields.len() == 3 => Ok(RecordKindV1::Case(CaseRecordRoleV1::Transaction {
            explicitly_empty: fields[2] == "0",
        })),
        Some("op") => classify_operation(&fields, line.number),
        Some("event") => classify_event(&fields, line.number),
        _ => Err(malformed(line.number)),
    }
}

fn classify_header(fields: &[&str], line: u32) -> Result<RecordKindV1, ArtifactDecodeError> {
    let ["fenestra-oracle-failure", version] = fields else {
        return Err(malformed(line));
    };
    reject_future_version(version, VersionKind::Envelope, line)?;
    Ok(singleton(SectionKind::Header))
}

fn classify_versions(fields: &[&str], line: u32) -> Result<RecordKindV1, ArtifactDecodeError> {
    let [
        "versions",
        "fixture",
        fixture,
        "generator",
        generator,
        "case",
        case,
        "state",
        state,
        "trace",
        trace,
        "fingerprint",
        fingerprint,
        "reducer",
        reducer,
    ] = fields
    else {
        return Err(malformed(line));
    };
    for (version, kind) in [
        (*fixture, VersionKind::Fixture),
        (*generator, VersionKind::Generator),
        (*case, VersionKind::Case),
        (*state, VersionKind::State),
        (*trace, VersionKind::Trace),
        (*fingerprint, VersionKind::Fingerprint),
        (*reducer, VersionKind::Reducer),
    ] {
        reject_future_version(version, kind, line)?;
    }
    Ok(singleton(SectionKind::Versions))
}

fn reject_future_version(
    version: &str,
    kind: VersionKind,
    line: u32,
) -> Result<(), ArtifactDecodeError> {
    if version.parse::<u32>().is_ok_and(|value| value != 1) {
        return Err(ArtifactDecodeError::at(
            ArtifactDecodeErrorKind::UnsupportedVersion(kind),
            line,
        ));
    }
    Ok(())
}

fn classify_failure(fields: &[&str], line: u32) -> Result<RecordKindV1, ArtifactDecodeError> {
    let [
        "failure",
        scope,
        _,
        _,
        fingerprint,
        location,
        field,
        expected,
        observed,
    ] = fields
    else {
        return Err(malformed(line));
    };
    if !matches!(
        *fingerprint,
        "candidate-rejected" | "state-mismatch" | "identity-mismatch"
    ) {
        return Err(malformed(line));
    }
    if !valid_location_shape(location)
        || !valid_fingerprint_field(field)
        || !valid_summary_shape(expected, line)?
        || !valid_summary_shape(observed, line)?
    {
        return Err(malformed(line));
    }
    match *scope {
        "original" => Ok(singleton(SectionKind::OriginalFailure)),
        "minimized" => Ok(singleton(SectionKind::MinimizedFailure)),
        _ => Err(malformed(line)),
    }
}

fn classify_operation(fields: &[&str], line: u32) -> Result<RecordKindV1, ArtifactDecodeError> {
    match fields {
        ["op", _, "set", _, _, value] => property_value_shape(value, line)?,
        ["op", _, "update", _, _, _, value] => property_value_shape(value, line)?,
        ["op", _, "insert" | "move", _, _, _] | ["op", _, "remove", _, _] => {}
        _ => return Err(malformed(line)),
    }
    Ok(RecordKindV1::Case(CaseRecordRoleV1::Operation))
}

fn classify_event(fields: &[&str], line: u32) -> Result<RecordKindV1, ArtifactDecodeError> {
    let ["event", _, _, _, _, _, outcome, _, invalidation, comparison] = fields else {
        return Err(malformed(line));
    };
    if !valid_outcome(outcome)
        || !valid_invalidation_shape(invalidation)
        || !matches!(*comparison, "match" | "mismatch")
    {
        return Err(malformed(line));
    }
    Ok(RecordKindV1::Trace)
}

fn valid_outcome(value: &str) -> bool {
    if matches!(value, "commit" | "noop") {
        return true;
    }
    value
        .strip_prefix("reject:")
        .is_some_and(valid_rejection_code)
}

fn valid_rejection_code(value: &str) -> bool {
    matches!(
        value,
        "capacity-operations"
            | "capacity-structural"
            | "capacity-live-nodes"
            | "capacity-live-fragments"
            | "capacity-live-properties"
            | "capacity-retained-generations"
            | "stale-base"
            | "missing-node"
            | "missing-fragment"
            | "missing-key"
            | "duplicate-key"
            | "unknown-property"
            | "property-type-mismatch"
            | "index-out-of-bounds"
            | "generation-exhausted"
            | "invariant-violation"
    )
}

fn valid_location_shape(value: &str) -> bool {
    value == "global" || value.starts_with("node:") || value.starts_with("fragment:")
}

fn valid_fingerprint_field(value: &str) -> bool {
    matches!(
        value,
        "candidate-outcome"
            | "template"
            | "component"
            | "property"
            | "parent"
            | "child-order"
            | "fragment-binding"
            | "keyed-order"
            | "node-count"
            | "fragment-count"
            | "property-count"
            | "identity-lifecycle"
    )
}

fn valid_summary_shape(value: &str, line: u32) -> Result<bool, ArtifactDecodeError> {
    if matches!(
        value,
        "none"
            | "binding:present"
            | "binding:absent"
            | "kind:accept"
            | "absent"
            | "preserved"
            | "fresh"
            | "retired"
            | "distinct"
            | "aliased"
    ) {
        return Ok(true);
    }
    let Some((kind, payload)) = value.split_once(':') else {
        return Ok(false);
    };
    match kind {
        "count" | "template" | "component" | "node" | "nodes" | "keys" => Ok(true),
        "property" => {
            let Some((_, value)) = payload.split_once(':') else {
                return Ok(false);
            };
            property_value_shape(value, line).map(|()| true)
        }
        "children" => Ok(payload == "-"
            || payload.split(',').all(|entry| {
                entry.is_empty() || entry.starts_with("s:") || entry.starts_with("r:")
            })),
        "kind" => Ok(valid_rejection_code(payload)),
        "binding" => Ok(false),
        _ => Ok(false),
    }
}

fn valid_invalidation_shape(value: &str) -> bool {
    value.is_empty() || value == "-" || value.split(',').all(valid_invalidation_word)
}

fn valid_invalidation_word(value: &str) -> bool {
    matches!(
        value,
        "structure"
            | "style-match"
            | "intrinsic"
            | "layout"
            | "semantics"
            | "hit-test"
            | "paint"
            | "composition"
            | "surface"
    )
}

const fn singleton(section: SectionKind) -> RecordKindV1 {
    RecordKindV1::Marker {
        section,
        role: MarkerRoleV1::Singleton,
    }
}

const fn begin_marker(section: SectionKind) -> RecordKindV1 {
    RecordKindV1::Marker {
        section,
        role: MarkerRoleV1::Begin,
    }
}

const fn end_marker(section: SectionKind) -> RecordKindV1 {
    RecordKindV1::Marker {
        section,
        role: MarkerRoleV1::End,
    }
}

fn malformed(line: u32) -> ArtifactDecodeError {
    ArtifactDecodeError::at(ArtifactDecodeErrorKind::MalformedRecord, line)
}
