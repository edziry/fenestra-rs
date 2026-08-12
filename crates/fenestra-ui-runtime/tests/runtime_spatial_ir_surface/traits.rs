use std::error::Error;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::panic::{RefUnwindSafe, UnwindSafe};

use fenestra_ui_runtime::prototype::RuntimeSpatialErrorV2;

use crate::{RuntimeSpatialIrErrorKindV2, RuntimeSpatialIrErrorV2};

macro_rules! assert_not {
    ($type:ty, $trait_name:ident) => {
        let _ = <$type as $trait_name<_>>::marker;
    };
}

#[test]
fn runtime_ir_error_kind_has_only_copy_value_traits() {
    assert_kind::<RuntimeSpatialIrErrorKindV2>();
    assert_not!(RuntimeSpatialIrErrorKindV2, AmbiguousIfDefault);
    assert_not!(RuntimeSpatialIrErrorKindV2, AmbiguousIfDisplay);
    assert_not!(RuntimeSpatialIrErrorKindV2, AmbiguousIfError);
    assert_not!(RuntimeSpatialIrErrorKindV2, AmbiguousIfHash);
    assert_not!(RuntimeSpatialIrErrorKindV2, AmbiguousIfOrd);
    assert_not!(RuntimeSpatialIrErrorKindV2, AmbiguousIfPartialOrd);
}

#[test]
fn runtime_ir_error_has_only_registered_error_traits() {
    assert_error::<RuntimeSpatialIrErrorV2>();
    assert_not!(RuntimeSpatialIrErrorV2, AmbiguousIfDefault);
    assert_not!(RuntimeSpatialIrErrorV2, AmbiguousIfHash);
    assert_not!(RuntimeSpatialIrErrorV2, AmbiguousIfOrd);
    assert_not!(RuntimeSpatialIrErrorV2, AmbiguousIfPartialOrd);
}

#[test]
fn outer_runtime_spatial_error_retains_its_exact_traits() {
    assert_error::<RuntimeSpatialErrorV2>();
    assert_not!(RuntimeSpatialErrorV2, AmbiguousIfDefault);
    assert_not!(RuntimeSpatialErrorV2, AmbiguousIfHash);
    assert_not!(RuntimeSpatialErrorV2, AmbiguousIfOrd);
    assert_not!(RuntimeSpatialErrorV2, AmbiguousIfPartialOrd);
}

fn assert_kind<T>()
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
        impl<T: ?Sized + $bound> $trait_name<u8> for T {}
    };
}

negative_trait!(AmbiguousIfDefault, Default);
negative_trait!(AmbiguousIfDisplay, Display);
negative_trait!(AmbiguousIfError, Error);
negative_trait!(AmbiguousIfHash, Hash);
negative_trait!(AmbiguousIfOrd, Ord);
negative_trait!(AmbiguousIfPartialOrd, PartialOrd);
