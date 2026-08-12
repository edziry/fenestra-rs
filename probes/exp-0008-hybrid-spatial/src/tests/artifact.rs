use crate::baseline::{
    ArtifactErrorKindV2, ArtifactKindV2, EvidenceSectionV2, GrammarValueKindV2,
    artifact_from_evidence_v2, decode_spatial_evidence_artifact_v2,
    encode_spatial_evidence_artifact_v2, grammar_value_accepts_v2, host_token_probe_v2,
    reconstruct_literal_v2, reconstruct_reference_v2, verify_spatial_evidence_artifact_v2,
};

use super::support::artifact::{
    HEADER, LIMITS, PACKAGES, PROFILE, decimal, fields, fnv1a64, parse, token,
};
use super::support::expected::{CASE_NAMES, CONTROL_FAMILIES, SECTION_NAMES};

#[test]
fn canonical_artifact_has_exact_header_blocks_and_record_order() {
    let evidence = reconstruct_literal_v2().expect("literal evidence");
    let artifact = artifact_from_evidence_v2(&evidence).expect("typed artifact");
    assert_eq!(artifact.kind, ArtifactKindV2::Baseline);
    assert_eq!(artifact.candidate_count, 0);
    let bytes = encode_spatial_evidence_artifact_v2(&artifact).expect("canonical encoding");
    let parsed = parse(&bytes);
    assert_eq!(&parsed.lines[..4], [HEADER, PACKAGES, PROFILE, LIMITS]);

    let mut line = 4;
    for (ordinal, case) in evidence.cases.iter().enumerate() {
        let case_fields = fields(parsed.lines[line]);
        assert!(parsed.lines[line].starts_with("case|"));
        assert_eq!(decimal(case_fields["ordinal"]), ordinal as u64);
        assert_eq!(case_fields["name"], CASE_NAMES[ordinal]);
        assert_eq!(
            decimal(case_fields["observations"]),
            case.observations.len() as u64
        );
        line += 1;
        for (step, _) in case.observations.iter().enumerate() {
            let observation = fields(parsed.lines[line]);
            assert!(parsed.lines[line].starts_with("observation|"));
            assert_eq!(decimal(observation["case"]), ordinal as u64);
            assert_eq!(decimal(observation["step"]), step as u64);
            line += 1;
            for section in SECTION_NAMES {
                let section_fields = fields(parsed.lines[line]);
                assert!(parsed.lines[line].starts_with("section|"));
                assert_eq!(section_fields["name"], section);
                line += 1;
            }
        }
        assert_eq!(
            parsed.lines[line],
            format!("case-result|case={ordinal}|literal=match|reference=match|repeat=match")
        );
        line += 1;
    }
    for family in CONTROL_FAMILIES {
        let control = fields(parsed.lines[line]);
        assert!(parsed.lines[line].starts_with("control|"));
        assert_eq!(control["family"], family);
        assert_eq!(decimal(control["registered"]), decimal(control["detected"]));
        line += 1;
    }
    assert_eq!(
        parsed.lines[line],
        "result|literal=pass|reference=pass|candidate-count=0"
    );
    assert_eq!(parsed.lines[line + 1], "end|spatial-v2");
    assert_eq!(line + 2, parsed.lines.len());
}

#[test]
fn every_section_line_uses_the_private_binary_count_and_fnv_digest() {
    let evidence = reconstruct_reference_v2().expect("reference evidence");
    let artifact = artifact_from_evidence_v2(&evidence).expect("typed artifact");
    let bytes = encode_spatial_evidence_artifact_v2(&artifact).expect("canonical encoding");
    let parsed = parse(&bytes);
    for case in &evidence.cases {
        for observation in &case.observations {
            for section in &observation.sections {
                let key = (case.ordinal, observation.step, section.name.token());
                let encoded = parsed.sections[&key];
                assert_eq!(encoded.records, section.record_count);
                assert_eq!(encoded.bytes, section.encoded.len() as u64);
                assert_eq!(
                    encoded.digest,
                    fnv1a64(section.name.token(), &section.encoded)
                );
            }
        }
    }
}

