use super::local_transform_support::{
    Placement, VIEWPORT, expect_transform, expect_valid, identity_values, input, limits, node,
    node_field, root, set_field, transform, validate,
};
use crate::error::SpatialErrorLocationV2;
use crate::model::SpatialScalarV2;
use crate::numeric_error::SpatialTransformErrorKindV2;
use crate::vocabulary::SpatialTransformScalarFieldV2;

#[test]
fn every_layout_and_free_scalar_rejects_both_domain_sides() {
    for placement in [Placement::Layout, Placement::Free] {
        for field in SpatialTransformScalarFieldV2::ALL {
            for raw in [SpatialScalarV2::MIN_RAW - 1, SpatialScalarV2::MAX_RAW + 1] {
                let mut values = identity_values();
                set_field(&mut values, field, raw);
                let fixture = input(vec![root(), node(placement, 1, 0, transform(values))]);
                let item_limits = match placement {
                    Placement::Layout => limits(1, 2, 2),
                    Placement::Free => limits(0, 0, 0),
                };

                expect_transform(
                    validate(&fixture, VIEWPORT, item_limits),
                    SpatialTransformErrorKindV2::ScalarOutOfDomain(field),
                    SpatialErrorLocationV2::NodeField {
                        index: 1,
                        field: node_field(field),
                    },
                );
            }
        }
    }
}

#[test]
fn every_layout_and_free_scalar_accepts_both_inclusive_edges() {
    for placement in [Placement::Layout, Placement::Free] {
        for field in SpatialTransformScalarFieldV2::ALL {
            for raw in [SpatialScalarV2::MIN_RAW, SpatialScalarV2::MAX_RAW] {
                let mut values = identity_values();
                set_field(&mut values, field, raw);
                let fixture = input(vec![root(), node(placement, 1, 0, transform(values))]);
                let item_limits = match placement {
                    Placement::Layout => limits(1, 2, 2),
                    Placement::Free => limits(0, 0, 0),
                };

                expect_valid(validate(&fixture, VIEWPORT, item_limits));
            }
        }
    }
}

#[test]
fn adjacent_scalar_faults_follow_the_registered_field_order() {
    let fields = SpatialTransformScalarFieldV2::ALL;
    for index in 0..fields.len() - 1 {
        let mut values = identity_values();
        set_field(&mut values, fields[index], SpatialScalarV2::MAX_RAW + 1);
        set_field(&mut values, fields[index + 1], SpatialScalarV2::MIN_RAW - 1);
        let fixture = input(vec![
            root(),
            node(Placement::Layout, 1, 0, transform(values)),
        ]);

        expect_transform(
            validate(&fixture, VIEWPORT, limits(1, 2, 2)),
            SpatialTransformErrorKindV2::ScalarOutOfDomain(fields[index]),
            SpatialErrorLocationV2::NodeField {
                index: 1,
                field: node_field(fields[index]),
            },
        );
    }
}
