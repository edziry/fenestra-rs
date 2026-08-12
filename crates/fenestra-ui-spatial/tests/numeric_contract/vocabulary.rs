use std::fmt::Debug;
use std::panic::{RefUnwindSafe, UnwindSafe};

use super::*;

#[test]
fn numeric_vocabularies_are_closed_in_diagnostic_order() {
    use SpatialAffineComponentV2::{A, B, C, D, Tx, Ty};
    use SpatialTransformScalarFieldV2::{
        AffineA, AffineB, AffineC, AffineD, AffineTx, AffineTy, TransformOriginX, TransformOriginY,
    };
    use SpatialTransformStageV2::{About, Placed, World};

    assert_eq!(SpatialTransformStageV2::ALL, [About, Placed, World]);
    assert_eq!(SpatialAffineComponentV2::ALL, [A, B, C, D, Tx, Ty]);
    assert_eq!(
        SpatialTransformScalarFieldV2::ALL,
        [
            AffineA,
            AffineB,
            AffineC,
            AffineD,
            AffineTx,
            AffineTy,
            TransformOriginX,
            TransformOriginY,
        ]
    );

    let expected_transform_errors = [
        SpatialTransformErrorKindV2::ScalarOutOfDomain(AffineA),
        SpatialTransformErrorKindV2::ScalarOutOfDomain(AffineB),
        SpatialTransformErrorKindV2::ScalarOutOfDomain(AffineC),
        SpatialTransformErrorKindV2::ScalarOutOfDomain(AffineD),
        SpatialTransformErrorKindV2::ScalarOutOfDomain(AffineTx),
        SpatialTransformErrorKindV2::ScalarOutOfDomain(AffineTy),
        SpatialTransformErrorKindV2::ScalarOutOfDomain(TransformOriginX),
        SpatialTransformErrorKindV2::ScalarOutOfDomain(TransformOriginY),
        SpatialTransformErrorKindV2::SingularTransform,
        SpatialTransformErrorKindV2::ComposedTransformSingular(About),
        SpatialTransformErrorKindV2::ComposedTransformSingular(Placed),
        SpatialTransformErrorKindV2::ComposedTransformSingular(World),
    ];
    assert_eq!(SpatialTransformErrorKindV2::ALL, expected_transform_errors);
    for value in SpatialTransformStageV2::ALL {
        exhaust_stage(value);
    }
    for value in SpatialAffineComponentV2::ALL {
        exhaust_component(value);
    }
    for value in SpatialTransformScalarFieldV2::ALL {
        exhaust_transform_field(value);
    }
    for value in SpatialTransformErrorKindV2::ALL {
        exhaust_transform_error(value);
    }
}

#[test]
fn arithmetic_operations_flatten_stage_then_component() {
    use SpatialAffineComponentV2::{A, B, C, D, Tx, Ty};
    use SpatialArithmeticOperationV2::{
        AabbMaxX, AabbMaxY, AabbMinX, AabbMinY, Affine, BaseFarX, BaseFarY, IslandTranslationX,
        IslandTranslationY, ParentDeltaX, ParentDeltaY, SelfSubtractionX, SelfSubtractionY,
        TargetOffsetX, TargetOffsetY,
    };
    use SpatialTransformStageV2::{About, Placed, World};

    let expected = [
        TargetOffsetX,
        TargetOffsetY,
        SelfSubtractionX,
        SelfSubtractionY,
        IslandTranslationX,
        IslandTranslationY,
        BaseFarX,
        BaseFarY,
        ParentDeltaX,
        ParentDeltaY,
        Affine {
            stage: About,
            component: A,
        },
        Affine {
            stage: About,
            component: B,
        },
        Affine {
            stage: About,
            component: C,
        },
        Affine {
            stage: About,
            component: D,
        },
        Affine {
            stage: About,
            component: Tx,
        },
        Affine {
            stage: About,
            component: Ty,
        },
        Affine {
            stage: Placed,
            component: A,
        },
        Affine {
            stage: Placed,
            component: B,
        },
        Affine {
            stage: Placed,
            component: C,
        },
        Affine {
            stage: Placed,
            component: D,
        },
        Affine {
            stage: Placed,
            component: Tx,
        },
        Affine {
            stage: Placed,
            component: Ty,
        },
        Affine {
            stage: World,
            component: A,
        },
        Affine {
            stage: World,
            component: B,
        },
        Affine {
            stage: World,
            component: C,
        },
        Affine {
            stage: World,
            component: D,
        },
        Affine {
            stage: World,
            component: Tx,
        },
        Affine {
            stage: World,
            component: Ty,
        },
        AabbMinX,
        AabbMinY,
        AabbMaxX,
        AabbMaxY,
    ];
    assert_eq!(SpatialArithmeticOperationV2::ALL, expected);
    for value in SpatialArithmeticOperationV2::ALL {
        exhaust_arithmetic(value);
    }
}