#[test]
fn two_fresh_models_encode_identically_and_decoder_is_canonical() {
    let first = artifact_from_evidence_v2(&reconstruct_literal_v2().expect("fresh literal one"))
        .expect("first artifact");
    let second = artifact_from_evidence_v2(&reconstruct_literal_v2().expect("fresh literal two"))
        .expect("second artifact");
    let first_bytes = encode_spatial_evidence_artifact_v2(&first).expect("first encoding");
    let second_bytes = encode_spatial_evidence_artifact_v2(&second).expect("second encoding");
    assert_eq!(first_bytes, second_bytes);

    let decoded = decode_spatial_evidence_artifact_v2(&first_bytes).expect("canonical decode");
    verify_spatial_evidence_artifact_v2(&decoded).expect("semantic replay");
    assert_eq!(decoded, first);
    assert_eq!(
        encode_spatial_evidence_artifact_v2(&decoded).unwrap(),
        first_bytes
    );
}

#[test]
fn committed_baseline_is_the_exact_fresh_host_neutral_encoding() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/artifacts/spatial-v2.txt");
    let committed = std::fs::read(path).expect("versioned spatial-v2 baseline artifact");
    let literal = artifact_from_evidence_v2(
        &reconstruct_literal_v2().expect("fresh literal committed evidence"),
    )
    .expect("literal artifact model");
    let reference = artifact_from_evidence_v2(
        &reconstruct_reference_v2().expect("fresh reference committed evidence"),
    )
    .expect("reference artifact model");
    let literal_bytes = encode_spatial_evidence_artifact_v2(&literal).expect("literal encoding");
    let reference_bytes =
        encode_spatial_evidence_artifact_v2(&reference).expect("reference encoding");

    assert_eq!(literal_bytes, reference_bytes);
    assert_eq!(committed, literal_bytes);
    let decoded = decode_spatial_evidence_artifact_v2(&committed).expect("committed decode");
    verify_spatial_evidence_artifact_v2(&decoded).expect("committed semantic replay");
    assert_eq!(decoded, literal);
}

#[test]
fn grammar_values_are_canonical_and_baseline_has_no_lane_rows() {
    let evidence = reconstruct_literal_v2().expect("literal evidence");
    let artifact = artifact_from_evidence_v2(&evidence).expect("typed artifact");
    let bytes = encode_spatial_evidence_artifact_v2(&artifact).expect("canonical encoding");
    let parsed = parse(&bytes);
    assert!(bytes.len() <= 1_048_576);
    assert!(parsed.lines.len() <= 4096);
    assert!(
        !parsed
            .lines
            .iter()
            .any(|line| line.starts_with("candidate|"))
    );
    assert!(
        !parsed
            .lines
            .iter()
            .any(|line| line.starts_with("classification|"))
    );
    for line in parsed.lines {
        let record = line.split('|').next().unwrap();
        assert!(token(record));
        if record == "end" {
            assert_eq!(line, "end|spatial-v2");
            continue;
        }
        for field in line.split('|').skip(1) {
            let (name, value) = field.split_once('=').expect("named field");
            assert!(token(name));
            assert!(!value.is_empty());
        }
    }
}

#[test]
fn canonical_bytes_are_host_neutral_and_reject_host_tokens() {
    let evidence = reconstruct_literal_v2().expect("literal evidence");
    let artifact = artifact_from_evidence_v2(&evidence).expect("typed artifact");
    let bytes = encode_spatial_evidence_artifact_v2(&artifact).expect("canonical encoding");
    let text = std::str::from_utf8(&bytes).unwrap().to_ascii_lowercase();
    for forbidden in [
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
    ] {
        assert!(!text.contains(forbidden), "host token leaked: {forbidden}");
        let error = host_token_probe_v2(forbidden).expect_err("host token rejected");
        assert_eq!(error.kind, ArtifactErrorKindV2::InvalidModel);
    }
    for value in [
        std::env::var("USER").ok(),
        std::env::var("HOSTNAME").ok(),
        std::env::current_dir()
            .ok()
            .map(|path| path.display().to_string()),
    ]
    .into_iter()
    .flatten()
    .filter(|value| value.len() >= 4)
    {
        assert!(!text.contains(&value.to_ascii_lowercase()));
    }
}

