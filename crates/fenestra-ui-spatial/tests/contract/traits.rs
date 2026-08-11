use std::fmt::Debug;
use std::hash::Hash;
use std::panic::{RefUnwindSafe, UnwindSafe};

use super::super::*;

#[test]
fn first_slice_values_and_vocabularies_preserve_required_traits() {
    assert_key_traits::<SpatialNodeKeyV2>();
    assert_value_traits::<(
        SpatialViewportV2,
        SpatialScalarV2,
        SpatialPointV2,
        SpatialOffsetV2,
        Affine2V2,
        SpatialLocalTransformV2,
        SpatialAnchorV2,
        SpatialContainerV2,
        SpatialLayoutPlacementV2,
        SpatialFreePlacementV2,
        SpatialNodeV2,
    )>();
    assert_value_traits::<(
        SpatialAnchorComponentV2,
        SpatialAnchorTargetKindV2,
        SpatialAnchorTargetV2,
        SpatialPlacementKindV2,
        SpatialPlacementV2,
        SpatialAxisV2,
        SpatialExtentV2,
        SpatialContainerErrorKindV2,
        SpatialLayoutDimensionErrorKindV2,
        SpatialInputErrorKindV2,
        SpatialDependencyErrorKindV2,
        SpatialNodeFieldV2,
    )>();
    assert_value_traits::<(SpatialErrorLocationV2, SpatialLimitKindV2, SpatialLimitsV2)>();
    assert_copy::<SpatialTopologyInputV2<'static>>();

    assert_auto_traits::<SpatialTopologyInputV2<'static>>();
    assert_auto_traits::<SpatialLimitsV2>();
    assert_auto_traits::<SpatialErrorLocationV2>();
}

fn assert_key_traits<T>()
where
    T: Clone + Copy + Debug + Eq + Hash + Ord + PartialEq,
{
}

fn assert_value_traits<T>()
where
    T: Clone + Copy + Debug + Eq + PartialEq,
{
}

fn assert_copy<T: Clone + Copy>() {}

fn assert_auto_traits<T>()
where
    T: Send + Sync + Unpin + UnwindSafe + RefUnwindSafe,
{
}
