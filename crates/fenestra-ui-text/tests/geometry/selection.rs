use crate::*;
use fenestra_ui::{TextBuffer, TextError};

#[test]
fn mixed_bidi_selection_is_disjoint_and_reversing_direction_keeps_the_same_highlight() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    let text = "ab \u{05d0}\u{05d1}\u{05d2} cd";
    let selection = TextSelection::new(
        TextPosition::new(1, TextAffinity::Downstream),
        TextPosition::new(5, TextAffinity::Upstream),
    );
    let forward = renderer
        .geometry(query(text, None, selection, TextGeometryQuery::Current))
        .unwrap();
    let reverse = renderer
        .geometry(query(
            text,
            None,
            TextSelection::new(selection.focus(), selection.anchor()),
            TextGeometryQuery::Current,
        ))
        .unwrap();
    assert_eq!(forward.highlights(), reverse.highlights());
    assert_eq!(forward.highlights().len(), 2);
    assert!(forward.highlights()[0].rect().x1() < forward.highlights()[1].rect().x0());
    assert!(
        forward
            .highlights()
            .iter()
            .all(|rect| rect.line() == 0 && rect.rect().height() == 28.0)
    );
}

#[test]
fn newline_only_selection_obeys_rectangle_limits_without_glyphs_or_pixels() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    let text = "\r\n\r\n";
    let selection = TextSelection::new(
        TextPosition::new(0, TextAffinity::Downstream),
        TextPosition::new(4, TextAffinity::Upstream),
    );
    let base = query(text, None, selection, TextGeometryQuery::Current);
    let output = renderer.geometry(base).unwrap();
    assert_eq!(output.metrics().glyphs(), 0);
    assert_eq!(output.metrics().lines(), 3);
    assert_eq!(output.highlights().len(), 2);
    assert_eq!(output.extent().width(), 0.0);
    assert!(matches!(
        renderer.geometry(base.with_max_rects(1)),
        Err(TextError::LimitExceeded {
            resource: "text geometry rectangles",
            actual: 2,
            limit: 1
        })
    ));
    assert_eq!(renderer.geometry(base).unwrap(), output);
}

#[test]
fn hits_across_combining_and_rtl_ligatures_are_valid_owned_buffer_selections() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    for text in [
        "e\u{301}x",
        "\u{0628}\u{0651}\u{062a}",
        "\u{0644}\u{0627}",
        "fi",
        "\n",
        "\r\n",
    ] {
        let mut buffer = TextBuffer::new(text, 1024).unwrap();
        for y in [-100.0, 0.0, 14.0, 40.0, 100.0] {
            for x in -3..40 {
                let output = renderer
                    .geometry(query(
                        text,
                        Some(50),
                        request(text, Some(50), 0).selection(),
                        TextGeometryQuery::Hit {
                            point: TextPoint::new(f64::from(x), y),
                            extend: false,
                        },
                    ))
                    .unwrap();
                buffer
                    .set_selection(output.selection().byte_selection())
                    .unwrap();
            }
        }
    }
}

#[test]
fn complete_glyph_preflight_applies_before_geometry_and_rejection_can_be_retried() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    let selection = request("abcdef", None, 0).selection();
    let measure = TextMeasureRequest::new(
        "abcdef",
        TextStyle::new(),
        Some(1),
        TextLimits::new(6, 0, 2),
    )
    .unwrap();
    let limited = TextGeometryRequest::new(measure, selection, TextGeometryQuery::Current).unwrap();
    assert!(matches!(
        renderer.geometry(limited),
        Err(TextError::LimitExceeded {
            resource: "text glyphs",
            actual: 3,
            limit: 2
        })
    ));
    assert!(renderer.geometry(request("abcdef", Some(1), 0)).is_ok());
}