#[test]
fn all_ten_section_tags_are_closed_and_zero_based() {
    for (tag, section) in EvidenceSectionV2::ALL.into_iter().enumerate() {
        assert_eq!(section.tag(), tag as u8);
        assert_eq!(section.token(), SECTION_NAMES[tag]);
    }
}

#[test]
fn scalar_list_and_digest_grammar_is_closed_and_canonical() {
    use GrammarValueKindV2 as G;
    let accepted = [
        (G::Token, "a"),
        (G::Token, "registered-v2"),
        (G::List, "a,b-2,c.d"),
        (G::Unsigned, "0"),
        (G::Unsigned, "4194304"),
        (G::Signed, "-1"),
        (G::Signed, "0"),
        (G::Hex16, "0123456789abcdef"),
        (
            G::Hex64,
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        ),
        (G::Absent, "-"),
    ];
    for (kind, value) in accepted {
        assert!(
            grammar_value_accepts_v2(kind, value),
            "must accept {kind:?} {value}"
        );
    }
    for (kind, value) in [
        (G::Token, "A"),
        (G::Token, "a/b"),
        (G::List, "a,,b"),
        (G::Unsigned, "00"),
        (G::Unsigned, "-1"),
        (G::Signed, "-0"),
        (G::Signed, "+1"),
        (G::Hex16, "ABCDEF0123456789"),
        (G::Hex16, "0"),
        (G::Hex64, "0123456789abcdef"),
        (G::Absent, "none"),
    ] {
        assert!(
            !grammar_value_accepts_v2(kind, value),
            "must reject {kind:?} {value}"
        );
    }
}

#[test]
fn decoder_rejects_version_order_count_reference_digest_and_grammar_faults() {
    let evidence = reconstruct_literal_v2().expect("literal evidence");
    let artifact = artifact_from_evidence_v2(&evidence).expect("typed artifact");
    let bytes = encode_spatial_evidence_artifact_v2(&artifact).expect("canonical encoding");
    let text = std::str::from_utf8(&bytes).unwrap();
    let receipt = text.find("name=receipt").unwrap();
    let mapping = text.find("name=mapping").unwrap();
    let receipt_line_start = text[..receipt].rfind('\n').unwrap() + 1;
    let mapping_line_end = text[mapping..].find('\n').unwrap() + mapping + 1;
    let block = &text[receipt_line_start..mapping_line_end];
    let mut pair = block.split_inclusive('\n');
    let first = pair.next().unwrap();
    let second = pair.next().unwrap();

    let faults = [
        (
            text.replacen("artifact=2", "artifact=3", 1),
            ArtifactErrorKindV2::InvalidVersion,
        ),
        (
            text.replacen(block, &format!("{second}{first}"), 1),
            ArtifactErrorKindV2::InvalidOrder,
        ),
        (
            text.replacen("observations=2", "observations=3", 1),
            ArtifactErrorKindV2::InvalidCount,
        ),
        (
            text.replacen("observation|case=0", "observation|case=9", 1),
            ArtifactErrorKindV2::InvalidReference,
        ),
        (
            mutate_first_digest(text),
            ArtifactErrorKindV2::DigestMismatch,
        ),
        (
            text.replacen('\n', "\r\n", 1),
            ArtifactErrorKindV2::InvalidGrammar,
        ),
    ];
    for (fault, expected) in faults {
        let error =
            decode_spatial_evidence_artifact_v2(fault.as_bytes()).expect_err("decoder fault");
        assert_eq!(error.kind, expected);
        assert!(error.artifact.is_none());
    }
}

fn mutate_first_digest(text: &str) -> String {
    let start = text.find("digest=").unwrap() + "digest=".len();
    let replacement = if text.as_bytes()[start] == b'0' {
        "1"
    } else {
        "0"
    };
    let mut value = text.to_owned();
    value.replace_range(start..start + 1, replacement);
    value
}
