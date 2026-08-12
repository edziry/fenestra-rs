use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::panic::{RefUnwindSafe, UnwindSafe};

use crate::*;

use super::source::all_source;

macro_rules! assert_not {
    ($type:ty, $trait_name:ident) => {
        let _ = <$type as $trait_name<_>>::marker;
    };
}

macro_rules! assert_no_recipe_extras {
    ($($type:ty),+ $(,)?) => {
        $(
            assert_not!($type, AmbiguousIfDefault);
            assert_not!($type, AmbiguousIfDisplay);
            assert_not!($type, AmbiguousIfError);
            assert_not!($type, AmbiguousIfHash);
            assert_not!($type, AmbiguousIfOrd);
            assert_not!($type, AmbiguousIfPartialOrd);
        )+
    };
}

#[test]
fn versions_and_symbols_have_only_the_registered_key_traits() {
    assert_key::<SpatialFormatVersion>();
    assert_key::<SpatialNodeSymbolV2>();
    assert_key::<SpatialShapeSymbolV2>();
    assert_key::<SpatialBrushSymbolV2>();
    assert_key::<SpatialClipSymbolV2>();
    assert_key::<SpatialImageSymbolV2>();
    assert_not!(SpatialFormatVersion, AmbiguousIfDefault);
    assert_not!(SpatialFormatVersion, AmbiguousIfDisplay);
    assert_not!(SpatialFormatVersion, AmbiguousIfError);
    assert_not!(SpatialNodeSymbolV2, AmbiguousIfDefault);
    assert_not!(SpatialNodeSymbolV2, AmbiguousIfDisplay);
    assert_not!(SpatialNodeSymbolV2, AmbiguousIfError);
    assert_not!(SpatialShapeSymbolV2, AmbiguousIfDefault);
    assert_not!(SpatialShapeSymbolV2, AmbiguousIfDisplay);
    assert_not!(SpatialShapeSymbolV2, AmbiguousIfError);
    assert_not!(SpatialBrushSymbolV2, AmbiguousIfDefault);
    assert_not!(SpatialBrushSymbolV2, AmbiguousIfDisplay);
    assert_not!(SpatialBrushSymbolV2, AmbiguousIfError);
    assert_not!(SpatialClipSymbolV2, AmbiguousIfDefault);
    assert_not!(SpatialClipSymbolV2, AmbiguousIfDisplay);
    assert_not!(SpatialClipSymbolV2, AmbiguousIfError);
    assert_not!(SpatialImageSymbolV2, AmbiguousIfDefault);
    assert_not!(SpatialImageSymbolV2, AmbiguousIfDisplay);
    assert_not!(SpatialImageSymbolV2, AmbiguousIfError);
}

#[test]
fn copy_recipes_have_only_the_registered_value_traits() {
    assert_copy_value::<SpatialFieldV2<i32>>();
    assert_copy_value::<SpatialBindingV2<i32>>();
    assert_copy_value::<SpatialAxisV2>();
    assert_copy_value::<SpatialAnchorComponentV2>();
    assert_copy_value::<SpatialFillRuleV2>();
    assert_copy_value::<SpatialNodeParentV2>();
    assert_copy_value::<SpatialAnchorTargetRecipeV2>();
    assert_copy_value::<SpatialClipAddressV2>();
    assert_copy_value::<SpatialPointRecipeV2>();
    assert_copy_value::<SpatialPaddingRecipeV2>();
    assert_copy_value::<SpatialDimensionRecipeV2>();
    assert_copy_value::<SpatialTransformRecipeV2>();
    assert_copy_value::<SpatialViewportContainerV2>();
    assert_copy_value::<SpatialContainerRecipeV2>();
    assert_copy_value::<SpatialLayoutPlacementRecipeV2>();
    assert_copy_value::<SpatialFreePlacementRecipeV2>();
    assert_copy_value::<SpatialPlacementRecipeV2>();
    assert_copy_value::<SpatialPathVerbRecipeV2>();
    assert_copy_value::<SpatialPolygonPointV2>();
    assert_copy_value::<SpatialGradientStopV2>();
    assert_copy_value::<SpatialCoverageRecipeV2>();
    assert_copy_value::<SpatialClipDeclarationV2>();
    assert_copy_value::<SpatialPaintRecipeV2>();
    assert_copy_value::<SpatialHitRecipeV2>();
    assert_copy_value::<SpatialSemanticRecipeV2>();
    assert_copy_value::<SpatialValidationLimitsV2>();

    assert_no_recipe_extras!(
        SpatialFieldV2<i32>,
        SpatialBindingV2<i32>,
        SpatialAxisV2,
        SpatialAnchorComponentV2,
        SpatialFillRuleV2,
        SpatialNodeParentV2,
        SpatialAnchorTargetRecipeV2,
        SpatialClipAddressV2,
        SpatialPointRecipeV2,
        SpatialPaddingRecipeV2,
        SpatialDimensionRecipeV2,
        SpatialTransformRecipeV2,
        SpatialViewportContainerV2,
        SpatialContainerRecipeV2,
        SpatialLayoutPlacementRecipeV2,
        SpatialFreePlacementRecipeV2,
        SpatialPlacementRecipeV2,
        SpatialPathVerbRecipeV2,
        SpatialPolygonPointV2,
        SpatialGradientStopV2,
        SpatialCoverageRecipeV2,
        SpatialClipDeclarationV2,
        SpatialPaintRecipeV2,
        SpatialHitRecipeV2,
        SpatialSemanticRecipeV2,
        SpatialValidationLimitsV2,
    );
}

