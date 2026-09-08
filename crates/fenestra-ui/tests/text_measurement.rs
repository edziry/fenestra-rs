use fenestra_ui::{
    Size, TextEngine, TextError, TextLayout, TextLimits, TextMeasureRequest, TextMetrics,
    TextRequest, TextStyle,
};

fn request() -> TextMeasureRequest<'static> {
    TextMeasureRequest::new("abc", TextStyle::new(), None, TextLimits::new(3, 0, 3)).unwrap()
}

#[test]
fn measurement_request_validates_bytes_typography_and_optional_width_without_pixels() {
    let request = request();
    assert_eq!(request.text(), "abc");
    assert_eq!(request.width(), None);
    assert_eq!(request.style(), TextStyle::new());
    assert_eq!(request.limits().max_pixels(), 0);
    assert!(TextMeasureRequest::new("abc", TextStyle::new(), Some(0), request.limits()).is_ok());
    assert!(
        TextMeasureRequest::new(
            "abc",
            TextStyle::new(),
            Some(i32::MAX as u32),
            request.limits()
        )
        .is_ok()
    );
    assert!(matches!(
        TextMeasureRequest::new("abcd", TextStyle::new(), None, request.limits()),
        Err(TextError::LimitExceeded {
            resource: "text bytes",
            actual: 4,
            limit: 3
        })
    ));
    assert!(matches!(
        TextMeasureRequest::new("abc", TextStyle::new().font_size(0), None, request.limits()),
        Err(TextError::InvalidStyle { .. })
    ));
    assert_eq!(
        TextMeasureRequest::new("abc", TextStyle::new(), Some(u32::MAX), request.limits()).err(),
        Some(TextError::InvalidMeasurementWidth { width: u32::MAX })
    );
}

#[test]
fn standalone_metrics_validate_counts_bounds_and_checked_ceil() {
    let metrics = TextMetrics::new(12.25, 27.5, 2, 3, 0);
    metrics.validate_measurement(request()).unwrap();
    assert_eq!(metrics.ceil_size(), Ok(Size::new(13, 28)));
    assert_eq!(
        TextMetrics::new(0.0, 0.0, 0, 0, 0).ceil_size(),
        Ok(Size::new(0, 0))
    );
    for metrics in [
        TextMetrics::new(f32::NAN, 1.0, 1, 1, 0),
        TextMetrics::new(1.0, f32::INFINITY, 1, 1, 0),
        TextMetrics::new(-1.0, 1.0, 1, 1, 0),
        TextMetrics::new(1.0, -1.0, 1, 1, 0),
        TextMetrics::new(1.0, 1.0, 1, 1, 2),
        TextMetrics::new(i32::MAX as f32, 1.0, 1, 1, 0),
        TextMetrics::new(1.0, i32::MAX as f32, 1, 1, 0),
    ] {
        assert_eq!(
            metrics.validate_measurement(request()),
            Err(TextError::InvalidMetrics)
        );
        assert_eq!(metrics.ceil_size(), Err(TextError::InvalidMetrics));
    }
    assert!(matches!(
        TextMetrics::new(1.0, 1.0, 1, 4, 0).validate_measurement(request()),
        Err(TextError::LimitExceeded {
            resource: "text glyphs",
            actual: 4,
            limit: 3
        })
    ));
    let largest = f32::from_bits((i32::MAX as f32).to_bits() - 1);
    assert_eq!(
        TextMetrics::new(largest, 0.0, 1, 1, 0).ceil_size(),
        Ok(Size::new(2_147_483_520, 0))
    );
}

struct LayoutOnly;

impl TextEngine for LayoutOnly {
    fn layout(&mut self, _: TextRequest<'_>) -> Result<TextLayout, TextError> {
        Err(TextError::FontUnavailable)
    }
}

#[test]
fn layout_only_engines_remain_compatible_and_report_measurement_unavailable() {
    assert_eq!(
        LayoutOnly.measure(request()),
        Err(TextError::MeasurementUnavailable)
    );
}
