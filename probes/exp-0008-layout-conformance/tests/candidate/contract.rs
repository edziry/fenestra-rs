use fenestra_ui_exp_0008_layout_conformance::prototype::TaffyStackEngineV1;
use fenestra_ui_layout::prototype::{
    LayoutEngineV1, LayoutErrorKindV1, LayoutErrorLocationV1, LayoutInputErrorKindV1,
    LayoutInputV1, REGISTERED_LAYOUT_LIMITS_V1, compute_layout_v1,
};

use super::support::viewport;

#[test]
fn adapter_implements_the_candidate_neutral_layout_engine_boundary() {
    fn assert_layout_engine<T: LayoutEngineV1>() {}

    assert_layout_engine::<TaffyStackEngineV1>();
    let _first = TaffyStackEngineV1::new();
    let _second = TaffyStackEngineV1::new();
}

#[test]
fn invalid_core_input_is_preserved_as_a_core_error() {
    let error = compute_layout_v1(
        &TaffyStackEngineV1::new(),
        LayoutInputV1::new(viewport(0, 0), &[]),
        REGISTERED_LAYOUT_LIMITS_V1,
    )
    .expect_err("empty core input must fail before candidate work");

    assert_eq!(
        error.kind(),
        LayoutErrorKindV1::Input(LayoutInputErrorKindV1::EmptyInput)
    );
    assert_eq!(error.location(), LayoutErrorLocationV1::Input);
}
