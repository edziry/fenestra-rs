use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::panic::{RefUnwindSafe, UnwindSafe};

use crate::*;

macro_rules! assert_not_debug {
    ($type:ty) => {
        let _ = <$type as AmbiguousIfDebug<_>>::marker;
    };
}

macro_rules! assert_not_partial_eq {
    ($type:ty) => {
        let _ = <$type as AmbiguousIfPartialEq<_>>::marker;
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

#[test]
fn owned_geometry_values_preserve_required_traits() {
    assert_key::<SpatialPathKeyV2>();
    assert_key::<SpatialShapeKeyV2>();
    assert_key::<SpatialClipKeyV2>();
    assert_value::<SpatialPathVerbKindV2>();
    assert_value::<SpatialPathVerbV2>();
    assert_value::<SpatialPathV2>();
    assert_value::<SpatialShapeKindV2>();
    assert_value::<SpatialShapeGeometryV2>();
    assert_value::<SpatialShapeV2>();
    assert_value::<SpatialFillRuleV2>();
    assert_value::<SpatialCoverageKindV2>();
    assert_value::<SpatialCoverageV2>();
    assert_value::<SpatialClipV2>();
}

#[test]
fn borrowed_geometry_input_has_only_the_registered_value_traits() {
    type StaticInput = SpatialGeometryInputV2<'static>;

    assert_borrowed::<StaticInput>();
    assert_not_debug!(StaticInput);
    assert_not_display!(StaticInput);
    assert_not_partial_eq!(StaticInput);
    assert_not_hash!(StaticInput);
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

fn assert_borrowed<T>()
where
    T: Clone + Copy + Send + Sync + Unpin + UnwindSafe + RefUnwindSafe,
{
}

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

trait AmbiguousIfPartialEq<A> {
    fn marker() {}
}

impl<T: ?Sized> AmbiguousIfPartialEq<()> for T {}
impl<T: ?Sized + PartialEq> AmbiguousIfPartialEq<u8> for T {}

trait AmbiguousIfHash<A> {
    fn marker() {}
}

impl<T: ?Sized> AmbiguousIfHash<()> for T {}
impl<T: ?Sized + Hash> AmbiguousIfHash<u8> for T {}
