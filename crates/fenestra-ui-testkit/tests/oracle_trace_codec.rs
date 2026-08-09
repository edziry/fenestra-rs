use fenestra_ui_testkit::prototype::{
    ArtifactDecodeErrorKind, ArtifactLimitKind, CaseDecodeContextV1, GeneratedCaseV1,
    GeneratorConfigV1, RuntimeOracleFixtureV1, SeedV1, decode_case_v1, encode_case_v1,
    generate_case_v1, replay_case_with_trace_v1,
};

#[test]
fn directed_case_and_trace_have_exact_canonical_bytes() {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let generated = directed_case(&fixture);

    let encoded = encode_case_v1(&generated).expect("directed case should encode");
    assert_eq!(encoded, directed_case_bytes());

    let trace = replay_case_with_trace_v1(&fixture, &generated)
        .expect("directed case should replay with a trace");
    assert_eq!(trace.case(), &generated);
    assert_eq!(trace.report().transaction_count(), 8);
    assert_eq!(trace.report().verified_step_count(), 8);
    assert_eq!(trace.events().len(), 8);
    assert_eq!(trace.canonical_bytes(), directed_trace_bytes());
}

#[test]
fn equal_seed_case_and_trace_bytes_roundtrip_deterministically() {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let seed = SeedV1::new(1_592_614_637);
    let config = GeneratorConfigV1::new(16, 4, 12);
    let first = generate_case_v1(&fixture, seed, config).expect("first case should generate");
    let second = generate_case_v1(&fixture, seed, config).expect("second case should generate");
    let first_bytes = encode_case_v1(&first).expect("first case should encode");
    let second_bytes = encode_case_v1(&second).expect("second case should encode");

    assert_eq!(first_bytes, second_bytes);
    let decoded =
        decode_case_v1(&first_bytes, context_for(&first)).expect("canonical case should decode");
    assert_eq!(decoded, first);
    assert_eq!(
        encode_case_v1(&decoded).expect("decoded case should reencode"),
        first_bytes
    );

    let first_trace =
        replay_case_with_trace_v1(&fixture, &first).expect("first trace should replay");
    let second_trace =
        replay_case_with_trace_v1(&fixture, &second).expect("second trace should replay");
    assert_eq!(
        first_trace.canonical_bytes(),
        second_trace.canonical_bytes()
    );
}

#[test]
fn case_decoder_rejects_noncanonical_and_malformed_records_privately() {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let generated = directed_case(&fixture);
    let cases: &[(&[u8], ArtifactDecodeErrorKind, Option<u32>)] = &[
        (
            b"tx|00|1\nop|0|set|root|0|i32:1\n",
            ArtifactDecodeErrorKind::NonCanonicalValue,
            Some(1),
        ),
        (
            b"tx|0|1\nop|0|set|root|0|i32:1",
            ArtifactDecodeErrorKind::MalformedRecord,
            Some(2),
        ),
    ];

    for (input, expected, line) in cases {
        let error =
            decode_case_v1(input, context_for(&generated)).expect_err("invalid case should fail");
        assert_eq!(error.kind(), *expected);
        assert_eq!(error.line(), *line);
    }

    let marker = "private-user-payload";
    let input = format!("tx|0|1\nop|0|set|root|0|{marker}\n");
    let error = decode_case_v1(input.as_bytes(), context_for(&generated))
        .expect_err("private payload should be rejected");
    assert_eq!(error.kind(), ArtifactDecodeErrorKind::MalformedRecord);
    assert_eq!(error.line(), Some(2));
    assert!(!format!("{error}").contains(marker));
    assert!(!format!("{error:?}").contains(marker));
}

#[test]
fn case_decoder_enforces_line_and_operation_bounds() {
    let fixture = RuntimeOracleFixtureV1::build().expect("fixture should validate");
    let generated = directed_case(&fixture);

    let mut overlong_line = vec![b'x'; 1_025];
    overlong_line.push(b'\n');
    assert_limit(
        &overlong_line,
        &generated,
        ArtifactLimitKind::LineBytes,
        Some(1),
    );

    let too_many_operations = concat!(
        "tx|0|5\n",
        "op|0|set|root|0|i32:1\n",
        "op|1|set|root|0|i32:1\n",
        "op|2|set|root|0|i32:1\n",
        "op|3|set|root|0|i32:1\n",
        "op|4|set|root|0|i32:1\n",
    );
    assert_limit(
        too_many_operations.as_bytes(),
        &generated,
        ArtifactLimitKind::OperationsPerTransaction,
        Some(1),
    );
}

fn directed_case(fixture: &RuntimeOracleFixtureV1) -> GeneratedCaseV1 {
    generate_case_v1(fixture, SeedV1::new(0), GeneratorConfigV1::new(8, 2, 8))
        .expect("directed case should generate")
}

fn context_for(case: &GeneratedCaseV1) -> CaseDecodeContextV1 {
    CaseDecodeContextV1::new(case.fixture_revision(), case.config(), case.seed())
}

fn assert_limit(
    input: &[u8],
    case: &GeneratedCaseV1,
    expected: ArtifactLimitKind,
    line: Option<u32>,
) {
    let error = decode_case_v1(input, context_for(case)).expect_err("case limit should fail");
    assert_eq!(
        error.kind(),
        ArtifactDecodeErrorKind::LimitExceeded(expected)
    );
    assert_eq!(error.line(), line);
}

fn directed_case_bytes() -> &'static [u8] {
    concat!(
        "tx|0|2\n",
        "op|0|set|root|0|i32:320\n",
        "op|1|set|root|0|i32:480\n",
        "tx|1|1\n",
        "op|2|set|root|0|i32:480\n",
        "tx|2|2\n",
        "op|3|insert|root/r:1|9|2\n",
        "op|4|move|root/r:1|9|0\n",
        "tx|3|1\n",
        "op|5|update|root/r:1|9|0|i32:90\n",
        "tx|4|1\n",
        "op|6|update|root/r:2|7|0|i32:70\n",
        "tx|5|1\n",
        "op|7|insert|root/m:1:9/r:1|2|1\n",
        "tx|6|1\n",
        "op|8|remove|root/r:1|9\n",
        "tx|7|1\n",
        "op|9|insert|root/r:1|9|2\n",
    )
    .as_bytes()
}

fn directed_trace_bytes() -> &'static [u8] {
    concat!(
        "event|0|0|0,1|0|1|commit|1|layout,paint|match\n",
        "event|1|1|2|1|1|noop|0|-|match\n",
        "event|2|2|3,4|1|2|commit|2|structure,layout,paint|match\n",
        "event|3|3|5|2|3|commit|1|intrinsic,layout,paint|match\n",
        "event|4|4|6|3|4|commit|1|intrinsic,layout,paint|match\n",
        "event|5|5|7|4|5|commit|1|structure,layout,paint|match\n",
        "event|6|6|8|5|6|commit|1|structure,layout,paint|match\n",
        "event|7|7|9|6|7|commit|1|structure,layout,paint|match\n",
    )
    .as_bytes()
}
