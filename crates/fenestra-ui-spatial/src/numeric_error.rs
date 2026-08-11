use crate::vocabulary::{
    SpatialAffineComponentV2, SpatialTransformScalarFieldV2, SpatialTransformStageV2,
};

/// Closed failure vocabulary for raw and composed transforms.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialTransformErrorKindV2 {
    /// One authored scalar lies outside the registered numeric domain.
    ScalarOutOfDomain(SpatialTransformScalarFieldV2),
    /// One authored affine has an exact zero determinant.
    SingularTransform,
    /// One composed stage rounded to an exact zero determinant.
    ComposedTransformSingular(SpatialTransformStageV2),
}

impl SpatialTransformErrorKindV2 {
    /// Every transform failure in deterministic validation order.
    pub const ALL: [Self; 12] = transform_errors();
}

/// Closed contextual operation vocabulary for spatial arithmetic failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialArithmeticOperationV2 {
    /// Horizontal target-anchor addition.
    TargetOffsetX,
    /// Vertical target-anchor addition.
    TargetOffsetY,
    /// Horizontal self-anchor subtraction.
    SelfSubtractionX,
    /// Vertical self-anchor subtraction.
    SelfSubtractionY,
    /// Horizontal island-to-scene translation.
    IslandTranslationX,
    /// Vertical island-to-scene translation.
    IslandTranslationY,
    /// Horizontal base-box far edge.
    BaseFarX,
    /// Vertical base-box far edge.
    BaseFarY,
    /// Horizontal parent-local delta.
    ParentDeltaX,
    /// Vertical parent-local delta.
    ParentDeltaY,
    /// One affine component at one composed transform stage.
    Affine {
        /// Transform stage being composed.
        stage: SpatialTransformStageV2,
        /// Matrix component being calculated.
        component: SpatialAffineComponentV2,
    },
    /// Minimum horizontal conservative bound.
    AabbMinX,
    /// Minimum vertical conservative bound.
    AabbMinY,
    /// Maximum horizontal conservative bound.
    AabbMaxX,
    /// Maximum vertical conservative bound.
    AabbMaxY,
}

impl SpatialArithmeticOperationV2 {
    /// Every arithmetic operation in deterministic diagnostic order.
    pub const ALL: [Self; 32] = arithmetic_operations();
}

const fn transform_errors() -> [SpatialTransformErrorKindV2; 12] {
    use SpatialTransformErrorKindV2::{
        ComposedTransformSingular, ScalarOutOfDomain, SingularTransform,
    };

    [
        ScalarOutOfDomain(SpatialTransformScalarFieldV2::AffineA),
        ScalarOutOfDomain(SpatialTransformScalarFieldV2::AffineB),
        ScalarOutOfDomain(SpatialTransformScalarFieldV2::AffineC),
        ScalarOutOfDomain(SpatialTransformScalarFieldV2::AffineD),
        ScalarOutOfDomain(SpatialTransformScalarFieldV2::AffineTx),
        ScalarOutOfDomain(SpatialTransformScalarFieldV2::AffineTy),
        ScalarOutOfDomain(SpatialTransformScalarFieldV2::TransformOriginX),
        ScalarOutOfDomain(SpatialTransformScalarFieldV2::TransformOriginY),
        SingularTransform,
        ComposedTransformSingular(SpatialTransformStageV2::About),
        ComposedTransformSingular(SpatialTransformStageV2::Placed),
        ComposedTransformSingular(SpatialTransformStageV2::World),
    ]
}

const fn arithmetic_operations() -> [SpatialArithmeticOperationV2; 32] {
    use SpatialAffineComponentV2::{A, B, C, D, Tx, Ty};
    use SpatialArithmeticOperationV2::{
        AabbMaxX, AabbMaxY, AabbMinX, AabbMinY, Affine, BaseFarX, BaseFarY, IslandTranslationX,
        IslandTranslationY, ParentDeltaX, ParentDeltaY, SelfSubtractionX, SelfSubtractionY,
        TargetOffsetX, TargetOffsetY,
    };
    use SpatialTransformStageV2::{About, Placed, World};

    [
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
    ]
}
