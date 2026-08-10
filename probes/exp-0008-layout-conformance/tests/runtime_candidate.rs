#![forbid(unsafe_code)]

#[path = "runtime_candidate/script.rs"]
mod script;
#[path = "runtime_candidate/support.rs"]
mod support;

use script::{LaneRunV1, run_candidate_lane_v1, run_reference_lane_v1};

#[test]
fn registered_runtime_script_matches_reference_oracle_and_taffy_twice() {
    let reference_first = run_reference_lane_v1();
    let reference_second = run_reference_lane_v1();
    let candidate_first = run_candidate_lane_v1();
    let candidate_second = run_candidate_lane_v1();

    assert_eq!(reference_first.transcript(), reference_second.transcript());
    assert_eq!(candidate_first.transcript(), candidate_second.transcript());
    assert_eq!(reference_first.transcript(), candidate_first.transcript());
    assert_eq!(reference_second.transcript(), candidate_second.transcript());

    assert_candidate_backend_entries(&candidate_first);
    assert_candidate_backend_entries(&candidate_second);
}

fn assert_candidate_backend_entries(run: &LaneRunV1) {
    let calls = run.engine_calls();
    assert_eq!(calls.len(), 7);
    assert_eq!(calls[0], 1, "initialization must enter Taffy exactly once");

    for index in 2..calls.len() {
        assert!(
            calls[index] > calls[index - 1],
            "layout-relevant milestone {index} must reenter Taffy"
        );
    }
}
