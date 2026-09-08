use std::cell::Cell;
use std::rc::Rc;

use fenestra_ui::{
    Application, Color, Element, Error, Raster, Size, Style, TextEngine, TextError, TextLayout,
    TextLimits, TextMetrics, TextRequest, TextStyle, View,
};

struct TestEngine {
    calls: Rc<Cell<usize>>,
}

impl TextEngine for TestEngine {
    fn layout(&mut self, request: TextRequest<'_>) -> Result<TextLayout, TextError> {
        self.calls.set(self.calls.get() + 1);
        if request.text() == "reject" {
            return Err(TextError::Engine("fixture rejection".into()));
        }
        let size = if request.text() == "wrong_size" {
            Size::new(1, 1)
        } else {
            request.size()
        };
        let mut bytes = vec![0; size.width() as usize * size.height() as usize * 4];
        let color = request.style().color_value().to_rgba8();
        if !request.text().is_empty() {
            for pixel in bytes.chunks_exact_mut(4).take(request.text().len()) {
                pixel.copy_from_slice(&color);
            }
        }
        TextLayout::new(
            Raster::new(size, bytes).unwrap(),
            TextMetrics::new(request.text().len() as f32, 1.0, 1, request.text().len(), 0),
        )
    }
}

fn scene() -> View {
    View::new(
        "text",
        Element::column("root")
            .style(Style::new().width(12).height(8).padding(1))
            .child(
                Element::text("label", "abc")
                    .style(Style::new().width(8).height(3).input(true))
                    .text_style(TextStyle::new().color(Color::rgba8(255, 0, 0, 255))),
            ),
    )
}

fn application() -> (Application, Rc<Cell<usize>>) {
    let calls = Rc::new(Cell::new(0));
    let app = Application::with_text_engine(
        scene(),
        Size::new(12, 8),
        TestEngine {
            calls: calls.clone(),
        },
    )
    .unwrap();
    (app, calls)
}

fn pixel(app: &Application, x: usize, y: usize) -> [u8; 4] {
    let frame = app.raster().unwrap();
    frame.bytes()[(y * 12 + x) * 4..(y * 12 + x + 1) * 4]
        .try_into()
        .unwrap()
}

#[test]
fn text_is_painted_at_committed_bounds_and_updates_transactionally() {
    let (mut app, calls) = application();
    assert_eq!(pixel(&app, 1, 1), [255, 0, 0, 255]);
    assert_eq!(pixel(&app, 4, 1), [0, 0, 0, 0]);
    assert_eq!(app.text("label").unwrap(), "abc");
    assert_eq!(app.text_metrics("label").unwrap().glyphs(), 3);
    assert_eq!(app.hit_test(1, 1), Some("label"));
    let generation = app.generation();
    app.set_text("label", "abcd").unwrap();
    assert_eq!(app.generation(), generation + 1);
    assert_eq!(pixel(&app, 4, 1), [255, 0, 0, 255]);
    let frame = app.raster().unwrap();
    assert!(app.set_text("label", "reject").is_err());
    assert_eq!(app.text("label").unwrap(), "abcd");
    assert_eq!(app.generation(), generation + 1);
    assert_eq!(app.raster().unwrap(), frame);
    let count = calls.get();
    app.set_text("label", "abcd").unwrap();
    assert_eq!(calls.get(), count);
    assert_eq!(app.generation(), generation + 1);
}

#[test]
fn translation_reuses_shaping_but_dimensions_and_text_style_rebuild_it() {
    let (mut app, calls) = application();
    let count = calls.get();
    app.set_style("root", app.style("root").unwrap().padding(2))
        .unwrap();
    assert_eq!(calls.get(), count);
    assert_eq!(pixel(&app, 2, 2), [255, 0, 0, 255]);
    assert_eq!(pixel(&app, 1, 1), [0, 0, 0, 0]);
    app.set_size("label", 6, 4).unwrap();
    assert_eq!(calls.get(), count + 1);
    app.set_text_style(
        "label",
        TextStyle::new().color(Color::rgba8(0, 255, 0, 255)),
    )
    .unwrap();
    assert_eq!(calls.get(), count + 2);
    assert_eq!(pixel(&app, 2, 2), [0, 255, 0, 255]);
}

#[test]
fn missing_engine_and_text_resource_budgets_fail_explicitly() {
    assert!(matches!(
        Application::new(scene(), Size::new(12, 8)),
        Err(Error::Text(TextError::EngineUnavailable))
    ));
    let limits = fenestra_ui::Limits::default().text_limits(TextLimits::new(2, 100, 100));
    assert!(
        Application::with_limits_and_text_engine(
            scene(),
            Size::new(12, 8),
            limits,
            TestEngine {
                calls: Rc::new(Cell::new(0))
            }
        )
        .is_err()
    );
    let (mut app, _) = application();
    let before = app.raster().unwrap();
    let generation = app.generation();
    assert!(
        app.set_text_style("label", TextStyle::new().font_size(0))
            .is_err()
    );
    assert_eq!(app.raster().unwrap(), before);
    assert_eq!(app.generation(), generation);
}

#[test]
fn engine_output_dimensions_and_glyph_budget_are_checked_before_publication() {
    let limits = fenestra_ui::Limits::default().text_limits(TextLimits::new(100, 100, 3));
    let calls = Rc::new(Cell::new(0));
    let mut app = Application::with_limits_and_text_engine(
        scene(),
        Size::new(12, 8),
        limits,
        TestEngine {
            calls: calls.clone(),
        },
    )
    .unwrap();
    let frame = app.raster().unwrap();
    for (text, error) in [
        ("wrong_size", TextError::InvalidRaster),
        (
            "abcd",
            TextError::LimitExceeded {
                resource: "text glyphs",
                actual: 4,
                limit: 3,
            },
        ),
    ] {
        assert_eq!(app.set_text("label", text), Err(Error::Text(error)));
        assert_eq!(app.generation(), 0);
        assert_eq!(app.text("label").unwrap(), "abc");
        assert_eq!(app.text_metrics("label").unwrap().glyphs(), 3);
        assert_eq!(app.raster().unwrap(), frame);
    }
    assert_eq!(calls.get(), 3);
}

