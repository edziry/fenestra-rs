#![allow(dead_code)]

#[path = "artifact_decode_support.rs"]
mod decode;

use fenestra_ui_testkit::prototype::{HeadlessArtifactV1, encode_headless_artifact_v1};

pub fn canonical_bytes() -> Vec<u8> {
    decode::canonical_bytes()
}

pub fn fixed_point(bytes: &[u8]) -> HeadlessArtifactV1 {
    let artifact = decode::decode(bytes);
    assert_eq!(
        encode_headless_artifact_v1(&artifact)
            .expect("structurally valid verification input should encode"),
        bytes
    );
    artifact
}

pub fn set_field(bytes: &[u8], prefix: &str, field: usize, value: &str) -> Vec<u8> {
    decode::set_field(bytes, prefix, field, value)
}

pub fn replace_once(bytes: &[u8], from: &str, to: &str) -> Vec<u8> {
    decode::replace_once(bytes, from, to)
}

pub fn shift_all_generations(bytes: &[u8]) -> Vec<u8> {
    rewrite_fields(bytes, |fields| match fields.first().map(String::as_str) {
        Some("h-event") => {
            shift_optional(&mut fields[8]);
            shift_optional(&mut fields[9]);
        }
        Some("s-event") => shift_number(&mut fields[11]),
        Some("projection-begin") => shift_number(&mut fields[1]),
        _ => {}
    })
}

pub fn shift_all_surfaces(bytes: &[u8]) -> Vec<u8> {
    rewrite_fields(bytes, |fields| match fields.first().map(String::as_str) {
        Some("h-event") => {
            shift_number(&mut fields[13]);
            shift_number(&mut fields[14]);
        }
        Some("projection-begin") => {
            shift_number(&mut fields[2]);
            shift_number(&mut fields[3]);
        }
        _ => {}
    })
}

pub fn add_semantic_record(bytes: &[u8]) -> Vec<u8> {
    let changed = set_field(bytes, "projection-begin|", 6, "1");
    replace_once(
        &changed,
        "semantic-begin\nsemantic-end",
        "semantic-begin\nsemantic|root/s:0/s:0|control|1|activate\nsemantic-end",
    )
}

fn rewrite_fields(bytes: &[u8], mut rewrite: impl FnMut(&mut Vec<String>)) -> Vec<u8> {
    let text = std::str::from_utf8(bytes).expect("canonical artifact should be ASCII");
    let mut output = String::new();
    for line in text.lines() {
        let mut fields = line.split('|').map(str::to_owned).collect::<Vec<_>>();
        rewrite(&mut fields);
        output.push_str(&fields.join("|"));
        output.push('\n');
    }
    output.into_bytes()
}

fn shift_optional(value: &mut String) {
    if value != "-" {
        shift_number(value);
    }
}

fn shift_number(value: &mut String) {
    let number = value
        .parse::<u64>()
        .expect("canonical verification field should be numeric");
    *value = (number + 1).to_string();
}
