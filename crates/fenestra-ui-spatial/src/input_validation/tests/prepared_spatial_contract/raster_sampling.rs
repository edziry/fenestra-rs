use super::raster_support::*;
use crate::coverage::SpatialFillRuleV2;

#[test]
fn sixteen_scene_lattice_samples_are_averaged_after_exact_coverage() {
    use super::super::validated_shape_support::rect_values;

    let snapshot = snapshot(owned_fixture(
        viewport(1, 1),
        root_and_owners(1, 1, 1),
        vec![rect_values(0, 1, 0, 0, S / 4, S)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![solid(0, color(255, 255, 255, 255))],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![fill(1, 0, 0, 0, 255, None, SpatialFillRuleV2::NonZero)],
    ));
    let raster = snapshot
        .rasterize_reference(limits(1))
        .expect("one reference pixel");
    assert_raster(&raster, 1, 1, &[64, 64, 64, 64]);
}

#[test]
fn source_over_occurs_for_each_sample_before_the_sixteen_sample_average() {
    use super::super::validated_shape_support::rect_values;

    let snapshot = snapshot(owned_fixture(
        viewport(1, 1),
        root_and_owners(1, 1, 1),
        vec![
            rect_values(0, 1, 0, 0, S / 2, S),
            rect_values(1, 1, 0, 0, S / 2, S),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![
            solid(0, color(255, 0, 0, 128)),
            solid(1, color(0, 255, 0, 128)),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![
            fill(1, 0, 0, 0, 255, None, SpatialFillRuleV2::NonZero),
            fill(1, 1, 1, 1, 255, None, SpatialFillRuleV2::NonZero),
        ],
    ));
    let raster = snapshot
        .rasterize_reference(limits(1))
        .expect("composited reference pixel");
    assert_raster(&raster, 1, 1, &[32, 64, 0, 96]);
}

#[test]
fn packed_rows_and_channels_have_the_exact_rgba_layout() {
    use super::super::validated_shape_support::rect_values;

    let snapshot = snapshot(owned_fixture(
        viewport(2, 2),
        root_and_owners(1, 2, 2),
        vec![
            rect_values(0, 1, 0, 0, S, S),
            rect_values(1, 1, S, 0, S, S),
            rect_values(2, 1, 0, S, S, S),
            rect_values(3, 1, S, S, S, S),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![
            solid(0, color(255, 0, 0, 255)),
            solid(1, color(0, 255, 0, 255)),
            solid(2, color(0, 0, 255, 255)),
            solid(3, color(255, 255, 255, 255)),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![
            fill(1, 0, 0, 0, 255, None, SpatialFillRuleV2::NonZero),
            fill(1, 1, 1, 1, 255, None, SpatialFillRuleV2::NonZero),
            fill(1, 2, 2, 2, 255, None, SpatialFillRuleV2::NonZero),
            fill(1, 3, 3, 3, 255, None, SpatialFillRuleV2::NonZero),
        ],
    ));
    let raster = snapshot
        .rasterize_reference(limits(4))
        .expect("packed reference rows");
    assert_raster(
        &raster,
        2,
        2,
        &[
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ],
    );
}
