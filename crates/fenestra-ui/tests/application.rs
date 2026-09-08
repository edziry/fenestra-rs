use fenestra_ui::{Application, Color, Element, Error, Limits, Raster, Size, Style, View};

fn panel() -> View {
    let card = Style::new()
        .width(20)
        .height(30)
        .background(Color::rgba8(40, 80, 120, 255))
        .input(true);
    View::new(
        "hello",
        Element::row("root")
            .style(
                Style::new()
                    .width(100)
                    .height(60)
                    .padding(4)
                    .gap(8)
                    .background(Color::rgba8(8, 16, 24, 255)),
            )
            .child(Element::rect("first").style(card))
            .child(Element::rect("second").style(card)),
    )
}

fn pixel(raster: &Raster, x: usize, y: usize) -> [u8; 4] {
    let start = (y * raster.size().width() as usize + x) * 4;
    raster.bytes()[start..start + 4].try_into().unwrap()
}

#[test]
fn named_views_render_and_hit_test_without_internal_ids() {
    let app = Application::new(panel(), Size::new(120, 80)).unwrap();
    assert_eq!(app.generation(), 0);
    assert_eq!(app.node_count(), 3);
    assert_eq!(
        app.node_names().collect::<Vec<_>>(),
        ["root", "first", "second"]
    );
    assert_eq!(app.hit_test(4, 4), Some("first"));
    assert_eq!(app.hit_test(32, 4), Some("second"));
    assert_eq!(app.hit_test(24, 4), None);
    assert_eq!(app.hit_test(-1, -1), None);
    let raster = app.raster().unwrap();
    assert_eq!(pixel(&raster, 0, 0), [8, 16, 24, 255]);
    assert_eq!(pixel(&raster, 4, 4), [40, 80, 120, 255]);
    assert_eq!(pixel(&raster, 119, 79), [0, 0, 0, 0]);
}

#[test]
fn style_and_size_changes_publish_coherent_layout_and_pixels() {
    let mut app = Application::new(panel(), Size::new(120, 80)).unwrap();
    app.set_background("first", Color::rgba8(255, 192, 32, 255))
        .unwrap();
    let selected = app.raster().unwrap();
    assert_eq!(pixel(&selected, 4, 4), [255, 192, 32, 255]);
    assert_eq!(pixel(&selected, 32, 4), [40, 80, 120, 255]);
    app.set_size("first", 30, 30).unwrap();
    assert_eq!(app.hit_test(32, 4), Some("first"));
    assert_eq!(app.hit_test(35, 4), None);
    assert_eq!(app.hit_test(42, 4), Some("second"));
    let resized = app.raster().unwrap();
    assert_eq!(pixel(&resized, 32, 4), [255, 192, 32, 255]);
    assert_eq!(pixel(&resized, 42, 4), [40, 80, 120, 255]);
    assert_eq!(app.generation(), 2);
    app.resize(Size::new(160, 100)).unwrap();
    assert_eq!(app.size(), Size::new(160, 100));
    assert_eq!(app.raster().unwrap().bytes().len(), 160 * 100 * 4);
}

#[test]
fn invalid_changes_and_oversized_viewports_are_atomic() {
    let mut app =
        Application::with_limits(panel(), Size::new(120, 80), Limits::new(3, 2, 9600)).unwrap();
    let initial = app.raster().unwrap();
    assert!(matches!(
        app.set_size("first", -1, 30),
        Err(Error::InvalidStyle { .. })
    ));
    assert!(matches!(
        app.set_background("missing", Color::rgba8(0, 0, 0, 0)),
        Err(Error::UnknownNode { .. })
    ));
    assert!(matches!(
        app.resize(Size::new(120, 81)),
        Err(Error::LimitExceeded {
            resource: "pixels",
            ..
        })
    ));
    assert!(matches!(
        app.resize(Size::new(u32::MAX, u32::MAX)),
        Err(Error::InvalidViewport { .. })
    ));
    assert!(app.resize(Size::new(0, 10)).is_err());
    assert_eq!(app.raster().unwrap(), initial);
    assert_eq!(app.generation(), 0);
    assert_eq!(app.size(), Size::new(120, 80));
}

#[test]
fn no_op_changes_preserve_generation_and_input_can_be_disabled() {
    let mut app = Application::new(panel(), Size::new(120, 80)).unwrap();
    app.set_style("first", app.style("first").unwrap()).unwrap();
    assert_eq!(app.generation(), 0);
    app.set_style("first", app.style("first").unwrap().input(false))
        .unwrap();
    assert_eq!(app.hit_test(4, 4), None);
    assert_eq!(app.hit_test(32, 4), Some("second"));
    assert_eq!(app.generation(), 1);
}

#[test]
fn raster_constructor_rejects_invalid_byte_counts() {
    assert_eq!(
        Raster::new(Size::new(2, 2), vec![0; 15]),
        Err(Error::InvalidRaster)
    );
    assert!(Raster::new(Size::new(0, 2), vec![]).is_err());
}

#[test]
fn hit_tests_clip_overflowing_elements_to_the_viewport() {
    let app = Application::new(panel(), Size::new(10, 10)).unwrap();
    assert_eq!(app.hit_test(9, 9), Some("first"));
    assert_eq!(app.hit_test(10, 9), None);
    assert_eq!(app.hit_test(9, 10), None);
}
