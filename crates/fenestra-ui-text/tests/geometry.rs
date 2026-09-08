use fenestra_ui::{
    TextAffinity, TextEngine, TextGeometryQuery, TextGeometryRequest, TextLimits,
    TextMeasureRequest, TextPoint, TextPosition, TextSelection, TextStyle,
};
use fenestra_ui_text::TextRenderer;

const FONT: &[u8] = include_bytes!("../../../assets/fonts/dejavu-sans-2.37/DejaVuSans.ttf");

mod geometry {
    mod caret;
    mod navigation;
    mod selection;
}

fn request(text: &str, width: Option<u32>, byte: usize) -> TextGeometryRequest<'_> {
    TextGeometryRequest::new(
        TextMeasureRequest::new(
            text,
            TextStyle::new().font_size(20).line_height(28),
            width,
            TextLimits::new(1024, 0, 1024),
        )
        .unwrap(),
        TextSelection::caret(TextPosition::new(byte, TextAffinity::Downstream)),
        TextGeometryQuery::Current,
    )
    .unwrap()
}

fn query<'a>(
    text: &'a str,
    width: Option<u32>,
    selection: TextSelection,
    operation: TextGeometryQuery,
) -> TextGeometryRequest<'a> {
    TextGeometryRequest::new(request(text, width, 0).measurement(), selection, operation).unwrap()
}

#[test]
fn combining_grapheme_hits_and_visual_steps_never_enter_its_interior() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    let text = "e\u{301}x";
    for x in 0..24 {
        let base = request(text, None, 0);
        let hit = TextGeometryRequest::new(
            base.measurement(),
            base.selection(),
            TextGeometryQuery::Hit {
                point: TextPoint::new(f64::from(x), 14.0),
                extend: false,
            },
        )
        .unwrap();
        let result = renderer.geometry(hit).unwrap();
        assert!([0, 3, 4].contains(&result.selection().focus().byte()));
        result.validate_request(hit).unwrap();
    }
    let base = request(text, None, 0);
    let right = TextGeometryRequest::new(
        base.measurement(),
        base.selection(),
        TextGeometryQuery::Right { extend: false },
    )
    .unwrap();
    assert_eq!(
        renderer.geometry(right).unwrap().selection().focus().byte(),
        3
    );
}

#[test]
fn crlf_is_one_hard_break_and_geometry_keeps_original_utf8_offsets() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    let text = "A\r\nB\r\n";
    let start = renderer.geometry(request(text, None, 0)).unwrap();
    let second = renderer.geometry(request(text, None, 3)).unwrap();
    let end = renderer.geometry(request(text, None, text.len())).unwrap();
    assert_eq!(start.metrics().lines(), 3);
    assert_eq!(start.metrics().height(), 84.0);
    assert_eq!(start.focus_caret().y0(), 0.0);
    assert_eq!(second.focus_caret().y0(), 28.0);
    assert_eq!(end.focus_caret().y0(), 56.0);
    assert_eq!(end.selection().focus().byte(), text.len());
    assert_eq!(
        start.metrics(),
        renderer
            .measure(request(text, None, 0).measurement())
            .unwrap()
    );
}

#[test]
fn caret_extent_includes_trailing_spaces_and_empty_line_has_no_phantom_width() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    let empty = renderer.geometry(request("", None, 0)).unwrap();
    assert_eq!(empty.focus_caret().x0(), 0.0);
    assert_eq!(empty.focus_caret().height(), 28.0);
    assert_eq!(empty.extent().width(), 0.0);
    let text = "A   ";
    let end = renderer.geometry(request(text, None, text.len())).unwrap();
    assert!(end.focus_caret().x0() > f64::from(end.metrics().width()));
    assert_eq!(end.extent().x1(), end.focus_caret().x0());
}
