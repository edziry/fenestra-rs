use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::panic::{RefUnwindSafe, UnwindSafe};

use super::*;

macro_rules! assert_not_clone {
    ($type:ty) => {
        let _ = <$type as AmbiguousIfClone<_>>::marker;
    };
}

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

#[test]
fn prepared_value_has_only_the_registered_runtime_traits() {
    assert_runtime::<PreparedSpatialV2>();
    assert_not_clone!(PreparedSpatialV2);
    assert_not_copy!(PreparedSpatialV2);
    assert_not_debug!(PreparedSpatialV2);
    assert_not_default!(PreparedSpatialV2);
    assert_not_display!(PreparedSpatialV2);
    assert_not_eq!(PreparedSpatialV2);
    assert_not_partial_eq!(PreparedSpatialV2);
    assert_not_hash!(PreparedSpatialV2);
    assert_not_ord!(PreparedSpatialV2);
    assert_not_partial_ord!(PreparedSpatialV2);

    assert_runtime::<SpatialResolvedSnapshotV2>();
    assert_not_clone!(SpatialResolvedSnapshotV2);
    assert_not_copy!(SpatialResolvedSnapshotV2);
    assert_not_debug!(SpatialResolvedSnapshotV2);
    assert_not_default!(SpatialResolvedSnapshotV2);
    assert_not_display!(SpatialResolvedSnapshotV2);
    assert_not_eq!(SpatialResolvedSnapshotV2);
    assert_not_partial_eq!(SpatialResolvedSnapshotV2);
    assert_not_hash!(SpatialResolvedSnapshotV2);
    assert_not_ord!(SpatialResolvedSnapshotV2);
    assert_not_partial_ord!(SpatialResolvedSnapshotV2);
}

fn assert_runtime<T>()
where
    T: Send + Sync + Unpin + UnwindSafe + RefUnwindSafe + 'static,
{
}

trait AmbiguousIfClone<A> {
    fn marker() {}
}
impl<T: ?Sized> AmbiguousIfClone<()> for T {}
impl<T: Clone> AmbiguousIfClone<u8> for T {}

trait AmbiguousIfCopy<A> {
    fn marker() {}
}
impl<T: ?Sized> AmbiguousIfCopy<()> for T {}
impl<T: Copy> AmbiguousIfCopy<u8> for T {}

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

trait AmbiguousIfEq<A> {
    fn marker() {}
}
impl<T: ?Sized> AmbiguousIfEq<()> for T {}
impl<T: ?Sized + Eq> AmbiguousIfEq<u8> for T {}

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
