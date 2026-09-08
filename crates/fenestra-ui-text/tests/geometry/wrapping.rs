use crate::*;

#[test]
fn narrow_wrapping_keeps_graphemes_whole_in_both_directions_and_later_runs() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    let cases = [
        "q\u{301}x".to_owned(),
        "q\u{301}\u{323}x".to_owned(),
        "\u{0628}\u{0651}\u{062a}".to_owned(),
        "\u{0628}\u{0651}\r\n\u{062a}\r\n".to_owned(),
        "q\u{301}\r\nx\r\n".to_owned(),
        "prefix \u{0628}\u{0651}\u{062a}".to_owned(),
        "\u{05d0} q\u{301}\u{323}x".to_owned(),
        format!("q{}x", "\u{301}".repeat(128)),
        format!("\u{0628}{}\u{062a}", "\u{0651}".repeat(128)),
    ];
    for text in cases {
        for width in [Some(1), Some(5), Some(10), Some(20), Some(30), None] {
            let base = request(&text, width, 0);
            let boundaries = base.grapheme_boundaries().collect::<Vec<_>>();
            for pair in boundaries.windows(2) {
                if text[pair[0]..pair[1]].contains(['\r', '\n']) {
                    continue;
                }
                let selection = TextSelection::new(
                    TextPosition::new(pair[0], TextAffinity::Downstream),
                    TextPosition::new(pair[1], TextAffinity::Upstream),
                );
                let geometry = renderer
                    .geometry(query(&text, width, selection, TextGeometryQuery::Current))
                    .unwrap();
                assert_eq!(
                    geometry.anchor_caret().y0(),
                    geometry.focus_caret().y0(),
                    "{text:?}, {width:?}, {pair:?}"
                );
                for highlight in geometry.highlights() {
                    assert_eq!(highlight.rect().y0(), geometry.anchor_caret().y0());
                }
            }
        }
    }
}

#[test]
fn emergency_wrapping_keeps_a_later_ltr_ligature_on_one_line() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    let text = "\u{05d0} ffi";
    let selection = TextSelection::new(
        TextPosition::new(3, TextAffinity::Downstream),
        TextPosition::new(text.len(), TextAffinity::Upstream),
    );
    let geometry = renderer
        .geometry(query(text, Some(1), selection, TextGeometryQuery::Current))
        .unwrap();
    assert_eq!(geometry.anchor_caret().y0(), geometry.focus_caret().y0());
    assert_eq!(geometry.highlights().len(), 1);
    assert_eq!(geometry.metrics().lines(), 2);
}

#[test]
fn ordinary_wrap_prefers_word_boundaries_and_no_wrap_keeps_one_line() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    let text = "abc def";
    let selection = TextSelection::new(
        TextPosition::new(0, TextAffinity::Downstream),
        TextPosition::new(3, TextAffinity::Upstream),
    );
    let first = renderer
        .geometry(query(text, Some(48), selection, TextGeometryQuery::Current))
        .unwrap();
    let second = renderer.geometry(request(text, Some(48), 4)).unwrap();
    assert_eq!(first.metrics().lines(), 2);
    assert_eq!(first.anchor_caret().y0(), 0.0);
    assert_eq!(first.focus_caret().y0(), 0.0);
    assert_eq!(second.focus_caret().y0(), 28.0);
    let unwrapped = renderer.geometry(request(text, None, text.len())).unwrap();
    assert_eq!(unwrapped.metrics().lines(), 1);
    assert!(unwrapped.extent().width() > 48.0);
}
