#[path = "support/layout_board/mod.rs"]
mod support;

use fenestra_ui_authoring::prototype::{FenSourceV1, canonical_rust_v1, compile_fen_v1};
use fenestra_ui_exp_0007_typed_authoring::{
    LAYOUT_BOARD_GENERATED_RUST_V1, generated_layout_board_v1,
};

#[test]
fn out_dir_generated_triple_matches_the_independent_layout_board_oracle() {
    let generated = generated_layout_board_v1();

    assert_eq!(generated.0, support::expected_schema());
    assert_eq!(generated.1, support::expected_construction());
    assert_eq!(generated.2, support::expected_style());
    assert_eq!(
        support::EXPECTED_LOGICAL_CATALOG.len(),
        support::EXPECTED_ANCHORS.len()
    );

    let repeated = generated_layout_board_v1();
    assert_eq!(repeated.0, generated.0);
    assert_eq!(repeated.1, generated.1);
    assert_eq!(repeated.2, generated.2);
}

#[test]
fn included_canonical_rust_matches_fresh_compilation_and_is_reproducible() {
    let first = fresh_canonical();
    let second = fresh_canonical();

    assert_eq!(LAYOUT_BOARD_GENERATED_RUST_V1, first);
    assert_eq!(LAYOUT_BOARD_GENERATED_RUST_V1, second);
    assert!(LAYOUT_BOARD_GENERATED_RUST_V1.is_ascii());
    assert!(!LAYOUT_BOARD_GENERATED_RUST_V1.contains('\r'));
    assert!(!LAYOUT_BOARD_GENERATED_RUST_V1.contains('/'));
    assert!(!LAYOUT_BOARD_GENERATED_RUST_V1.contains('\\'));

    let body = LAYOUT_BOARD_GENERATED_RUST_V1
        .strip_suffix('\n')
        .expect("included canonical Rust should have one final LF");
    assert!(!body.contains('\n'));
}

fn fresh_canonical() -> String {
    let compiled = compile_fen_v1(
        FenSourceV1::new(support::SOURCE, support::FIXTURE),
        support::REGISTERED_LIMITS,
    )
    .expect("the registered FEN fixture should compile freshly");
    canonical_rust_v1(&compiled, support::REGISTERED_LIMITS)
        .expect("the registered fixture should fit its canonical output bound")
        .as_str()
        .to_owned()
}
