use fenestra_text_screen_common::{FONT, cases, grapheme_boundaries};

#[test]
fn corpus_contains_combining_bidi_and_explicit_missing_glyph_cases() {
    let fixtures = cases();
    assert!(fixtures.iter().any(|case| case.name == "mixed-bidi"));
    assert!(fixtures.iter().any(|case| case.name == "unsupported"));
    let combining = fixtures
        .iter()
        .find(|case| case.name == "combining")
        .unwrap();
    assert_eq!(grapheme_boundaries(combining.text), vec![0, 3, 4, 7, 8, 11]);
    assert_eq!(FONT.len(), 757076);
}
