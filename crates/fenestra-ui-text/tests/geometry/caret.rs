use crate::*;

#[test]
fn rtl_hard_breaks_keep_both_edges_of_each_grapheme_on_its_line() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    for text in [
        "\u{0628}\n\u{062a}\n",
        "\u{0628}\r\n\u{062a}\r\n",
        "\u{05d0}\u{05d1}\r\n\u{05d2}\r\n",
    ] {
        let base = request(text, None, 0);
        let boundaries = base.grapheme_boundaries().collect::<Vec<_>>();
        let mut line = 0;
        for pair in boundaries.windows(2) {
            if text[pair[0]..pair[1]].contains('\n') {
                line += 1;
                continue;
            }
            for (byte, affinity) in [
                (pair[0], TextAffinity::Downstream),
                (pair[1], TextAffinity::Upstream),
            ] {
                let selection = TextSelection::caret(TextPosition::new(byte, affinity));
                let output = renderer
                    .geometry(query(text, None, selection, TextGeometryQuery::Current))
                    .unwrap();
                assert_eq!(
                    output.focus_caret().y0(),
                    f64::from(line * 28),
                    "{text:?}, {byte}, {affinity:?}"
                );
                assert_eq!(output.focus_caret().height(), 28.0);
                assert_eq!(output.selection().focus().byte(), byte);
            }
        }
    }
}

#[test]
fn both_affinities_after_rtl_hard_breaks_use_the_following_line() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    for text in ["\u{0628}\n\u{062a}\n", "\u{0628}\r\n\u{062a}\r\n"] {
        for (line, (newline, _)) in text.match_indices('\n').enumerate() {
            for affinity in [TextAffinity::Downstream, TextAffinity::Upstream] {
                let byte = newline + 1;
                let selection = TextSelection::caret(TextPosition::new(byte, affinity));
                let output = renderer
                    .geometry(query(text, None, selection, TextGeometryQuery::Current))
                    .unwrap();
                assert_eq!(output.metrics().lines(), 3);
                assert_eq!(
                    output.focus_caret().y0(),
                    ((line + 1) * 28) as f64,
                    "{text:?}, {byte}, {affinity:?}"
                );
                assert_eq!(output.focus_caret().height(), 28.0);
            }
        }
    }
}

#[test]
fn rtl_hard_break_caret_retains_the_shaped_fractional_horizontal_position() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    let single = renderer.geometry(request("\u{0628}", None, 0)).unwrap();
    let multiline = renderer
        .geometry(request("\u{0628}\r\n\u{062a}\r\n", None, 0))
        .unwrap();
    assert_ne!(single.focus_caret().x0().fract(), 0.0);
    assert_eq!(multiline.focus_caret(), single.focus_caret());
    assert_eq!(multiline.focus_caret().width(), 1.0);
}

#[test]
fn rtl_first_grapheme_selection_before_a_hard_break_retains_its_highlight() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    for text in [
        "\u{0628}\r\n\u{062a}\r\n",
        "\u{05d0}\u{05d1}\r\n\u{05d2}\r\n",
    ] {
        let selection = TextSelection::new(
            TextPosition::new(0, TextAffinity::Downstream),
            TextPosition::new(2, TextAffinity::Upstream),
        );
        let output = renderer
            .geometry(query(text, None, selection, TextGeometryQuery::Current))
            .unwrap();
        assert_eq!(output.highlights().len(), 1, "{text:?}");
        let highlight = output.highlights()[0];
        assert_eq!(highlight.line(), 0);
        assert_eq!(highlight.rect().y0(), 0.0);
        assert!(highlight.rect().width() > 0.0);
        let reverse = renderer
            .geometry(query(
                text,
                None,
                TextSelection::new(selection.focus(), selection.anchor()),
                TextGeometryQuery::Current,
            ))
            .unwrap();
        assert_eq!(output.highlights(), reverse.highlights());
    }
}

#[test]
fn rtl_home_and_end_at_logical_start_stay_before_the_first_hard_break() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    for (text, end) in [
        ("\u{0628}\r\n\u{062a}\r\n", 2),
        ("\u{05d0}\u{05d1}\r\n\u{05d2}\r\n", 4),
    ] {
        let start = request(text, None, 0).selection();
        for (operation, expected) in [
            (TextGeometryQuery::LineStart { extend: false }, 0),
            (TextGeometryQuery::LineEnd { extend: false }, end),
        ] {
            let output = renderer
                .geometry(query(text, None, start, operation))
                .unwrap();
            assert_eq!(
                output.selection().focus().byte(),
                expected,
                "{text:?}, {operation:?}"
            );
            assert_eq!(output.focus_caret().y0(), 0.0);
        }
    }
}
