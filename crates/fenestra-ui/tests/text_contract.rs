use fenestra_ui::{
    Application, Element, Error, Raster, Size, Style, TextError, TextLayout, TextLimits,
    TextMetrics, TextRequest, TextStyle, View,
};

#[test]
fn requests_reject_invalid_typography_dimensions_and_preparation_budgets() {
    let limits = TextLimits::new(2, 2, 10);
    for style in [
        TextStyle::new().font_size(0),
        TextStyle::new().font_size(513),
        TextStyle::new().line_height(0),
        TextStyle::new().line_height(2049),
    ] {
        assert!(matches!(
            TextRequest::new("a", style, Size::new(1, 1), limits),
            Err(TextError::InvalidStyle { .. })
        ));
    }
    for size in [Size::new(0, 1), Size::new(1, 0), Size::new(u32::MAX, 1)] {
        assert!(matches!(
            TextRequest::new("a", TextStyle::new(), size, limits),
            Err(TextError::InvalidRaster)
        ));
    }
    for (text, size) in [("abc", Size::new(1, 1)), ("a", Size::new(3, 1))] {
        assert!(matches!(
            TextRequest::new(text, TextStyle::new(), size, limits),
            Err(TextError::LimitExceeded { .. })
        ));
    }
    let request = TextRequest::new(
        "ab",
        TextStyle::new().font_size(512).line_height(2048),
        Size::new(2, 1),
        limits,
    )
    .unwrap();
    assert_eq!(request.text(), "ab");
    assert_eq!(request.size(), Size::new(2, 1));
}

#[test]
fn output_rejects_nonfinite_metrics_and_nonpremultiplied_channels() {
    let raster = Raster::new(Size::new(1, 1), vec![10, 20, 30, 40]).unwrap();
    for metrics in [
        TextMetrics::new(f32::NAN, 0.0, 1, 1, 0),
        TextMetrics::new(0.0, f32::INFINITY, 1, 1, 0),
        TextMetrics::new(-1.0, 1.0, 1, 1, 0),
        TextMetrics::new(1.0, -1.0, 1, 1, 0),
        TextMetrics::new(1.0, 1.0, 1, 1, 2),
    ] {
        assert_eq!(
            TextLayout::new(raster.clone(), metrics),
            Err(TextError::InvalidMetrics)
        );
    }
    let metrics = TextMetrics::new(1.0, 1.0, 1, 1, 0);
    for bytes in [vec![255, 0, 0, 0], vec![0, 11, 0, 10], vec![0, 0, 20, 19]] {
        assert_eq!(
            TextLayout::new(Raster::new(Size::new(1, 1), bytes).unwrap(), metrics),
            Err(TextError::InvalidRaster)
        );
    }
    assert!(TextLayout::new(raster, metrics).is_ok());
}

#[test]
fn typed_text_leaves_reject_container_properties_and_nontext_typography() {
    for element in [
        Element::text("label", "a").child(Element::rect("nested")),
        Element::text("label", "a").style(Style::new().padding(1)),
        Element::text("label", "a").style(Style::new().gap(1)),
        Element::rect("label").text_style(TextStyle::new()),
        Element::row("label").text_style(TextStyle::new()),
    ] {
        assert!(matches!(
            Application::new(View::new("invalid", element), Size::new(1, 1)),
            Err(Error::InvalidElement { .. })
        ));
    }
    let mut app =
        Application::new(View::new("plain", Element::rect("root")), Size::new(1, 1)).unwrap();
    assert!(matches!(
        app.set_text("root", "a"),
        Err(Error::InvalidElement { .. })
    ));
    assert!(matches!(
        app.set_text_style("root", TextStyle::new()),
        Err(Error::InvalidElement { .. })
    ));
    assert!(matches!(
        app.text("missing"),
        Err(Error::UnknownNode { .. })
    ));
    assert!(app.text_metrics("root").is_err());
    assert_eq!(app.generation(), 0);
}