#[test]
fn numeric_exports_preserve_value_and_runtime_traits() {
    fn assert_traits<T>()
    where
        T: Clone + Copy + Debug + Eq + PartialEq + Send + Sync + Unpin + UnwindSafe + RefUnwindSafe,
    {
    }

    assert_traits::<SpatialAabbV2>();
    assert_traits::<SpatialAffineComponentV2>();
    assert_traits::<SpatialArithmeticOperationV2>();
    assert_traits::<SpatialTransformErrorKindV2>();
    assert_traits::<SpatialTransformScalarFieldV2>();
    assert_traits::<SpatialTransformStageV2>();
}

fn exhaust_stage(value: SpatialTransformStageV2) {
    match value {
        SpatialTransformStageV2::About
        | SpatialTransformStageV2::Placed
        | SpatialTransformStageV2::World => {}
    }
}

fn exhaust_component(value: SpatialAffineComponentV2) {
    match value {
        SpatialAffineComponentV2::A
        | SpatialAffineComponentV2::B
        | SpatialAffineComponentV2::C
        | SpatialAffineComponentV2::D
        | SpatialAffineComponentV2::Tx
        | SpatialAffineComponentV2::Ty => {}
    }
}

fn exhaust_transform_field(value: SpatialTransformScalarFieldV2) {
    match value {
        SpatialTransformScalarFieldV2::AffineA
        | SpatialTransformScalarFieldV2::AffineB
        | SpatialTransformScalarFieldV2::AffineC
        | SpatialTransformScalarFieldV2::AffineD
        | SpatialTransformScalarFieldV2::AffineTx
        | SpatialTransformScalarFieldV2::AffineTy
        | SpatialTransformScalarFieldV2::TransformOriginX
        | SpatialTransformScalarFieldV2::TransformOriginY => {}
    }
}

fn exhaust_transform_error(value: SpatialTransformErrorKindV2) {
    match value {
        SpatialTransformErrorKindV2::ScalarOutOfDomain(field) => exhaust_transform_field(field),
        SpatialTransformErrorKindV2::SingularTransform => {}
        SpatialTransformErrorKindV2::ComposedTransformSingular(stage) => exhaust_stage(stage),
    }
}

fn exhaust_arithmetic(value: SpatialArithmeticOperationV2) {
    match value {
        SpatialArithmeticOperationV2::TargetOffsetX
        | SpatialArithmeticOperationV2::TargetOffsetY
        | SpatialArithmeticOperationV2::SelfSubtractionX
        | SpatialArithmeticOperationV2::SelfSubtractionY
        | SpatialArithmeticOperationV2::IslandTranslationX
        | SpatialArithmeticOperationV2::IslandTranslationY
        | SpatialArithmeticOperationV2::BaseFarX
        | SpatialArithmeticOperationV2::BaseFarY
        | SpatialArithmeticOperationV2::ParentDeltaX
        | SpatialArithmeticOperationV2::ParentDeltaY
        | SpatialArithmeticOperationV2::AabbMinX
        | SpatialArithmeticOperationV2::AabbMinY
        | SpatialArithmeticOperationV2::AabbMaxX
        | SpatialArithmeticOperationV2::AabbMaxY => {}
        SpatialArithmeticOperationV2::Affine { stage, component } => {
            exhaust_stage(stage);
            exhaust_component(component);
        }
    }
}
