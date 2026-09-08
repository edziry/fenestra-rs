use fenestra_ui::{Size, TextEngine, TextLimits, TextRequest, TextStyle, TextViewportRequest};
use fenestra_ui_text::TextRenderer;

const FONT: &[u8] = include_bytes!("../../../assets/fonts/dejavu-sans-2.37/DejaVuSans.ttf");

fn request(text: &str, size: Size) -> TextRequest<'_> {
    TextRequest::new(
        text,
        TextStyle::new().font_size(20).line_height(28),
        size,
        TextLimits::default(),
    )
    .unwrap()
}

#[test]
fn scrolled_viewport_is_exactly_a_crop_of_the_same_unwrapped_glyph_raster() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    let text = "First line\r\nSecond line with text\r\nThird line";
    let full_request =
        TextViewportRequest::new(request(text, Size::new(300, 84)), None, 0, 0).unwrap();
    let full = renderer.layout_viewport(full_request).unwrap();
    let viewport =
        TextViewportRequest::new(request(text, Size::new(80, 28)), None, 13, 28).unwrap();
    let cropped = renderer.layout_viewport(viewport).unwrap();
    assert_eq!(full.metrics(), cropped.metrics());
    assert_eq!(full.metrics().lines(), 3);
    for (row, bytes) in cropped.raster().bytes().chunks_exact(80 * 4).enumerate() {
        let start = ((row + 28) * 300 + 13) * 4;
        assert_eq!(bytes, &full.raster().bytes()[start..start + 80 * 4]);
    }
    assert!(cropped.raster().bytes().iter().any(|&byte| byte != 0));
}

#[test]
fn compatible_viewport_preserves_legacy_pixels_and_wrap_width_is_independent() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    let text = "Different widths wrap this paragraph differently.";
    let base = request(text, Size::new(80, 112));
    let old = renderer.layout(base).unwrap();
    let compatible = renderer
        .layout_viewport(TextViewportRequest::new(base, Some(80), 0, 0).unwrap())
        .unwrap();
    assert_eq!(old, compatible);
    let unwrapped = renderer
        .layout_viewport(TextViewportRequest::new(base, None, 0, 0).unwrap())
        .unwrap();
    assert_eq!(unwrapped.metrics().lines(), 1);
    assert!(old.metrics().lines() > 1);
    assert_eq!(unwrapped.raster().size(), old.raster().size());
}

#[test]
fn scrolling_beyond_content_is_transparent_and_keeps_complete_metrics() {
    let mut renderer = TextRenderer::new([FONT]).unwrap();
    let base = request("short", Size::new(80, 28));
    let visible = renderer
        .layout_viewport(TextViewportRequest::new(base, None, 0, 0).unwrap())
        .unwrap();
    let far = renderer
        .layout_viewport(TextViewportRequest::new(base, None, 1000, 1000).unwrap())
        .unwrap();
    assert_eq!(visible.metrics(), far.metrics());
    assert!(far.raster().bytes().iter().all(|&byte| byte == 0));
}
