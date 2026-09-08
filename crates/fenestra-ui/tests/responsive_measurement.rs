use std::cell::Cell;
use std::rc::Rc;

use fenestra_ui::{
    Application, Dimension, Element, Error, Limits, Raster, Size, Style, TextEngine, TextError,
    TextLayout, TextLimits, TextMeasureRequest, TextMetrics, TextRequest, View,
};

#[derive(Clone, Copy)]
enum Fault {
    None,
    InvalidMetrics,
    ExcessGlyphs,
    RasterMismatch,
}

struct State {
    fault: Cell<Fault>,
    measures: Cell<usize>,
    rasters: Cell<usize>,
}

impl State {
    fn new(fault: Fault) -> Rc<Self> {
        Rc::new(Self {
            fault: Cell::new(fault),
            measures: Cell::new(0),
            rasters: Cell::new(0),
        })
    }
}

struct Engine(Rc<State>);

impl TextEngine for Engine {
    fn measure(&mut self, request: TextMeasureRequest<'_>) -> Result<TextMetrics, TextError> {
        self.0.measures.set(self.0.measures.get() + 1);
        Ok(match self.0.fault.get() {
            Fault::InvalidMetrics => TextMetrics::new(f32::NAN, 2.0, 1, 3, 0),
            Fault::ExcessGlyphs => TextMetrics::new(3.0, 2.0, 1, 4, 0),
            _ => metrics(request.text(), 2.0),
        })
    }

    fn layout(&mut self, request: TextRequest<'_>) -> Result<TextLayout, TextError> {
        self.0.rasters.set(self.0.rasters.get() + 1);
        let height = if matches!(self.0.fault.get(), Fault::RasterMismatch) {
            3.0
        } else {
            2.0
        };
        let size = request.size();
        let bytes = vec![255; size.width() as usize * size.height() as usize * 4];
        TextLayout::new(
            Raster::new(size, bytes).unwrap(),
            metrics(request.text(), height),
        )
    }
}

fn metrics(text: &str, height: f32) -> TextMetrics {
    TextMetrics::new(text.len() as f32, height, 1, text.len(), 0)
}

fn view(height: Dimension) -> View {
    View::new(
        "measurement",
        Element::text("label", "abc").style(Style::new().width(12).height_mode(height).input(true)),
    )
}

#[test]
fn constructor_rejects_unvalidated_measurements_before_raster_work() {
    for (fault, expected) in [
        (Fault::InvalidMetrics, TextError::InvalidMetrics),
        (
            Fault::ExcessGlyphs,
            TextError::LimitExceeded {
                resource: "text glyphs",
                actual: 4,
                limit: 3,
            },
        ),
    ] {
        let state = State::new(fault);
        let result = Application::with_limits_and_text_engine(
            view(Dimension::Auto),
            Size::new(20, 20),
            Limits::default().text_limits(TextLimits::new(3, 100, 3)),
            Engine(state.clone()),
        );
        assert_eq!(result.err(), Some(Error::Text(expected)));
        assert_eq!(state.measures.get(), 1);
        assert_eq!(state.rasters.get(), 0);
    }
}

#[test]
fn rejected_measurement_preserves_text_and_frame_and_can_be_retried() {
    let state = State::new(Fault::None);
    let mut app = Application::with_text_engine(
        view(Dimension::Auto),
        Size::new(20, 20),
        Engine(state.clone()),
    )
    .unwrap();
    let frame = app.raster().unwrap();
    let bounds = app.bounds("label").unwrap();
    let accepted_metrics = app.text_metrics("label").unwrap();
    state.fault.set(Fault::InvalidMetrics);
    assert_eq!(
        app.set_text("label", "abcd"),
        Err(Error::Text(TextError::InvalidMetrics))
    );
    assert_eq!(app.generation(), 0);
    assert_eq!(app.text("label").unwrap(), "abc");
    assert_eq!(app.text_metrics("label").unwrap(), accepted_metrics);
    assert_eq!(app.bounds("label").unwrap(), bounds);
    assert_eq!(app.raster().unwrap(), frame);
    assert_eq!(state.rasters.get(), 1);
    state.fault.set(Fault::None);
    app.set_text("label", "abcd").unwrap();
    assert_eq!(app.generation(), 1);
    assert_eq!(app.text("label").unwrap(), "abcd");
    assert_eq!(state.measures.get(), 3);
    assert_eq!(state.rasters.get(), 2);
}

#[test]
fn inconsistent_raster_drops_candidate_measurement_cache_and_preserves_policy() {
    let state = State::new(Fault::None);
    let mut app = Application::with_text_engine(
        view(Dimension::Auto),
        Size::new(20, 20),
        Engine(state.clone()),
    )
    .unwrap();
    let accepted_style = app.style("label").unwrap();
    let next_style = accepted_style.width(14);
    let frame = app.raster().unwrap();
    let bounds = app.bounds("label").unwrap();
    state.fault.set(Fault::RasterMismatch);
    assert_eq!(
        app.set_style("label", next_style),
        Err(Error::Text(TextError::InconsistentMeasurement))
    );
    assert_eq!(app.generation(), 0);
    assert_eq!(app.style("label").unwrap(), accepted_style);
    assert_eq!(app.bounds("label").unwrap(), bounds);
    assert_eq!(app.raster().unwrap(), frame);
    assert_eq!(app.hit_test(11, 1), Some("label"));
    assert_eq!(app.hit_test(13, 1), None);
    state.fault.set(Fault::None);
    app.set_style("label", next_style).unwrap();
    assert_eq!(app.bounds("label").unwrap().width(), 14);
    assert_eq!(app.style("label").unwrap(), next_style);
    assert_eq!(app.generation(), 1);
    assert_eq!(state.measures.get(), 3);
    assert_eq!(state.rasters.get(), 3);
}

struct LayoutOnly(Engine);

impl TextEngine for LayoutOnly {
    fn layout(&mut self, request: TextRequest<'_>) -> Result<TextLayout, TextError> {
        self.0.layout(request)
    }
}

#[test]
fn existing_fixed_text_engine_rejects_intrinsic_policy_atomically() {
    let state = State::new(Fault::None);
    let mut app = Application::with_text_engine(
        view(Dimension::Px(2)),
        Size::new(20, 20),
        LayoutOnly(Engine(state.clone())),
    )
    .unwrap();
    let style = app.style("label").unwrap();
    let frame = app.raster().unwrap();
    assert_eq!(
        app.set_style("label", style.height_mode(Dimension::Auto)),
        Err(Error::Text(TextError::MeasurementUnavailable))
    );
    assert_eq!(app.style("label").unwrap(), style);
    assert_eq!(app.generation(), 0);
    assert_eq!(app.raster().unwrap(), frame);
    assert_eq!(state.rasters.get(), 1);
}
