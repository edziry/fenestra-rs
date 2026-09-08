use fenestra_ui::{
    Size, TextEngine, TextError, TextLimits, TextMeasureRequest, TextRequest, TextStyle,
};
use fenestra_ui_text::TextRenderer;

const FONT: &[u8] = include_bytes!("../../../assets/fonts/dejavu-sans-2.37/DejaVuSans.ttf");

fn request(text: &str, width: Option<u32>) -> TextMeasureRequest<'_> {
    TextMeasureRequest::new(
        text,
        TextStyle::new().font_size(20).line_height(28),
        width,
        TextLimits::new(1024, 0, 1024),
    )
    .unwrap()
}

#[test]
fn intrinsic_measurement_keeps_hard_breaks_and_wraps_only_at_supplied_width() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    let text = "A paragraph wraps when the available width becomes smaller.";
    let intrinsic = renderer.measure(request(text, None)).unwrap();
    let wide = renderer.measure(request(text, Some(300))).unwrap();
    let narrow = renderer.measure(request(text, Some(100))).unwrap();
    assert_eq!(intrinsic.lines(), 1);
    assert_eq!(intrinsic.height(), 28.0);
    assert!(intrinsic.width() > wide.width());
    assert!(wide.height() < narrow.height());
    assert!(narrow.width() <= 100.0);
    let hard_break = renderer.measure(request("first\nsecond", None)).unwrap();
    assert_eq!(hard_break.lines(), 2);
    assert_eq!(hard_break.height(), 56.0);
}

#[test]
fn measurement_matches_complete_raster_metrics_for_edges_and_mixed_direction_text() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    for text in [
        "",
        " ",
        "   ",
        "A ",
        "A\n",
        "\n",
        "\n\n",
        "e\u{301}",
        "English \u{05e9}\u{05dc}\u{05d5}\u{05dd} 123 \u{0627}\u{0644}\u{0639}\u{0631}\u{0628}\u{064a}\u{0629}",
    ] {
        for width in [1, 100, 300] {
            let measure = request(text, Some(width));
            let metrics = renderer.measure(measure).unwrap();
            let raster_request = TextRequest::new(
                text,
                measure.style(),
                Size::new(width, 64),
                TextLimits::default(),
            )
            .unwrap();
            assert_eq!(metrics, renderer.layout(raster_request).unwrap().metrics());
        }
    }
}

#[test]
fn zero_width_measurement_is_distinct_from_unconstrained_width() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    let zero = renderer.measure(request("ABC", Some(0))).unwrap();
    let intrinsic = renderer.measure(request("ABC", None)).unwrap();
    assert!(zero.lines() > intrinsic.lines());
    assert!(zero.width() > 0.0);
    assert!(zero.width() < intrinsic.width());
}

#[test]
fn empty_lines_and_trailing_whitespace_preserve_existing_line_box_metrics() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    for (text, lines, glyphs) in [
        ("", 1, 0),
        (" ", 1, 1),
        ("   ", 1, 3),
        ("\n", 2, 0),
        ("\n\n", 3, 0),
    ] {
        let metrics = renderer.measure(request(text, None)).unwrap();
        assert_eq!(metrics.width(), 0.0);
        assert_eq!(metrics.height(), lines as f32 * 28.0);
        assert_eq!(metrics.lines(), lines);
        assert_eq!(metrics.glyphs(), glyphs);
    }
    let plain = renderer.measure(request("A", None)).unwrap();
    let space = renderer.measure(request("A ", None)).unwrap();
    let newline = renderer.measure(request("A\n", None)).unwrap();
    assert_eq!(plain.width(), space.width());
    assert_eq!(plain.width(), newline.width());
    assert_eq!(newline.lines(), 2);
    assert_eq!(newline.height(), 56.0);
}

#[test]
fn measurement_checks_glyph_budget_and_coverage_before_any_raster_work() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    let limited =
        TextMeasureRequest::new("abcdef", TextStyle::new(), None, TextLimits::new(6, 0, 2))
            .unwrap();
    assert!(matches!(
        renderer.measure(limited),
        Err(TextError::LimitExceeded {
            resource: "text glyphs",
            actual: 3,
            limit: 2,
        })
    ));
    assert_eq!(
        renderer.measure(request("\u{4e2d}", None)).err(),
        Some(TextError::MissingGlyphs { count: 1 })
    );
    assert!(renderer.measure(request("valid", None)).is_ok());
}

#[test]
fn alternating_measurement_and_rendering_keeps_pixels_repeatable() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    let raster_request = TextRequest::new(
        "Stable pixels",
        TextStyle::new(),
        Size::new(120, 48),
        TextLimits::default(),
    )
    .unwrap();
    let first = renderer.layout(raster_request).unwrap();
    for width in [None, Some(0), Some(50), Some(300)] {
        renderer
            .measure(request("different content", width))
            .unwrap();
    }
    assert_eq!(first, renderer.layout(raster_request).unwrap());
}
