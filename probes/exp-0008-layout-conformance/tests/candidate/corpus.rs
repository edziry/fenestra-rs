use fenestra_ui_exp_0008_layout_conformance::prototype::{
    TaffyStackEngineV1, compare_layout_records_v1, registered_layout_corpus_v1,
};
use fenestra_ui_layout::prototype::{
    LayoutInputV1, REGISTERED_LAYOUT_LIMITS_V1, ReferenceStackEngineV1, compute_layout_v1,
};

#[test]
fn all_registered_cases_match_oracle_and_reference_deterministically() {
    let candidate = TaffyStackEngineV1::new();
    let reference = ReferenceStackEngineV1::new();
    let corpus = registered_layout_corpus_v1();

    assert_eq!(corpus.len(), 23);
    for case in &corpus {
        let nodes_before = case.nodes().to_vec();
        let input = LayoutInputV1::new(case.viewport(), case.nodes());
        let input_before = input;

        let reference_output = compute_layout_v1(&reference, input, REGISTERED_LAYOUT_LIMITS_V1)
            .unwrap_or_else(|error| panic!("{}: reference failed: {error:?}", case.name()));
        assert!(
            compare_layout_records_v1(case.expected_records(), reference_output.records())
                .is_none(),
            "{}: reference differs from the literal oracle",
            case.name()
        );

        let first = compute_layout_v1(&candidate, input, REGISTERED_LAYOUT_LIMITS_V1)
            .unwrap_or_else(|error| {
                panic!("{}: first candidate run failed: {error:?}", case.name())
            });
        assert!(
            compare_layout_records_v1(case.expected_records(), first.records()).is_none(),
            "{}: candidate differs from the literal oracle",
            case.name()
        );
        assert_eq!(
            first,
            reference_output,
            "{}: candidate differs from reference",
            case.name()
        );

        let second = compute_layout_v1(&candidate, input, REGISTERED_LAYOUT_LIMITS_V1)
            .unwrap_or_else(|error| {
                panic!("{}: second candidate run failed: {error:?}", case.name())
            });
        assert_eq!(second, first, "{}: candidate output changed", case.name());
        assert_eq!(input, input_before, "{}: input changed", case.name());
        assert_eq!(case.nodes(), nodes_before, "{}: nodes changed", case.name());
    }
}
