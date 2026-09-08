use fenestra_ui::{Application, Color, Element, Error, Limits, Size, Style, View};

fn application(root: Element) -> Result<Application, Error> {
    Application::new(View::new("test_view", root), Size::new(640, 480))
}

#[test]
fn ordinary_nested_view_exceeds_the_old_fixture_node_and_depth_bounds() {
    let mut root = Element::column("branch_0");
    for depth in 1..40 {
        root = Element::column(format!("branch_{depth}")).child(root);
    }
    for index in 0..12 {
        root = root.child(
            Element::rect(format!("leaf_{index}")).style(
                Style::new()
                    .width(10)
                    .height(10)
                    .background(Color::rgba8(20, 40, 60, 255)),
            ),
        );
    }
    assert!(application(root).is_ok());
}

#[test]
fn caller_capacity_allows_more_than_the_spatial_conformance_profile() {
    let mut root = Element::row("root");
    for index in 0..300 {
        root = root
            .child(Element::rect(format!("leaf_{index}")).style(Style::new().width(1).height(1)));
    }
    assert!(
        Application::with_limits(
            View::new("wide_view", root),
            Size::new(640, 1),
            Limits::new(301, 2, 640),
        )
        .is_ok()
    );
}

#[test]
fn rejects_duplicate_names_across_separate_subtrees() {
    let root = Element::row("root")
        .child(Element::column("left").child(Element::rect("duplicate")))
        .child(Element::column("right").child(Element::rect("duplicate")));
    assert!(matches!(application(root), Err(Error::DuplicateName { name }) if name == "duplicate"));
}

#[test]
fn rejects_invalid_node_identifiers() {
    for name in [
        "",
        "two words",
        "1leading",
        "has-hyphen",
        "non_ascii_\u{e9}",
    ] {
        assert!(matches!(
            application(Element::rect(name)),
            Err(Error::InvalidName { .. })
        ));
    }
    assert!(application(Element::rect("_valid_12")).is_ok());
}

#[test]
fn rejects_negative_style_values_with_node_and_property() {
    for (property, style) in [
        ("width", Style::new().width(-1)),
        ("height", Style::new().height(-1)),
        ("padding", Style::new().padding(-1)),
        ("gap", Style::new().gap(-1)),
    ] {
        assert!(matches!(
            application(Element::column("target").style(style)),
            Err(Error::InvalidStyle { node, property: actual, value: -1 })
                if node == "target" && actual == property
        ));
    }
}

#[test]
fn rectangles_reject_container_configuration() {
    for root in [
        Element::rect("target").child(Element::rect("child")),
        Element::rect("target").style(Style::new().padding(1)),
        Element::rect("target").style(Style::new().gap(1)),
    ] {
        assert!(
            matches!(application(root), Err(Error::InvalidElement { node, .. }) if node == "target")
        );
    }
}

#[test]
fn custom_node_and_depth_limits_are_inclusive() {
    let root = Element::column("root").child(Element::rect("child"));
    let view = View::new("test_view", root);
    let viewport = Size::new(64, 64);
    assert!(Application::with_limits(view.clone(), viewport, Limits::new(2, 2, 4096)).is_ok());
    assert!(matches!(
        Application::with_limits(view.clone(), viewport, Limits::new(1, 2, 4096)),
        Err(Error::LimitExceeded {
            resource: "nodes",
            limit: 1,
            actual: 2
        })
    ));
    assert!(matches!(
        Application::with_limits(view, viewport, Limits::new(2, 1, 4096)),
        Err(Error::LimitExceeded {
            resource: "depth",
            limit: 1,
            actual: 2
        })
    ));
}
