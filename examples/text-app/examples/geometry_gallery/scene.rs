use fenestra_ui::{
    Color, Raster, Size, TextAffinity, TextEngine, TextGeometryQuery, TextGeometryRequest,
    TextLimits, TextMeasureRequest, TextPosition, TextRect, TextRequest, TextSelection, TextStyle,
    TextViewportRequest,
};
use fenestra_ui_text::TextRenderer;

use super::{GalleryError, canvas::Canvas};

pub(super) struct Gallery {
    pub(super) raster: Raster,
    pub(super) report: Vec<String>,
    pub(super) ime_caret: (i32, i32, u32, u32),
}

struct Case {
    title: &'static str,
    note: &'static str,
    text: &'static str,
    selection: (usize, usize),
    wrap: Option<u32>,
    offset: (u32, u32),
}

pub(super) fn render(size: Size) -> Result<Gallery, GalleryError> {
    let font = include_bytes!("../../../../assets/fonts/dejavu-sans-2.37/DejaVuSans.ttf");
    let mut engine = TextRenderer::new([font.as_slice()])?;
    let mut canvas = Canvas::new(size, [17, 24, 36, 255])?;
    label(&mut engine, &mut canvas, "Text geometry", 24, 18, 30)?;
    label(
        &mut engine,
        &mut canvas,
        "Selections, carets and text viewports",
        24,
        58,
        16,
    )?;
    let cases = [
        Case {
            title: "Combining sequence",
            note: "The accent and base form one editable grapheme.",
            text: "Cafe\u{301} and office fi",
            selection: (3, 6),
            wrap: None,
            offset: (0, 0),
        },
        Case {
            title: "Bidirectional selection",
            note: "Logical selection can have separate visual fragments.",
            text: "abc \u{05d0}\u{05d1}\u{05d2} def",
            selection: (2, 8),
            wrap: None,
            offset: (0, 0),
        },
        Case {
            title: "CRLF and empty final line",
            note: "Original byte offsets survive line-break shaping.",
            text: "First\r\nSecond\r\n",
            selection: (15, 15),
            wrap: Some(360),
            offset: (0, 0),
        },
        Case {
            title: "Horizontal viewport",
            note: "Text is clipped after a 180 px horizontal offset.",
            text: "A long line keeps its real glyph positions while scrolling.",
            selection: (24, 35),
            wrap: None,
            offset: (180, 0),
        },
    ];
    let mut report = Vec::new();
    let mut ime_caret = (0, 0, 1, 1);
    for (index, case) in cases.iter().enumerate() {
        let x = 24 + (index as u32 % 2) * 438;
        let y = 110 + (index as u32 / 2) * 208;
        label(&mut engine, &mut canvas, case.title, x, y, 20)?;
        let viewport = Size::new(390, 102);
        let style = TextStyle::new()
            .font_size(24)
            .line_height(32)
            .color(Color::rgba8(235, 241, 246, 255));
        let measurement =
            TextMeasureRequest::new(case.text, style, case.wrap, TextLimits::default())?;
        let position = |byte| TextPosition::new(byte, TextAffinity::Downstream);
        let selection = TextSelection::new(position(case.selection.0), position(case.selection.1));
        let query = TextGeometryRequest::new(measurement, selection, TextGeometryQuery::Current)?;
        let geometry = engine.geometry(query)?;
        geometry.validate_request(query)?;
        let raster_request = TextRequest::new(case.text, style, viewport, TextLimits::default())?;
        let request =
            TextViewportRequest::new(raster_request, case.wrap, case.offset.0, case.offset.1)?;
        let text = engine.layout_viewport(request)?;
        text.validate_request(raster_request)?;
        assert_eq!(text.metrics(), geometry.metrics());
        if index == 2 {
            assert_eq!(geometry.metrics().lines(), 3);
            assert_eq!(geometry.selection().focus().byte(), case.text.len());
        }
        let mut pane = Canvas::new(viewport, [29, 41, 58, 255])?;
        for highlight in geometry.highlights() {
            pane.fill(
                translate(
                    highlight.rect(),
                    -(case.offset.0 as f64),
                    -(case.offset.1 as f64),
                ),
                [24, 82, 132, 255],
            );
        }
        pane.layer(text.raster(), 0, 0);
        let caret = translate(
            geometry.focus_caret(),
            -(case.offset.0 as f64),
            -(case.offset.1 as f64),
        );
        pane.fill(caret, [255, 211, 110, 255]);
        canvas.layer(&pane.finish()?, x, y + 38);
        label(&mut engine, &mut canvas, case.note, x, y + 150, 14)?;
        if index == 0 {
            ime_caret = (
                x as i32 + caret.x0().floor() as i32,
                (y + 38) as i32 + caret.y0().floor() as i32,
                (caret.x1().ceil() - caret.x0().floor()) as u32,
                (caret.y1().ceil() - caret.y0().floor()) as u32,
            );
        }
        report.push(format!(
            "case={index} text_bytes={} selection={}..{} lines={} highlights={} caret=({:.3},{:.3},{:.3},{:.3}) extent=({:.3},{:.3}) offset={},{}",
            case.text.len(), geometry.selection().anchor().byte(), geometry.selection().focus().byte(), geometry.metrics().lines(), geometry.highlights().len(), geometry.focus_caret().x0(), geometry.focus_caret().y0(), geometry.focus_caret().x1(), geometry.focus_caret().y1(), geometry.extent().width(), geometry.extent().height(), case.offset.0, case.offset.1,
        ));
    }
    Ok(Gallery {
        raster: canvas.finish()?,
        report,
        ime_caret,
    })
}

fn label(
    engine: &mut TextRenderer,
    canvas: &mut Canvas,
    text: &str,
    x: u32,
    y: u32,
    font_size: u32,
) -> Result<(), GalleryError> {
    let style = TextStyle::new()
        .font_size(font_size)
        .line_height(font_size + 6)
        .color(Color::rgba8(177, 197, 215, 255));
    let size = Size::new(if font_size < 20 { 410 } else { 800 }, font_size + 12);
    let layout = engine.layout(TextRequest::new(text, style, size, TextLimits::default())?)?;
    canvas.layer(layout.raster(), x, y);
    Ok(())
}

fn translate(rect: TextRect, x: f64, y: f64) -> TextRect {
    TextRect::new(rect.x0() + x, rect.y0() + y, rect.x1() + x, rect.y1() + y)
}
