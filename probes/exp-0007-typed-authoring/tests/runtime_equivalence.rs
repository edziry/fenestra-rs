#[path = "support/runtime_equivalence/mod.rs"]
mod support;

use fenestra_ui_exp_0007_typed_authoring::{generated_layout_board_v1, macro_layout_board_v1};
use fenestra_ui_testkit::prototype::{HeadlessFixtureV1, compare_headless_projection_v1};

use support::{
    expected_receipts, oracle_projection_log, registered_operations, run_lane, validate_programs,
};

#[test]
fn fen_ui_and_manual_programs_publish_the_same_registered_runtime_log() {
    let fixture = HeadlessFixtureV1::build().expect("the manual fixture should validate");
    let operations = registered_operations(&fixture);
    let expected_receipts = expected_receipts(&fixture);
    let oracle = oracle_projection_log(&fixture, &operations);

    let fen = validate_programs(generated_layout_board_v1());
    let ui = validate_programs(macro_layout_board_v1());
    let manual = fixture.style().clone();

    let fen_log = run_lane(&fixture, fen, &operations);
    let ui_log = run_lane(&fixture, ui, &operations);
    let manual_log = run_lane(&fixture, manual, &operations);

    for log in [&fen_log, &ui_log, &manual_log] {
        assert_eq!(log.receipts(), expected_receipts);
        assert_eq!(log.final_keys(), &[10, 30]);
        assert_eq!(log.projections().len(), oracle.len());
        for (expected, observed) in oracle.iter().zip(log.projections()) {
            assert_eq!(
                compare_headless_projection_v1(expected, observed)
                    .expect("registered surfaces should compare"),
                None
            );
        }
    }

    assert_eq!(fen_log, ui_log);
    assert_eq!(fen_log, manual_log);
}
