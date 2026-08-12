use super::{ArtifactErrorKindV2, ArtifactErrorV2};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GrammarValueKindV2 {
    Token,
    List,
    Unsigned,
    Signed,
    Hex16,
    Hex64,
    Absent,
}

pub(crate) fn grammar_value_accepts_v2(kind: GrammarValueKindV2, value: &str) -> bool {
    match kind {
        GrammarValueKindV2::Token => token(value),
        GrammarValueKindV2::List => !value.is_empty() && value.split(',').all(token),
        GrammarValueKindV2::Unsigned => unsigned(value),
        GrammarValueKindV2::Signed => signed(value),
        GrammarValueKindV2::Hex16 => lowercase_hex(value, 16),
        GrammarValueKindV2::Hex64 => lowercase_hex(value, 64),
        GrammarValueKindV2::Absent => value == "-",
    }
}

pub(super) fn token(value: &str) -> bool {
    let mut bytes = value.bytes();
    matches!(bytes.next(), Some(byte) if byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && bytes.all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'.' | b'_' | b':' | b'-')
        })
}

pub(super) fn unsigned(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && (value == "0" || !value.starts_with('0'))
}

fn signed(value: &str) -> bool {
    if let Some(rest) = value.strip_prefix('-') {
        rest != "0" && unsigned(rest)
    } else {
        unsigned(value)
    }
}

fn lowercase_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(super) fn validate_record_grammar(line: &str, record: usize) -> Result<(), ArtifactErrorV2> {
    if line.is_empty() || !line.bytes().all(|byte| (0x20..=0x7e).contains(&byte)) {
        return Err(ArtifactErrorV2::at(
            ArtifactErrorKindV2::InvalidGrammar,
            record,
        ));
    }
    let mut parts = line.split('|');
    let name = parts.next().unwrap_or_default();
    if !token(name) {
        return Err(ArtifactErrorV2::at(
            ArtifactErrorKindV2::InvalidGrammar,
            record,
        ));
    }
    if name == "end" {
        return (parts.next() == Some("spatial-v2") && parts.next().is_none())
            .then_some(())
            .ok_or_else(|| ArtifactErrorV2::at(ArtifactErrorKindV2::InvalidGrammar, record));
    }
    for field in parts {
        let Some((key, value)) = field.split_once('=') else {
            return Err(ArtifactErrorV2::at(
                ArtifactErrorKindV2::InvalidGrammar,
                record,
            ));
        };
        if !token(key) || value.is_empty() || value.contains('=') {
            return Err(ArtifactErrorV2::at(
                ArtifactErrorKindV2::InvalidGrammar,
                record,
            ));
        }
    }
    Ok(())
}

pub(crate) fn host_token_probe_v2(value: &str) -> Result<(), ArtifactErrorV2> {
    let lower = value.to_ascii_lowercase();
    let forbidden = [
        "/home/",
        "\\",
        "hostname",
        "duration",
        "thread-id",
        "process-id",
        "pointer",
        "runtime-id",
        "native-handle",
        "gpu-device",
        "driver-string",
        "environment=",
        "panic",
        "debug",
        "source-payload",
        "target/debug",
        "0x",
    ];
    if forbidden.iter().any(|token| lower.contains(token)) {
        Err(ArtifactErrorV2::new(ArtifactErrorKindV2::InvalidModel))
    } else {
        Ok(())
    }
}
