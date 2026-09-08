use fenestra_ui::{
    Raster, Size, TextEngine, TextLayout, TextMetrics, TextRequest, TextViewportRequest,
};

use crate::*;

#[derive(Default)]
struct LayoutOnly {
    calls: usize,
}

impl TextEngine for LayoutOnly {
    fn layout(&mut self, request: TextRequest<'_>) -> Result<TextLayout, TextError> {
        self.calls += 1;
        assert_eq!(request.text(), "ab");
        assert_eq!(request.style().font_size_value(), 20);
        TextLayout::new(
            Raster::new(request.size(), [12, 24, 36, 128].repeat(6)).unwrap(),
            TextMetrics::new(2.0, 3.0, 1, 2, 0),
        )
    }
}

#[test]
fn layout_only_engines_keep_old_methods_and_delegate_only_compatible_viewports() {
    let mut engine = LayoutOnly::default();
    let base = TextRequest::new(
        "ab",
        TextStyle::new().font_size(20),
        Size::new(2, 3),
        TextLimits::new(2, 6, 2),
    )
    .unwrap();
    let legacy = engine.layout(base).unwrap();
    let compatible = TextViewportRequest::new(base, Some(2), 0, 0).unwrap();
    assert_eq!(engine.layout_viewport(compatible).unwrap(), legacy);
    assert_eq!(engine.calls, 2);

    for (width, x, y) in [
        (None, 0, 0),
        (Some(1), 0, 0),
        (Some(2), 1, 0),
        (Some(2), 0, 1),
    ] {
        let request = TextViewportRequest::new(base, width, x, y).unwrap();
        assert_eq!(
            engine.layout_viewport(request),
            Err(TextError::ViewportUnavailable)
        );
    }
    assert_eq!(
        engine.geometry(request("ab", selection(0, 0), TextGeometryQuery::Current).unwrap()),
        Err(TextError::GeometryUnavailable),
    );
    assert_eq!(
        engine.measure(TextMeasureRequest::new("ab", base.style(), None, base.limits()).unwrap()),
        Err(TextError::MeasurementUnavailable),
    );
    assert_eq!(engine.calls, 2);
}
