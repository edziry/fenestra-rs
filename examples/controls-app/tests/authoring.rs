use fenestra_controls_app::{macro_preferences, preferences};

#[test]
fn fen_and_real_ui_macro_produce_the_same_preferences_controls() {
    assert_eq!(preferences(), macro_preferences());
}