#[test]
fn owned_recipes_have_only_the_registered_owned_value_traits() {
    assert_owned_value::<SpatialShapeGeometryV2>();
    assert_owned_value::<SpatialShapeDeclarationV2>();
    assert_owned_value::<SpatialBrushContentV2>();
    assert_owned_value::<SpatialBrushDeclarationV2>();
    assert_owned_value::<SpatialImageDeclarationV2>();
    assert_owned_value::<SpatialNodeDeclarationV2>();
    assert_owned_value::<SpatialProgramV2>();

    assert_no_recipe_extras!(
        SpatialShapeGeometryV2,
        SpatialShapeDeclarationV2,
        SpatialBrushContentV2,
        SpatialBrushDeclarationV2,
        SpatialImageDeclarationV2,
        SpatialNodeDeclarationV2,
        SpatialProgramV2,
    );
    assert_not!(SpatialShapeGeometryV2, AmbiguousIfCopy);
    assert_not!(SpatialShapeDeclarationV2, AmbiguousIfCopy);
    assert_not!(SpatialBrushContentV2, AmbiguousIfCopy);
    assert_not!(SpatialBrushDeclarationV2, AmbiguousIfCopy);
    assert_not!(SpatialImageDeclarationV2, AmbiguousIfCopy);
    assert_not!(SpatialNodeDeclarationV2, AmbiguousIfCopy);
    assert_not!(SpatialProgramV2, AmbiguousIfCopy);
}

#[test]
fn generic_wrappers_remain_conditionally_derived_and_unconstrained() {
    assert_owned_value::<SpatialFieldV2<String>>();
    assert_owned_value::<SpatialBindingV2<String>>();
    assert_not!(SpatialFieldV2<String>, AmbiguousIfCopy);
    assert_not!(SpatialBindingV2<String>, AmbiguousIfCopy);
    assert_no_recipe_extras!(SpatialFieldV2<String>, SpatialBindingV2<String>);
}

#[test]
fn validated_program_has_only_clone_redacted_debug_and_runtime_traits() {
    assert_validated::<ValidatedSpatialProgramV2>();
    assert_not!(ValidatedSpatialProgramV2, AmbiguousIfCopy);
    assert_not!(ValidatedSpatialProgramV2, AmbiguousIfDefault);
    assert_not!(ValidatedSpatialProgramV2, AmbiguousIfDisplay);
    assert_not!(ValidatedSpatialProgramV2, AmbiguousIfError);
    assert_not!(ValidatedSpatialProgramV2, AmbiguousIfEq);
    assert_not!(ValidatedSpatialProgramV2, AmbiguousIfHash);
    assert_not!(ValidatedSpatialProgramV2, AmbiguousIfOrd);
    assert_not!(ValidatedSpatialProgramV2, AmbiguousIfPartialEq);
    assert_not!(ValidatedSpatialProgramV2, AmbiguousIfPartialOrd);

    let source = all_source();
    assert!(source.contains("impl fmt::Debug for ValidatedSpatialProgramV2"));
    assert!(source.contains("formatter.write_str(\"ValidatedSpatialProgramV2(..)\")"));
}

fn assert_key<T>()
where
    T: Clone
        + Copy
        + Debug
        + Eq
        + Hash
        + Ord
        + PartialEq
        + PartialOrd
        + Send
        + Sync
        + Unpin
        + UnwindSafe
        + RefUnwindSafe
        + 'static,
{
}

fn assert_copy_value<T>()
where
    T: Clone
        + Copy
        + Debug
        + Eq
        + PartialEq
        + Send
        + Sync
        + Unpin
        + UnwindSafe
        + RefUnwindSafe
        + 'static,
{
}

fn assert_owned_value<T>()
where
    T: Clone + Debug + Eq + PartialEq + Send + Sync + Unpin + UnwindSafe + RefUnwindSafe + 'static,
{
}

fn assert_validated<T>()
where
    T: Clone + Debug + Send + Sync + Unpin + UnwindSafe + RefUnwindSafe + 'static,
{
}

macro_rules! negative_trait {
    ($trait_name:ident, $bound:path) => {
        trait $trait_name<A> {
            fn marker() {}
        }
        impl<T: ?Sized> $trait_name<()> for T {}
        impl<T: ?Sized + $bound> $trait_name<u8> for T {}
    };
}

negative_trait!(AmbiguousIfCopy, Copy);
negative_trait!(AmbiguousIfDefault, Default);
negative_trait!(AmbiguousIfDisplay, Display);
negative_trait!(AmbiguousIfError, std::error::Error);
negative_trait!(AmbiguousIfEq, Eq);
negative_trait!(AmbiguousIfHash, Hash);
negative_trait!(AmbiguousIfOrd, Ord);
negative_trait!(AmbiguousIfPartialEq, PartialEq);
negative_trait!(AmbiguousIfPartialOrd, PartialOrd);
