#[allow(dead_code, unused_imports)]
#[path = "support/layout_board/mod.rs"]
mod support;

use fenestra_ui_authoring::prototype::{FenSourceV1, canonical_rust_v1, compile_fen_v1};
use fenestra_ui_ir::prototype::SourceId;

const CRLF_SOURCE: SourceId = SourceId::new(11);

#[test]
fn crlf_input_retains_supplied_byte_offsets_and_emits_canonical_lf() {
    let lf = compile_fen_v1(
        FenSourceV1::new(support::SOURCE, support::FIXTURE),
        support::REGISTERED_LIMITS,
    )
    .expect("the registered LF fixture should compile");
    let crlf_bytes = String::from_utf8(support::FIXTURE.to_vec())
        .expect("the registered fixture should be UTF-8")
        .replace('\n', "\r\n")
        .into_bytes();
    let crlf = compile_fen_v1(
        FenSourceV1::new(CRLF_SOURCE, &crlf_bytes),
        support::REGISTERED_LIMITS,
    )
    .expect("the equivalent CRLF fixture should compile");

    assert_eq!(crlf.schema(), lf.schema());
    assert_eq!(crlf.construction(), lf.construction());
    assert_eq!(crlf.style(), lf.style());
    assert_eq!(crlf.logical_source_catalog(), lf.logical_source_catalog());

    for (entry, expected) in crlf
        .source_map()
        .entries()
        .iter()
        .zip(support::EXPECTED_ANCHORS)
    {
        let start = crlf_offset(expected.start);
        let end = crlf_offset(expected.end);
        assert_eq!(entry.physical_origin().source_id(), Some(CRLF_SOURCE));
        assert_eq!(entry.physical_origin().fen_byte_range(), Some((start, end)));
        assert_eq!(
            &crlf_bytes[start as usize..end as usize],
            expected.label.as_bytes()
        );
    }

    let lf_rust = canonical_rust_v1(&lf, support::REGISTERED_LIMITS)
        .expect("the LF fixture should emit canonical Rust");
    let crlf_rust = canonical_rust_v1(&crlf, support::REGISTERED_LIMITS)
        .expect("the CRLF fixture should emit canonical Rust");
    assert_eq!(crlf_rust.as_str().as_bytes(), lf_rust.as_str().as_bytes());
    assert!(crlf_rust.as_str().ends_with('\n'));
    assert!(!crlf_rust.as_str().contains('\r'));
}

fn crlf_offset(lf_offset: u32) -> u32 {
    let preceding_lines = support::FIXTURE[..lf_offset as usize]
        .iter()
        .filter(|byte| **byte == b'\n')
        .count();
    lf_offset + u32::try_from(preceding_lines).expect("the fixture line count fits u32")
}
