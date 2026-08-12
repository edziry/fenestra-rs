use std::error::Error;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::panic::{RefUnwindSafe, UnwindSafe};

use crate::{
    RuntimeSpatialBuildViewV2, RuntimeSpatialErrorV2, RuntimeSpatialInputV2,
    RuntimeSpatialProgramV2, RuntimeSpatialViewV2, SpatialViewportChangeViewV2,
};

macro_rules! assert_not {
    ($type:ty, $trait_name:ident) => {
        let _ = <$type as $trait_name<_>>::marker;
    };
}

macro_rules! assert_no_view_extras {
    ($type:ty) => {
        assert_not!($type, AmbiguousIfDebug);
        assert_not!($type, AmbiguousIfDefault);
        assert_not!($type, AmbiguousIfDisplay);
        assert_not!($type, AmbiguousIfEq);
        assert_not!($type, AmbiguousIfHash);
        assert_not!($type, AmbiguousIfOrd);
        assert_not!($type, AmbiguousIfPartialEq);
        assert_not!($type, AmbiguousIfPartialOrd);
    };
}

#[test]
fn runtime_spatial_views_have_only_copy_runtime_traits() {
    assert_copy_runtime::<RuntimeSpatialBuildViewV2<'static>>();
    assert_copy_runtime::<RuntimeSpatialViewV2<'static>>();
    assert_copy_runtime::<SpatialViewportChangeViewV2<'static>>();

    assert_no_view_extras!(RuntimeSpatialBuildViewV2<'static>);
    assert_no_view_extras!(RuntimeSpatialViewV2<'static>);
    assert_no_view_extras!(SpatialViewportChangeViewV2<'static>);
}

#[test]
fn runtime_spatial_input_has_only_runtime_traits() {
    assert_runtime::<RuntimeSpatialInputV2>();
    assert_not!(RuntimeSpatialInputV2, AmbiguousIfClone);
    assert_not!(RuntimeSpatialInputV2, AmbiguousIfCopy);
    assert_not!(RuntimeSpatialInputV2, AmbiguousIfDebug);
    assert_not!(RuntimeSpatialInputV2, AmbiguousIfDefault);
    assert_not!(RuntimeSpatialInputV2, AmbiguousIfDisplay);
    assert_not!(RuntimeSpatialInputV2, AmbiguousIfEq);
    assert_not!(RuntimeSpatialInputV2, AmbiguousIfHash);
    assert_not!(RuntimeSpatialInputV2, AmbiguousIfOrd);
    assert_not!(RuntimeSpatialInputV2, AmbiguousIfPartialEq);
    assert_not!(RuntimeSpatialInputV2, AmbiguousIfPartialOrd);
}

#[test]
fn runtime_spatial_error_has_only_the_registered_error_traits() {
    assert_error::<RuntimeSpatialErrorV2>();
    assert_not!(RuntimeSpatialErrorV2, AmbiguousIfDefault);
    assert_not!(RuntimeSpatialErrorV2, AmbiguousIfHash);
    assert_not!(RuntimeSpatialErrorV2, AmbiguousIfOrd);
    assert_not!(RuntimeSpatialErrorV2, AmbiguousIfPartialOrd);
}

#[test]
fn runtime_spatial_program_bound_is_the_complete_runtime_bound() {
    fn assert_program<T: RuntimeSpatialProgramV2>() {}

    struct Program;
    impl RuntimeSpatialProgramV2 for Program {
        fn build(
            &self,
            _runtime: RuntimeSpatialBuildViewV2<'_>,
            _viewport: fenestra_ui_spatial::prototype::SpatialViewportV2,
        ) -> RuntimeSpatialInputV2 {
            panic!("trait probe does not invoke the program")
        }
    }

    assert_program::<Program>();
}

fn assert_copy_runtime<T>()
where
    T: Clone + Copy + Send + Sync + Unpin + UnwindSafe + RefUnwindSafe + 'static,
{
}

fn assert_runtime<T>()
where
    T: Send + Sync + Unpin + UnwindSafe + RefUnwindSafe + 'static,
{
}

fn assert_error<T>()
where
    T: Clone
        + Copy
        + Debug
        + Display
        + Error
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

macro_rules! negative_trait {
    ($trait_name:ident, $bound:path) => {
        trait $trait_name<A> {
            fn marker() {}
        }
        impl<T: ?Sized> $trait_name<()> for T {}
        impl<T: $bound> $trait_name<u8> for T {}
    };
}

negative_trait!(AmbiguousIfClone, Clone);
negative_trait!(AmbiguousIfCopy, Copy);
negative_trait!(AmbiguousIfDebug, Debug);
negative_trait!(AmbiguousIfDefault, Default);
negative_trait!(AmbiguousIfDisplay, Display);
negative_trait!(AmbiguousIfEq, Eq);
negative_trait!(AmbiguousIfHash, Hash);
negative_trait!(AmbiguousIfOrd, Ord);
negative_trait!(AmbiguousIfPartialEq, PartialEq);
negative_trait!(AmbiguousIfPartialOrd, PartialOrd);
