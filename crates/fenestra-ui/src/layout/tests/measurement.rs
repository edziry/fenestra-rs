use super::*;

fn metrics(width: f32, height: f32) -> TextMetrics {
    TextMetrics::new(width, height, 1, 1, 0)
}

#[test]
fn final_width_precedes_wrapped_height_and_auto_cross_fill() {
    let nodes = [
        node("root", Row, style(Fill(1), Auto).padding(1).gap(1), &[1, 2]),
        node("text", Text, style(Fill(1), Auto), &[]),
        node("side", Rect, style(Px(3), Fill(5)), &[]),
    ];
    let mut calls = Vec::new();
    let sizes = resolve(&nodes, Size::new(16, 1), |index, width| {
        calls.push((index, width));
        Ok(metrics(9.2, 11.2))
    })
    .unwrap();
    assert_eq!(calls, [(1, Some(10))]);
    assert_eq!(
        sizes,
        [Size::new(16, 14), Size::new(10, 12), Size::new(3, 12)]
    );
}

#[test]
fn fill_children_contribute_intrinsic_sizes_inside_auto_parents_without_cycles() {
    let nodes = [
        node("root", Row, style(Auto, Auto).gap(1), &[1, 2]),
        node("first", Text, style(Fill(1), Fill(1)), &[]),
        node("second", Text, style(Fill(2), Fill(1)), &[]),
    ];
    let mut calls = Vec::new();
    let sizes = resolve(&nodes, Size::new(1, 1), |index, width| {
        calls.push((index, width));
        Ok(match (index, width) {
            (1, None) => metrics(9.0, 4.0),
            (2, None) => metrics(12.0, 6.0),
            (1, Some(7)) => metrics(7.0, 8.0),
            (2, Some(14)) => metrics(12.0, 6.0),
            _ => panic!("unexpected measurement: {index}, {width:?}"),
        })
    })
    .unwrap();
    assert_eq!(sizes, [Size::new(22, 8), Size::new(7, 8), Size::new(14, 8)]);
    assert_eq!(calls, [(1, None), (2, None), (1, Some(7)), (2, Some(14))]);
}

#[test]
fn text_auto_width_hugs_and_clamps_without_implicit_viewport_shrink() {
    let mut text = style(Auto, Auto);
    text.min_width = 4;
    text.max_width = 8;
    text.min_height = 3;
    text.max_height = 5;
    let nodes = [node("text", Text, text, &[])];
    let mut calls = Vec::new();
    let sizes = resolve(&nodes, Size::new(1, 1), |_, width| {
        calls.push(width);
        Ok(metrics(15.1, 8.1))
    })
    .unwrap();
    assert_eq!(sizes, [Size::new(8, 5)]);
    assert_eq!(calls, [None, Some(8)]);
}

#[test]
fn zero_width_text_has_zero_auto_height_without_measurement() {
    let nodes = [
        node("root", Row, style(Px(1), Auto), &[1, 2]),
        node("fixed", Rect, style(Px(2), Px(1)), &[]),
        node("text", Text, style(Fill(1), Auto), &[]),
    ];
    assert_eq!(
        resolve(&nodes, Size::new(1, 1), no_measure).unwrap(),
        [Size::new(1, 1), Size::new(2, 1), Size::new(0, 0),]
    );
}

#[test]
fn fixed_text_ancestors_do_not_force_unneeded_intrinsic_measurement() {
    let nodes = [
        node("root", Column, style(Auto, Auto), &[1]),
        node("fixed", Row, style(Px(20), Px(10)), &[2]),
        node("text", Text, style(Fill(1), Fill(1)), &[]),
    ];
    assert_eq!(
        resolve(&nodes, Size::new(1, 1), no_measure).unwrap(),
        [Size::new(20, 10), Size::new(20, 10), Size::new(20, 10),]
    );
}

#[test]
fn invalid_measurements_and_measurement_errors_are_rejected() {
    let nodes = [node("text", Text, style(Auto, Auto), &[])];
    for bad in [
        metrics(f32::NAN, 1.0),
        metrics(-1.0, 1.0),
        metrics(1.0, f32::INFINITY),
        TextMetrics::new(1.0, 1.0, 1, 0, 1),
    ] {
        assert!(matches!(
            resolve(&nodes, Size::new(1, 1), |_, _| Ok(bad)),
            Err(Error::Text(_))
        ));
    }
    assert_eq!(
        resolve(&nodes, Size::new(1, 1), |_, _| Ok(metrics(
            i32::MAX as f32,
            1.0
        ))),
        Err(Error::Text(crate::TextError::InvalidMetrics)),
    );
    let error = Error::Text(crate::TextError::Engine("measurement rejected".into()));
    assert_eq!(
        resolve(&nodes, Size::new(1, 1), |_, _| Err(error.clone())),
        Err(error)
    );
}

#[test]
fn auto_column_measures_fill_heights_before_weighted_allocation() {
    let nodes = [
        node("root", Column, style(Px(20), Auto).gap(1), &[1, 2]),
        node("first", Text, style(Fill(1), Fill(2)), &[]),
        node("second", Text, style(Fill(1), Fill(1)), &[]),
    ];
    let mut calls = Vec::new();
    let sizes = resolve(&nodes, Size::new(1, 1), |index, width| {
        calls.push((index, width));
        Ok(metrics(10.0, index as f32 * 10.0))
    })
    .unwrap();
    assert_eq!(
        sizes,
        [Size::new(20, 31), Size::new(20, 20), Size::new(20, 10)]
    );
    assert_eq!(calls, [(1, Some(20)), (2, Some(20))]);
}

#[test]
fn nested_intrinsic_requests_are_memoized_once_per_text_and_axis() {
    let children: Vec<_> = (0..63)
        .map(|index| vec![index + 1])
        .chain([vec![]])
        .collect();
    let nodes: Vec<_> = children
        .iter()
        .enumerate()
        .map(|(index, children)| {
            node(
                "nested",
                if index == 63 { Text } else { Column },
                style(Auto, Auto),
                children,
            )
        })
        .collect();
    let mut calls = Vec::new();
    let sizes = resolve(&nodes, Size::new(1, 1), |index, width| {
        calls.push((index, width));
        Ok(metrics(7.2, 9.5))
    })
    .unwrap();
    assert_eq!(sizes, vec![Size::new(8, 10); 64]);
    assert_eq!(calls, [(63, None), (63, Some(8))]);
}
