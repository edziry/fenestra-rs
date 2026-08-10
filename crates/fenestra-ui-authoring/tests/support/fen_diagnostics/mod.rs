use fenestra_ui_authoring::prototype::{
    AnchorKindV1, AuthoringDiagnosticKindV1, AuthoringFrontendV1, AuthoringLimitsV1,
    DiagnosticLocationV1, FenSourceV1, compile_fen_v1,
};
use fenestra_ui_ir::prototype::{SourceId, SourceSpan};

pub const SOURCE: SourceId = SourceId::new(91);
pub const BASE: &str = "format 1;
schema namespace 77 revision 1 {
  component boxy = 0 {
    property scalar = 0: scalar_i32 = 0 invalidates [layout];
    property shade = 1: rgba8 = rgba8(1, 2, 3, 255) invalidates [paint];
  }
}
construction {
  template root = 0: boxy {
    set scalar = 1;
    child template leaf;
    child region rows;
  }
  template leaf = 1: boxy {}
  template cell = 2: boxy {}
  region rows = 0 owner root repeat cell keys [10] invalidates [structure];
}
style {
  set leaf.shade = rgba8(10, 20, 30, 255);
}
";

pub const GENEROUS_LIMITS: AuthoringLimitsV1 = AuthoringLimitsV1::new(
    16_384, 4_096, 64, 16, 32, 32, 32, 32, 64, 64, 64, 64, 128, 65_536,
);

#[derive(Clone, Copy)]
pub struct ExpectedDiagnostic {
    pub kind: AuthoringDiagnosticKindV1,
    pub anchor_kind: AnchorKindV1,
    pub ordinal: u32,
    pub culprit: &'static str,
    pub occurrence: usize,
}

pub fn diagnostic_mismatch(source: &str, expected: ExpectedDiagnostic) -> Option<String> {
    let error = match compile_fen_v1(FenSourceV1::new(SOURCE, source.as_bytes()), GENEROUS_LIMITS) {
        Ok(_) => return Some(format!("accepted source expecting {:?}", expected.kind)),
        Err(error) => error,
    };
    if error.frontend() != AuthoringFrontendV1::Fen {
        return Some(format!("wrong frontend for {:?}", expected.kind));
    }
    if error.kind() != expected.kind {
        return Some(format!(
            "expected {:?}, got {:?}",
            expected.kind,
            error.kind()
        ));
    }
    let DiagnosticLocationV1::Anchored {
        logical,
        anchor_kind,
        physical,
    } = error.location()
    else {
        return Some(format!("expected anchored {:?}", expected.kind));
    };
    if *anchor_kind == AnchorKindV1::Document {
        return Some(format!("invented Document origin for {:?}", expected.kind));
    }
    let expected_span = SourceSpan::bytes(SourceId::new(0), expected.ordinal, expected.ordinal + 1);
    if *logical != expected_span || *anchor_kind != expected.anchor_kind {
        return Some(format!(
            "wrong logical anchor for {:?}: kind {:?}, span {:?}",
            expected.kind, anchor_kind, logical
        ));
    }
    if physical.source_id() != Some(SOURCE) {
        return Some(format!("wrong source identity for {:?}", expected.kind));
    }
    let expected_range = nth_range(source, expected.culprit, expected.occurrence);
    if physical.fen_byte_range() != Some(expected_range) {
        return Some(format!(
            "wrong physical token for {:?}: expected {:?}, got {:?}",
            expected.kind,
            expected_range,
            physical.fen_byte_range()
        ));
    }
    None
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
        u32::try_from(start).expect("test source offset should fit"),
        u32::try_from(end).expect("test source offset should fit"),
    )
}
