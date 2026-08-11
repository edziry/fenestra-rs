use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::panic::{RefUnwindSafe, UnwindSafe};

use crate::*;

macro_rules! assert_not_copy {
    ($type:ty) => {
        let _ = <$type as AmbiguousIfCopy<_>>::marker;
    };
}

macro_rules! assert_not_debug {
    ($type:ty) => {
        let _ = <$type as AmbiguousIfDebug<_>>::marker;
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
fn owned_content_values_preserve_the_registered_traits() {
    assert_key::<SpatialBrushKeyV2>();
    assert_key::<SpatialImageKeyV2>();
    assert_value::<SpatialRgba8V2>();
    assert_value::<SpatialGradientStopV2>();
    assert_value::<SpatialBrushKindV2>();
    assert_value::<SpatialBrushContentV2>();
    assert_value::<SpatialBrushV2>();
    assert_value::<SpatialImageSourceRectV2>();
    assert_value::<SpatialImageDestinationRectV2>();
    assert_value::<SpatialPaintKindV2>();
    assert_value::<SpatialPaintContentV2>();
    assert_value::<SpatialPaintV2>();
    assert_value::<SpatialInputPolicyV2>();
    assert_value::<SpatialHitV2>();
    assert_value::<SpatialSemanticGeometryV2>();
    assert_image::<SpatialImageV2>();

    assert_not_copy!(SpatialImageV2);
    assert_not_hash!(SpatialImageV2);
    assert_not_ord!(SpatialImageV2);
    assert_not_partial_ord!(SpatialImageV2);
}

#[test]
fn borrowed_content_inputs_have_only_the_registered_value_traits() {
    type StaticResources = SpatialResourceInputV2<'static>;
    type StaticItems = SpatialItemInputV2<'static>;

    assert_borrowed::<StaticResources>();
    assert_borrowed::<StaticItems>();
    assert_not_debug!(StaticResources);
    assert_not_display!(StaticResources);
    assert_not_partial_eq!(StaticResources);
    assert_not_hash!(StaticResources);
    assert_not_debug!(StaticItems);
    assert_not_display!(StaticItems);
    assert_not_partial_eq!(StaticItems);
    assert_not_hash!(StaticItems);
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
        + RefUnwindSafe,
{
}

fn assert_value<T>()
where
    T: Clone + Copy + Debug + Eq + PartialEq + Send + Sync + Unpin + UnwindSafe + RefUnwindSafe,
{
}

fn assert_image<T>()
where
    T: Clone + Debug + Eq + PartialEq + Send + Sync + Unpin + UnwindSafe + RefUnwindSafe,
{
}

fn assert_borrowed<T>()
where
    T: Clone + Copy + Send + Sync + Unpin + UnwindSafe + RefUnwindSafe,
{
}

trait AmbiguousIfCopy<A> {
    fn marker() {}
}
impl<T: ?Sized> AmbiguousIfCopy<()> for T {}
impl<T: ?Sized + Copy> AmbiguousIfCopy<u8> for T {}

trait AmbiguousIfDebug<A> {
    fn marker() {}
}
impl<T: ?Sized> AmbiguousIfDebug<()> for T {}
impl<T: ?Sized + Debug> AmbiguousIfDebug<u8> for T {}

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
