use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::panic::{RefUnwindSafe, UnwindSafe};

use crate::SpatialPaintFrameV2;

macro_rules! assert_not_trait {
    ($type:ty, $ambiguous:ident) => {
        let _ = <$type as $ambiguous<_>>::marker;
    };
}

#[test]
fn paint_frame_has_only_the_registered_borrowed_traits() {
    type StaticFrame = SpatialPaintFrameV2<'static>;

    assert_borrowed::<StaticFrame>();
    assert_not_trait!(StaticFrame, AmbiguousIfDebug);
    assert_not_trait!(StaticFrame, AmbiguousIfDefault);
    assert_not_trait!(StaticFrame, AmbiguousIfDisplay);
    assert_not_trait!(StaticFrame, AmbiguousIfEq);
    assert_not_trait!(StaticFrame, AmbiguousIfHash);
    assert_not_trait!(StaticFrame, AmbiguousIfOrd);
    assert_not_trait!(StaticFrame, AmbiguousIfPartialEq);
    assert_not_trait!(StaticFrame, AmbiguousIfPartialOrd);
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
