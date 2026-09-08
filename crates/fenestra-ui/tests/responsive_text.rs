use std::cell::Cell;
use std::rc::Rc;

use fenestra_ui::{
    Application, Color, Dimension, Element, Error, Raster, Size, Style, TextEngine, TextError,
    TextLayout, TextLimits, TextMeasureRequest, TextMetrics, TextRequest, TextStyle, View,
};

#[derive(Default)]
struct Calls {
    measure: Cell<usize>,
    raster: Cell<usize>,
}

struct Engine(Rc<Calls>);

fn metrics(text: &str, width: Option<u32>, style: TextStyle) -> TextMetrics {
    let count = text.chars().count();
    let per_line = width.map_or(count.max(1), |width| (width as usize / 4).max(1));
    let lines = count.div_ceil(per_line).max(1);
    TextMetrics::new(
        (count.min(per_line) * 4) as f32,
        (lines * style.line_height_value() as usize) as f32,
        lines,
        count,
        0,
    )
}

impl TextEngine for Engine {
    fn measure(&mut self, request: TextMeasureRequest<'_>) -> Result<TextMetrics, TextError> {
        self.0.measure.set(self.0.measure.get() + 1);
        if request.text() == "reject" {
            return Err(TextError::Engine("measurement fixture failure".into()));
        }
        Ok(metrics(request.text(), request.width(), request.style()))
    }

    fn layout(&mut self, request: TextRequest<'_>) -> Result<TextLayout, TextError> {
        self.0.raster.set(self.0.raster.get() + 1);
        let size = request.size();
        let pixels = vec![255; size.width() as usize * size.height() as usize * 4];
        TextLayout::new(
            Raster::new(size, pixels).unwrap(),
            metrics(request.text(), Some(size.width()), request.style()),
        )
    }
}

fn view() -> View {
    View::new(
        "workspace",
        Element::column("root")
            .style(
                Style::new()
                    .width_mode(Dimension::Fill(1))
                    .height_mode(Dimension::Fill(1))
                    .padding(2)
                    .gap(1),
            )
            .child(
                Element::text("header", "abc").style(
                    Style::new()
                        .width_mode(Dimension::Fill(1))
                        .height_mode(Dimension::Auto),
                ),
            )
            .child(
                Element::row("body")
                    .style(
                        Style::new()
                            .width_mode(Dimension::Fill(1))
                            .height_mode(Dimension::Fill(1))
                            .gap(2),
                    )
                    .child(
                        Element::rect("sidebar")
                            .style(Style::new().width(10).height_mode(Dimension::Fill(1))),
                    )
                    .child(
                        Element::column("main")
                            .style(
                                Style::new()
                                    .width_mode(Dimension::Fill(1))
                                    .height_mode(Dimension::Fill(1))
                                    .gap(1),
                            )
                            .child(
                                Element::text("article", "abcdefghijklmnopqrst").style(
                                    Style::new()
                                        .width_mode(Dimension::Fill(1))
                                        .height_mode(Dimension::Auto),
                                ),
                            )
                            .child(
                                Element::rect("button").style(
                                    Style::new()
                                        .width_mode(Dimension::Fill(1))
                                        .height(4)
                                        .background(Color::rgba8(10, 20, 30, 255))
                                        .input(true),
                                ),
                            ),
                    ),
            ),
    )
}

fn application() -> (Application, Rc<Calls>) {
    let calls = Rc::new(Calls::default());
    let app =
        Application::with_text_engine(view(), Size::new(80, 100), Engine(calls.clone())).unwrap();
    (app, calls)
}

#[test]
fn window_resize_rewraps_text_and_moves_sibling_paint_and_hits_together() {
    let (mut app, calls) = application();
    let old = app.raster().unwrap();
    assert_eq!(app.bounds("article").unwrap().width(), 64);
    assert_eq!(app.text_metrics("article").unwrap().lines(), 2);
    assert_eq!(app.bounds("button").unwrap().y(), 76);
    assert_eq!(app.hit_test(14, 76), Some("button"));
    app.resize(Size::new(120, 100)).unwrap();
    assert_eq!(app.bounds("article").unwrap().width(), 104);
    assert_eq!(app.text_metrics("article").unwrap().lines(), 1);
    assert_eq!(app.bounds("button").unwrap().y(), 52);
    assert_eq!(app.hit_test(14, 52), Some("button"));
    assert_eq!(app.hit_test(14, 76), None);
    let new = app.raster().unwrap();
    let offset = (52 * 120 + 14) * 4;
    assert_eq!(&new.bytes()[offset..offset + 4], &[10, 20, 30, 255]);
    assert_eq!(old.size(), Size::new(80, 100));
    assert_eq!(app.generation(), 1);
    let measured = calls.measure.get();
    let rasterized = calls.raster.get();
    app.set_background("button", Color::rgba8(40, 50, 60, 255))
        .unwrap();
    app.resize(Size::new(120, 100)).unwrap();
    assert_eq!(calls.measure.get(), measured);
    assert_eq!(calls.raster.get(), rasterized);
}

#[test]
fn content_and_typography_reflow_neighbors_while_failed_measurement_preserves_everything() {
    let (mut app, _) = application();
    app.set_text("article", "abcd").unwrap();
    assert_eq!(app.bounds("button").unwrap().y(), 52);
    app.set_text_style("article", TextStyle::new().line_height(30))
        .unwrap();
    assert_eq!(app.bounds("button").unwrap().y(), 58);
    let frame = app.raster().unwrap();
    let style = app.style("article").unwrap();
    let metrics = app.text_metrics("article").unwrap();
    assert!(app.set_text("article", "reject").is_err());
    assert_eq!(app.generation(), 2);
    assert_eq!(app.text("article").unwrap(), "abcd");
    assert_eq!(app.style("article").unwrap(), style);
    assert_eq!(app.text_metrics("article").unwrap(), metrics);
    assert_eq!(app.bounds("button").unwrap().y(), 58);
    assert_eq!(app.raster().unwrap(), frame);
}

#[test]
fn aggregate_pixels_after_intrinsic_reflow_reject_before_allocating_rasters() {
    let calls = Rc::new(Calls::default());
    let limits = fenestra_ui::Limits::default().text_limits(TextLimits::new(1000, 5000, 1000));
    let mut app = Application::with_limits_and_text_engine(
        view(),
        Size::new(80, 100),
        limits,
        Engine(calls.clone()),
    )
    .unwrap();
    let old = app.raster().unwrap();
    let count = calls.raster.get();
    // A narrow paragraph gains enough wrapped lines to exceed aggregate area.
    let result = app.set_text(
        "article",
        "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz",
    );
    assert!(matches!(
        result,
        Err(Error::Text(TextError::LimitExceeded {
            resource: "text pixels",
            ..
        }))
    ));
    assert_eq!(calls.raster.get(), count);
    assert_eq!(app.generation(), 0);
    assert_eq!(app.raster().unwrap(), old);
}
