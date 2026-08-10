use fenestra_ui_exp_0008_layout_conformance::prototype::{
    compare_layout_records_v1, registered_layout_corpus_v1,
};
use fenestra_ui_layout::prototype::{
    LayoutInputV1, REGISTERED_LAYOUT_LIMITS_V1, ReferenceStackEngineV1, compute_layout_v1,
};

#[test]
fn reference_matches_each_registered_oracle_twice_without_mutating_input() {
    let engine = ReferenceStackEngineV1::new();

    let corpus = registered_layout_corpus_v1();
    for case in corpus.iter() {
        let nodes_before = case.nodes().to_vec();
        let input = LayoutInputV1::new(case.viewport(), case.nodes());
        let input_before = input;

        let first = compute_layout_v1(&engine, input, REGISTERED_LAYOUT_LIMITS_V1)
            .unwrap_or_else(|error| panic!("{}: first run failed: {error:?}", case.name()));
        assert_eq!(
            input,
            input_before,
            "{}: first run changed input",
            case.name()
        );
        assert_eq!(
            case.nodes(),
            nodes_before,
            "{}: first run changed nodes",
            case.name()
        );
        assert!(
            compare_layout_records_v1(case.expected_records(), first.records()).is_none(),
            "{}: first run differs from independent oracle",
            case.name()
        );

        let second = compute_layout_v1(&engine, input, REGISTERED_LAYOUT_LIMITS_V1)
            .unwrap_or_else(|error| panic!("{}: second run failed: {error:?}", case.name()));
        assert_eq!(
            input,
            input_before,
            "{}: second run changed input",
            case.name()
        );
        assert_eq!(
            case.nodes(),
            nodes_before,
            "{}: second run changed nodes",
            case.name()
        );
        assert!(
            compare_layout_records_v1(case.expected_records(), second.records()).is_none(),
            "{}: second run differs from independent oracle",
            case.name()
        );
        assert_eq!(second, first, "{}: repeated output changed", case.name());
    }
}
