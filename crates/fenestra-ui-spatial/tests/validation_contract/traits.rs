use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::panic::{RefUnwindSafe, UnwindSafe};

use crate::*;

macro_rules! assert_not_debug {
    ($type:ty) => {
        let _ = <$type as AmbiguousIfDebug<_>>::marker;
    };
}

macro_rules! assert_not_default {
    ($type:ty) => {
        let _ = <$type as AmbiguousIfDefault<_>>::marker;
    };
}

macro_rules! assert_not_display {
    ($type:ty) => {
        let _ = <$type as AmbiguousIfDisplay<_>>::marker;
    };
}

macro_rules! assert_not_hash {
    ($type:ty) => {
        let _ = <$type as AmbiguousIfHash<_>>::marker;
    };
}

macro_rules! assert_not_ord {
    ($type:ty) => {
        let _ = <$type as AmbiguousIfOrd<_>>::marker;
    };
}

macro_rules! assert_not_partial_eq {
    ($type:ty) => {
        let _ = <$type as AmbiguousIfPartialEq<_>>::marker;
    };
}

macro_rules! assert_not_partial_ord {
    ($type:ty) => {
        let _ = <$type as AmbiguousIfPartialOrd<_>>::marker;
    };
}

#[test]
fn aggregate_input_has_only_registered_borrowed_view_traits() {
    type StaticInput = SpatialInputV2<'static>;

    assert_borrowed::<StaticInput>();
    assert_not_debug!(StaticInput);
    assert_not_default!(StaticInput);
    assert_not_display!(StaticInput);
    assert_not_partial_eq!(StaticInput);
    assert_not_hash!(StaticInput);
    assert_not_ord!(StaticInput);
    assert_not_partial_ord!(StaticInput);
}

#[test]
fn diagnostic_enums_have_registered_value_traits() {
    assert_value::<SpatialColorChannelV2>();
    assert_value::<SpatialPathFieldV2>();
    assert_value::<SpatialPathVerbFieldV2>();
    assert_value::<SpatialShapeFieldV2>();
    assert_value::<SpatialPolygonPointFieldV2>();
    assert_value::<SpatialBrushFieldV2>();
    assert_value::<SpatialGradientStopFieldV2>();
    assert_value::<SpatialImageFieldV2>();
    assert_value::<SpatialClipFieldV2>();
    assert_value::<SpatialPaintFieldV2>();
    assert_value::<SpatialHitFieldV2>();
    assert_value::<SpatialSemanticFieldV2>();
    assert_value::<SpatialOutputTableV2>();
    assert_value::<SpatialOutputFieldV2>();
    assert_value::<SpatialKeyedContentTableV2>();
    assert_value::<SpatialPayloadTableV2>();
    assert_value::<SpatialContentReferenceV2>();
    assert_value::<SpatialOrderedItemTableV2>();
    assert_value::<SpatialPathGrammarErrorV2>();
    assert_value::<SpatialShapeErrorV2>();
    assert_value::<SpatialStrokeErrorV2>();
    assert_value::<SpatialGradientErrorV2>();
    assert_value::<SpatialImageErrorV2>();
    assert_value::<SpatialClipErrorV2>();
    assert_value::<SpatialContentErrorKindV2>();
    assert_value::<SpatialLayoutErrorKindV2>();
    assert_value::<SpatialOutputErrorKindV2>();
    assert_value::<SpatialResolveErrorKindV2>();
    assert_value::<SpatialErrorLocationV2>();
}

#[test]
fn stored_resolver_error_has_only_registered_error_traits() {
    assert_error::<SpatialResolveErrorV2>();
    assert_not_default!(SpatialResolveErrorV2);
    assert_not_hash!(SpatialResolveErrorV2);
    assert_not_ord!(SpatialResolveErrorV2);
    assert_not_partial_ord!(SpatialResolveErrorV2);
}

fn assert_borrowed<T>()
where
    T: Clone + Copy + Send + Sync + Unpin + UnwindSafe + RefUnwindSafe,
{
}

fn assert_value<T>()
where
    T: Clone + Copy + Debug + Eq + PartialEq + Send + Sync + Unpin + UnwindSafe + RefUnwindSafe,
{
}

fn assert_error<T>()
where
    T: Clone
        + Copy
        + Debug
        + Display
        + Eq
        + PartialEq
        + std::error::Error
        + Send
        + Sync
        + Unpin
        + UnwindSafe
        + RefUnwindSafe
        + 'static,
{
}

trait AmbiguousIfDebug<A> {
    fn marker() {}
}
impl<T: ?Sized> AmbiguousIfDebug<()> for T {}
impl<T: ?Sized + Debug> AmbiguousIfDebug<u8> for T {}

trait AmbiguousIfDefault<A> {
    fn marker() {}
}
impl<T: ?Sized> AmbiguousIfDefault<()> for T {}
impl<T: ?Sized + Default> AmbiguousIfDefault<u8> for T {}

trait AmbiguousIfDisplay<A> {
    fn marker() {}
}
impl<T: ?Sized> AmbiguousIfDisplay<()> for T {}
impl<T: ?Sized + Display> AmbiguousIfDisplay<u8> for T {}

trait AmbiguousIfHash<A> {
    fn marker() {}
}
impl<T: ?Sized> AmbiguousIfHash<()> for T {}
impl<T: ?Sized + Hash> AmbiguousIfHash<u8> for T {}

trait AmbiguousIfOrd<A> {
    fn marker() {}
}
impl<T: ?Sized> AmbiguousIfOrd<()> for T {}
impl<T: ?Sized + Ord> AmbiguousIfOrd<u8> for T {}

trait AmbiguousIfPartialEq<A> {
    fn marker() {}
}
impl<T: ?Sized> AmbiguousIfPartialEq<()> for T {}
impl<T: ?Sized + PartialEq> AmbiguousIfPartialEq<u8> for T {}

trait AmbiguousIfPartialOrd<A> {
    fn marker() {}
}
impl<T: ?Sized> AmbiguousIfPartialOrd<()> for T {}
impl<T: ?Sized + PartialOrd> AmbiguousIfPartialOrd<u8> for T {}
