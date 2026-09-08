use super::*;

fn allocated(available: u32, constraints: [(u32, i32, i32); 3]) -> Vec<u32> {
    let mut nodes = vec![node("root", Row, style(Fill(1), Px(1)), &[1, 2, 3])];
    for (weight, minimum, maximum) in constraints {
        let mut fill = style(Fill(weight), Px(1));
        fill.min_width = minimum;
        fill.max_width = maximum;
        nodes.push(node("fill", Rect, fill, &[]));
    }
    resolve(&nodes, Size::new(available, 1), no_measure)
        .unwrap()
        .iter()
        .skip(1)
        .map(|size| size.width())
        .collect()
}

#[test]
fn unequal_fractional_remainders_precede_authored_tie_breaking() {
    assert_eq!(
        allocated(7, [(1, 0, 10), (2, 0, 10), (3, 0, 10)]),
        [1, 2, 4]
    );
    assert_eq!(allocated(2, [(1, 0, 10); 3]), [1, 1, 0]);
}

#[test]
fn capped_shares_are_redistributed_until_remaining_children_fit() {
    assert_eq!(
        allocated(25, [(1, 0, 1), (2, 0, 3), (4, 0, 100)]),
        [1, 3, 21]
    );
    assert_eq!(allocated(100, [(1, 0, 2), (2, 0, 4), (3, 0, 0)]), [2, 4, 0]);
    assert_eq!(
        allocated(20, [(1, 5, 20), (1, 0, 20), (1, 0, 20)]),
        [10, 5, 5]
    );
}

#[test]
fn maximum_weights_and_extents_use_wide_arithmetic() {
    assert_eq!(
        allocated(i32::MAX as u32, [(65_535, 0, i32::MAX); 3]),
        [715_827_883, 715_827_882, 715_827_882],
    );
    assert_eq!(
        allocated(1, [(65_535, i32::MAX, i32::MAX); 3]),
        [i32::MAX as u32; 3],
    );
}

#[test]
fn bounded_repartitions_conserve_pixels_and_honor_each_constraint() {
    let constraints = [(1, 2, 5), (3, 1, 9), (2, 0, 7)];
    for available in 0..=25 {
        let sizes = allocated(available, constraints);
        assert_eq!(sizes.iter().sum::<u32>(), available.clamp(3, 21));
        for (&size, (_, minimum, maximum)) in sizes.iter().zip(constraints) {
            assert!((minimum as u32..=maximum as u32).contains(&size));
        }
    }
}
