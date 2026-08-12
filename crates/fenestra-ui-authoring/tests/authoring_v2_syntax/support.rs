use fenestra_ui_authoring::prototype::{
    AnchorKindV2, AuthoringDiagnosticKindV2, AuthoringDiagnosticV2, AuthoringFrontendV2,
    AuthoringLimitKindV2, AuthoringLimitsV2, DiagnosticLocationV2, FenSourceV2, compile_fen_v2,
};
use fenestra_ui_ir::prototype::{SourceId, SourceSpan};

pub const SOURCE: SourceId = SourceId::new(13013);
pub const FIXTURE: &str = include_str!("../fixtures/hybrid_spatial_v2.fen");

pub const LIMIT_VALUES: [usize; 28] = [
    8_192,
    2_048,
    15,
    12,
    1,
    8,
    7,
    1,
    6,
    19,
    2,
    3,
    1,
    16,
    7,
    264,
    5,
    1,
    5,
    3,
    3,
    3,
    3,
    4,
    4,
    3,
    512,
    usize::MAX,
];

pub const GENEROUS_LIMITS: AuthoringLimitsV2 = AuthoringLimitsV2::new(LIMIT_VALUES);

#[derive(Clone, Copy)]
pub struct ExpectedDiagnostic {
    pub kind: AuthoringDiagnosticKindV2,
    pub anchor_kind: AnchorKindV2,
    pub ordinal: u32,
    pub culprit: &'static str,
    pub occurrence: usize,
}

pub fn compile(source: &str) -> fenestra_ui_authoring::prototype::CompiledAuthoringV2 {
    compile_fen_v2(FenSourceV2::new(SOURCE, source.as_bytes()), GENEROUS_LIMITS)
        .unwrap_or_else(|error| panic!("format-2 source should compile: {error:?}"))
}

pub fn assert_diagnostic(source: &str, expected: ExpectedDiagnostic) {
    assert_diagnostic_with_limits(source, GENEROUS_LIMITS, expected);
}

pub fn assert_diagnostic_with_limits(
    source: &str,
    limits: AuthoringLimitsV2,
    expected: ExpectedDiagnostic,
) {
    let error = compile_fen_v2(FenSourceV2::new(SOURCE, source.as_bytes()), limits)
        .expect_err("mutated format-2 source should fail");
    assert_eq!(error.frontend(), AuthoringFrontendV2::Fen);
    assert_eq!(error.kind(), expected.kind);
    assert_anchor(source, &error, expected);
}

pub fn assert_anchor(source: &str, error: &AuthoringDiagnosticV2, expected: ExpectedDiagnostic) {
    let DiagnosticLocationV2::Anchored {
        logical,
        anchor_kind,
        physical,
    } = error.location()
    else {
        panic!("expected anchored diagnostic: {error:?}");
    };
    assert_eq!(
        *logical,
        SourceSpan::bytes(SourceId::new(0), expected.ordinal, expected.ordinal + 1)
    );
    assert_eq!(*anchor_kind, expected.anchor_kind);
    assert_eq!(physical.source_id(), Some(SOURCE));
    assert_eq!(
        physical.fen_byte_range(),
        Some(nth_range(source, expected.culprit, expected.occurrence))
    );
}

pub fn replace_once(source: &str, before: &str, after: &str) -> String {
    assert_eq!(source.matches(before).count(), 1, "ambiguous `{before}`");
    source.replacen(before, after, 1)
}

pub fn nth_range(source: &str, needle: &str, occurrence: usize) -> (u32, u32) {
    let start = source
        .match_indices(needle)
        .nth(occurrence)
        .unwrap_or_else(|| panic!("missing occurrence {occurrence} of `{needle}`"))
        .0;
    let end = start + needle.len();
    (
        u32::try_from(start).expect("test offset should fit"),
        u32::try_from(end).expect("test offset should fit"),
    )
}

pub fn limits_with(kind: AuthoringLimitKindV2, value: usize) -> AuthoringLimitsV2 {
    let mut values = LIMIT_VALUES;
    values[kind as usize] = value;
    AuthoringLimitsV2::new(values)
}
