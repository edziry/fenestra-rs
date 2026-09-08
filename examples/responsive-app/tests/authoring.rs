#[test]
fn fen_and_real_ui_macro_produce_the_same_responsive_workspace() {
    assert_eq!(
        fenestra_responsive_app::workspace(),
        fenestra_responsive_app::macro_workspace()
    );
}
