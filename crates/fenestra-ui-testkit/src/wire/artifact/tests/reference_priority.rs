use super::super::decode_failure_artifact_v1;
use super::support::{
    assert_decode_error, canonical_structural_artifact, decode_error, replace_once,
};
use crate::wire::error::ArtifactDecodeErrorKind;

const ORIGINAL_FAILURE: &str =
    "failure|original|2|-|state-mismatch|fragment:root/r:1|keyed-order|keys:9,7,8|keys:7,8,9";
const MINIMIZED_FAILURE: &str =
    "failure|minimized|2|-|state-mismatch|fragment:root/r:1|keyed-order|keys:9,7,8|keys:7,8,9";

#[test]
fn case_identifiers_increase_strictly_across_transaction_boundaries() {
    let cases = [
        (
            replace_once(canonical_structural_artifact(), "tx|1|1", "tx|0|1"),
            11,
        ),
        (
            replace_once(
                canonical_structural_artifact(),
                "op|2|set|root|0|i32:480",
                "op|1|set|root|0|i32:480",
            ),
            12,
        ),
        (
            replace_once(
                canonical_structural_artifact(),
                "op|3|insert|root/r:1|9|2\nop|4|move|root/r:1|9|0\nminimized-end",
                "op|3|insert|root/r:1|9|2\nop|3|move|root/r:1|9|0\nminimized-end",
            ),
            33,
        ),
    ];

    for (input, line) in cases {
        assert_decode_error(
            &input,
            ArtifactDecodeErrorKind::OrderingViolation,
            Some(line),
        );
    }
}

#[test]
fn minimized_transactions_retain_original_boundary_ids() {
    let input = replace_once(
        canonical_structural_artifact(),
        "minimized-begin|1|2|55\ntx|2|2",
        "minimized-begin|1|2|55\ntx|8|2",
    );

    assert_invalid_reference(&input, 31);
}

#[test]
fn minimized_operation_ids_are_an_original_subsequence() {
    let cases = [
        (
            replace_once(
                canonical_structural_artifact(),
                "minimized-begin|1|2|55\ntx|2|2\nop|3|insert|root/r:1|9|2",
                "minimized-begin|1|2|55\ntx|2|2\nop|2|insert|root/r:1|9|2",
            ),
            32,
        ),
        (
            replace_once(
                canonical_structural_artifact(),
                "minimized-begin|1|2|55\ntx|2|2\nop|3|insert|root/r:1|9|2\nop|4|move|root/r:1|9|0",
                "minimized-begin|1|2|56\ntx|2|2\nop|3|insert|root/r:1|9|2\nop|10|move|root/r:1|9|0",
            ),
            33,
        ),
    ];

    for (input, line) in cases {
        assert_invalid_reference(&input, line);
    }
}

#[test]
fn minimized_operands_may_shrink_without_changing_record_identity() {
    let input = replace_once(
        canonical_structural_artifact(),
        "minimized-begin|1|2|55\ntx|2|2\nop|3|insert|root/r:1|9|2",
        "minimized-begin|1|2|55\ntx|2|2\nop|3|insert|root/r:1|9|1",
    );

    assert!(decode_failure_artifact_v1(&input).is_ok());
}

#[test]
fn fault_target_exists_in_both_cases_and_remains_move_keyed() {
    let cases = [
        replace_once(
            canonical_structural_artifact(),
            "fault|omit-move|4",
            "fault|omit-move|99",
        ),
        replace_once(
            canonical_structural_artifact(),
            "fault|omit-move|4",
            "fault|omit-move|3",
        ),
        replace_once(
            canonical_structural_artifact(),
            "minimized-begin|1|2|55\ntx|2|2\nop|3|insert|root/r:1|9|2\nop|4|move|root/r:1|9|0",
            "minimized-begin|1|2|57\ntx|2|2\nop|3|insert|root/r:1|9|2\nop|4|insert|root/r:1|9|0",
        ),
    ];

    for input in cases {
        assert_invalid_reference(&input, 27);
    }
}

#[test]
fn failure_identifiers_resolve_in_their_case_and_match_across_sections() {
    let wrong_operation = replace_once(
        canonical_structural_artifact(),
        ORIGINAL_FAILURE,
        "failure|original|2|2|candidate-rejected|global|candidate-outcome|kind:accept|kind:missing-key",
    );
    let mismatched_operations = replace_once(
        &replace_once(
            canonical_structural_artifact(),
            ORIGINAL_FAILURE,
            "failure|original|2|3|candidate-rejected|global|candidate-outcome|kind:accept|kind:missing-key",
        ),
        MINIMIZED_FAILURE,
        "failure|minimized|2|4|candidate-rejected|global|candidate-outcome|kind:accept|kind:missing-key",
    );
    let cases = [
        (
            replace_once(
                canonical_structural_artifact(),
                ORIGINAL_FAILURE,
                "failure|original|9|-|state-mismatch|fragment:root/r:1|keyed-order|keys:9,7,8|keys:7,8,9",
            ),
            28,
        ),
        (
            replace_once(
                canonical_structural_artifact(),
                MINIMIZED_FAILURE,
                "failure|minimized|3|-|state-mismatch|fragment:root/r:1|keyed-order|keys:9,7,8|keys:7,8,9",
            ),
            35,
        ),
        (wrong_operation, 28),
        (mismatched_operations, 35),
        (
            replace_once(
                canonical_structural_artifact(),
                ORIGINAL_FAILURE,
                "failure|original|3|-|state-mismatch|fragment:root/r:1|keyed-order|keys:9,7,8|keys:7,8,9",
            ),
            35,
        ),
    ];

    for (input, line) in cases {
        assert_invalid_reference(&input, line);
    }
}

#[test]
fn invalid_reference_precedes_invalid_fingerprint_and_trailing_data() {
    const PRIVATE: &str = "private-trailing-reference-marker";
    let bad_reference = replace_once(
        canonical_structural_artifact(),
        "fault|omit-move|4",
        "fault|omit-move|3",
    );
    let bad_fingerprint = replace_once(
        &bad_reference,
        ORIGINAL_FAILURE,
        "failure|original|2|-|state-mismatch|node:root|fragment-binding|binding:present|binding:absent",
    );
    let mut input = bad_fingerprint;
    input.extend_from_slice(format!("{PRIVATE}\n").as_bytes());

    let error = decode_error(&input);
    assert_eq!(error.kind(), ArtifactDecodeErrorKind::InvalidReference);
    assert_eq!(error.line(), Some(27));
    assert!(!format!("{error:?}").contains(PRIVATE));
    assert!(!error.to_string().contains(PRIVATE));
}

#[test]
fn section_references_precede_the_fault_target() {
    let input = replace_once(
        &replace_once(
            canonical_structural_artifact(),
            ORIGINAL_FAILURE,
            "failure|original|9|-|state-mismatch|fragment:root/r:1|keyed-order|keys:9,7,8|keys:7,8,9",
        ),
        "fault|omit-move|4",
        "fault|omit-move|3",
    );

    assert_invalid_reference(&input, 28);
}

fn assert_invalid_reference(input: &[u8], line: u32) {
    assert_decode_error(input, ArtifactDecodeErrorKind::InvalidReference, Some(line));
}