#[test]
fn aggregate_text_budgets_reject_before_invoking_the_engine() {
    let view = View::new(
        "aggregate",
        Element::column("root")
            .child(Element::text("first", "ab").style(Style::new().width(3).height(2)))
            .child(Element::text("second", "cd").style(Style::new().width(3).height(2))),
    );
    let calls = Rc::new(Cell::new(0));
    for limits in [TextLimits::new(3, 12, 10), TextLimits::new(4, 11, 10)] {
        let result = Application::with_limits_and_text_engine(
            view.clone(),
            Size::new(1, 1),
            fenestra_ui::Limits::default().text_limits(limits),
            TestEngine {
                calls: calls.clone(),
            },
        );
        assert!(matches!(
            result,
            Err(Error::Text(TextError::LimitExceeded { .. }))
        ));
        assert_eq!(calls.get(), 0);
    }
    let mut app = Application::with_limits_and_text_engine(
        view,
        Size::new(1, 1),
        fenestra_ui::Limits::default().text_limits(TextLimits::new(4, 12, 10)),
        TestEngine {
            calls: calls.clone(),
        },
    )
    .unwrap();
    assert_eq!(calls.get(), 2);
    assert!(app.set_text("first", "abc").is_err());
    assert!(app.set_size("second", 4, 2).is_err());
    assert_eq!(calls.get(), 2);
    assert_eq!(app.generation(), 0);
    assert_eq!(app.text("first").unwrap(), "ab");
}

#[test]
fn zero_area_text_retains_content_and_recovers_when_its_box_grows() {
    let (mut app, calls) = application();
    app.set_size("label", 0, 3).unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(app.text_metrics("label").unwrap().glyphs(), 0);
    assert_eq!(app.hit_test(1, 1), None);
    assert_eq!(pixel(&app, 1, 1), [0; 4]);
    app.set_text("label", "abcd").unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(app.text("label").unwrap(), "abcd");
    app.set_size("label", 8, 3).unwrap();
    assert_eq!(calls.get(), 2);
    assert_eq!(app.text_metrics("label").unwrap().glyphs(), 4);
    assert_eq!(pixel(&app, 4, 1), [255, 0, 0, 255]);
    let generation = app.generation();
    app.set_text_style("label", app.text_style("label").unwrap())
        .unwrap();
    app.set_size("label", 8, 3).unwrap();
    app.resize(app.size()).unwrap();
    assert_eq!(calls.get(), 2);
    assert_eq!(app.generation(), generation);
}

#[test]
fn text_obeys_ancestor_backgrounds_sibling_order_and_viewport_clipping() {
    let view = View::new(
        "overlap",
        Element::column("root")
            .style(
                Style::new()
                    .width(12)
                    .height(8)
                    .background(Color::rgba8(0, 0, 255, 255)),
            )
            .child(
                Element::column("overflowing")
                    .style(Style::new().width(12).height(0))
                    .child(
                        Element::text("label", "abcdefghijklm")
                            .style(Style::new().width(16).height(2).input(true))
                            .text_style(TextStyle::new().color(Color::rgba8(255, 0, 0, 255))),
                    ),
            )
            .child(
                Element::rect("cover").style(
                    Style::new()
                        .width(1)
                        .height(1)
                        .background(Color::rgba8(0, 255, 0, 255)),
                ),
            ),
    );
    let mut app = Application::with_text_engine(
        view,
        Size::new(12, 8),
        TestEngine {
            calls: Rc::new(Cell::new(0)),
        },
    )
    .unwrap();
    assert_eq!(pixel(&app, 0, 0), [0, 255, 0, 255]);
    assert_eq!(pixel(&app, 1, 0), [255, 0, 0, 255]);
    assert_eq!(pixel(&app, 11, 0), [255, 0, 0, 255]);
    assert_eq!(pixel(&app, 0, 1), [0, 0, 255, 255]);
    assert_eq!(app.hit_test(12, 0), None);
    app.resize(Size::new(2, 1)).unwrap();
    assert_eq!(
        app.raster().unwrap().bytes(),
        &[0, 255, 0, 255, 255, 0, 0, 255]
    );
}

#[test]
fn failed_runtime_layout_after_shaping_keeps_text_geometry_and_generation() {
    let view = View::new(
        "overflow",
        Element::row("root")
            .style(Style::new().width(1).height(1))
            .child(Element::rect("spacer").style(Style::new().width(i32::MAX - 2).height(1)))
            .child(Element::text("label", "ab").style(Style::new().width(2).height(1))),
    );
    let calls = Rc::new(Cell::new(0));
    let mut app = Application::with_text_engine(
        view,
        Size::new(1, 1),
        TestEngine {
            calls: calls.clone(),
        },
    )
    .unwrap();
    let bounds = app.bounds("label").unwrap();
    let metrics = app.text_metrics("label").unwrap();
    let raster = app.raster().unwrap();
    assert!(matches!(
        app.set_size("label", 4, 1),
        Err(Error::Runtime(_))
    ));
    assert_eq!(calls.get(), 2);
    assert_eq!(app.bounds("label").unwrap(), bounds);
    assert_eq!(app.text_metrics("label").unwrap(), metrics);
    assert_eq!(app.raster().unwrap(), raster);
    assert_eq!(app.generation(), 0);
}
