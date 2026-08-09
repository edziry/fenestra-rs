#![allow(dead_code)]

use fenestra_ui_testkit::prototype::{
    HeadlessArtifactDecodeErrorKindV1, HeadlessArtifactDecodeErrorV1, HeadlessArtifactV1,
    build_headless_artifact_v1, decode_headless_artifact_v1, encode_headless_artifact_v1,
    run_headless_spine_v1,
};

pub fn canonical_bytes() -> Vec<u8> {
    let run = run_headless_spine_v1().expect("the registered headless run should pass");
    let artifact = build_headless_artifact_v1(&run);
    encode_headless_artifact_v1(&artifact).expect("the registered artifact should encode")
}

pub fn canonical_artifact() -> HeadlessArtifactV1 {
    let run = run_headless_spine_v1().expect("the registered headless run should pass");
    build_headless_artifact_v1(&run)
}

pub fn decode(bytes: &[u8]) -> HeadlessArtifactV1 {
    let decoder: fn(&[u8]) -> Result<HeadlessArtifactV1, HeadlessArtifactDecodeErrorV1> =
        decode_headless_artifact_v1;
    decoder(bytes).expect("structurally valid headless artifact should decode")
}

pub fn decode_error(bytes: &[u8]) -> HeadlessArtifactDecodeErrorV1 {
    decode_headless_artifact_v1(bytes).expect_err("invalid headless artifact should fail")
}

pub fn assert_decode_error(
    bytes: &[u8],
    expected: HeadlessArtifactDecodeErrorKindV1,
    line: Option<u32>,
) {
    let error = decode_error(bytes);
    assert_eq!(error.kind(), expected);
    assert_eq!(error.line(), line);
}

pub fn replace_once(bytes: &[u8], from: &str, to: &str) -> Vec<u8> {
    let text = std::str::from_utf8(bytes).expect("canonical artifact should be ASCII");
    assert_eq!(text.matches(from).count(), 1, "marker must be unique");
    text.replacen(from, to, 1).into_bytes()
}

pub fn remove_line(bytes: &[u8], line: &str) -> Vec<u8> {
    replace_once(bytes, &format!("{line}\n"), "")
}

pub fn duplicate_line(bytes: &[u8], line: &str) -> Vec<u8> {
    replace_once(bytes, &format!("{line}\n"), &format!("{line}\n{line}\n"))
}

pub fn replace_line(bytes: &[u8], prefix: &str, replacement: &str) -> Vec<u8> {
    let text = std::str::from_utf8(bytes).expect("canonical artifact should be ASCII");
    let mut replaced = false;
    let mut output = String::new();
    for line in text.lines() {
        if !replaced && line.starts_with(prefix) {
            output.push_str(replacement);
            replaced = true;
        } else {
            output.push_str(line);
        }
        output.push('\n');
    }
    assert!(replaced, "line prefix must exist: {prefix}");
    output.into_bytes()
}

pub fn lines(bytes: &[u8]) -> Vec<&str> {
    std::str::from_utf8(bytes)
        .expect("artifact should be ASCII")
        .lines()
        .collect()
}

pub fn line_number(bytes: &[u8], prefix: &str) -> u32 {
    let index = lines(bytes)
        .iter()
        .position(|line| line.starts_with(prefix))
        .expect("line prefix should exist");
    u32::try_from(index + 1).expect("artifact line should fit u32")
}

pub fn set_field(bytes: &[u8], prefix: &str, field: usize, value: &str) -> Vec<u8> {
    let source = lines(bytes);
    let index = source
        .iter()
        .position(|line| line.starts_with(prefix))
        .expect("line prefix should exist");
    assert_eq!(
        source
            .iter()
            .filter(|line| line.starts_with(prefix))
            .count(),
        1,
        "line prefix must be unique"
    );
    let mut fields = source[index].split('|').collect::<Vec<_>>();
    assert!(field < fields.len(), "field must exist");
    fields[field] = value;
    let replacement = fields.join("|");
    replace_line(bytes, prefix, &replacement)
}

pub fn replace_section(
    bytes: &[u8],
    begin_prefix: &str,
    end: &str,
    begin: &str,
    records: &[String],
) -> Vec<u8> {
    let source = lines(bytes);
    let first = source
        .iter()
        .position(|line| line.starts_with(begin_prefix))
        .expect("section begin should exist");
    let last = source
        .iter()
        .position(|line| *line == end)
        .expect("section end should exist");
    assert!(first < last);
    let mut output = String::new();
    for line in &source[..first] {
        output.push_str(line);
        output.push('\n');
    }
    output.push_str(begin);
    output.push('\n');
    for record in records {
        output.push_str(record);
        output.push('\n');
    }
    output.push_str(end);
    output.push('\n');
    for line in &source[(last + 1)..] {
        output.push_str(line);
        output.push('\n');
    }
    output.into_bytes()
}
