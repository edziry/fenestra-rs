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

macro_rules! assert_not_eq {
    ($type:ty) => {
        let _ = <$type as AmbiguousIfEq<_>>::marker;
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

macro_rules! assert_negative_value_traits {
    ($type:ty) => {
        assert_not_default!($type);
        assert_not_display!($type);
        assert_not_hash!($type);
        assert_not_ord!($type);
        assert_not_partial_ord!($type);
    };
}

#[test]
fn raw_output_values_have_only_the_registered_value_traits() {
    assert_value::<SpatialOutputAabbV2>();
    assert_value::<SpatialPaintOutputReferenceV2>();
    assert_value::<SpatialGeometryOutputRecordV2>();
    assert_value::<SpatialClipOutputRecordV2>();
    assert_value::<SpatialPaintOutputRecordV2>();
    assert_value::<SpatialHitOutputRecordV2>();
    assert_value::<SpatialSemanticOutputRecordV2>();

    assert_negative_value_traits!(SpatialOutputAabbV2);
    assert_negative_value_traits!(SpatialPaintOutputReferenceV2);
    assert_negative_value_traits!(SpatialGeometryOutputRecordV2);
    assert_negative_value_traits!(SpatialClipOutputRecordV2);
    assert_negative_value_traits!(SpatialPaintOutputRecordV2);
    assert_negative_value_traits!(SpatialHitOutputRecordV2);
    assert_negative_value_traits!(SpatialSemanticOutputRecordV2);
}

#[test]
fn output_view_has_only_the_registered_borrowed_traits() {
    type StaticOutput = SpatialOutputV2<'static>;

    assert_borrowed::<StaticOutput>();
    assert_not_debug!(StaticOutput);
    assert_not_default!(StaticOutput);
    assert_not_display!(StaticOutput);
    assert_not_eq!(StaticOutput);
    assert_not_partial_eq!(StaticOutput);
    assert_not_hash!(StaticOutput);
    assert_not_ord!(StaticOutput);
    assert_not_partial_ord!(StaticOutput);
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

trait AmbiguousIfDefault<A> {
    fn marker() {}
}
impl<T: ?Sized> AmbiguousIfDefault<()> for T {}
impl<T: Default> AmbiguousIfDefault<u8> for T {}

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

trait AmbiguousIfEq<A> {
    fn marker() {}
}
impl<T: ?Sized> AmbiguousIfEq<()> for T {}
impl<T: ?Sized + Eq> AmbiguousIfEq<u8> for T {}

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
