use std::error::Error;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::panic::{RefUnwindSafe, UnwindSafe};

use super::*;
use crate::error::SpatialErrorLocationV2;

macro_rules! assert_not {
    ($type:ty, $trait_name:ident) => {
        let _ = <$type as $trait_name<_>>::marker;
    };
}

#[test]
fn reference_raster_api_has_the_exact_signatures() {
    let _: fn(usize) -> ReferenceRasterLimitsV2 = ReferenceRasterLimitsV2::new;
    let _: fn(ReferenceRasterLimitsV2, ReferenceRasterLimitKindV2) -> usize =
        ReferenceRasterLimitsV2::limit;
    let _: fn(ReferenceRasterErrorV2) -> ReferenceRasterErrorKindV2 = ReferenceRasterErrorV2::kind;
    let _: fn(ReferenceRasterErrorV2) -> SpatialErrorLocationV2 = ReferenceRasterErrorV2::location;
    let _: fn(ReferenceRasterErrorV2) -> Option<u128> = ReferenceRasterErrorV2::observed;
    let _: fn(ReferenceRasterErrorV2) -> Option<u128> = ReferenceRasterErrorV2::maximum;
    let _: for<'a> fn(&'a ReferenceRasterV2) -> u32 = ReferenceRasterV2::width;
    let _: for<'a> fn(&'a ReferenceRasterV2) -> u32 = ReferenceRasterV2::height;
    let _: for<'a> fn(&'a ReferenceRasterV2) -> u64 = ReferenceRasterV2::stride;
    let _: for<'a> fn(&'a ReferenceRasterV2) -> &'a [u8] = ReferenceRasterV2::bytes;
    let _: for<'a> fn(
        &'a SpatialResolvedSnapshotV2,
        ReferenceRasterLimitsV2,
    ) -> Result<ReferenceRasterV2, ReferenceRasterErrorV2> =
        SpatialResolvedSnapshotV2::rasterize_reference;
}

#[test]
fn closed_raster_values_have_only_the_registered_traits() {
    assert_value::<ReferenceRasterLimitKindV2>();
    assert_value::<ReferenceRasterLimitsV2>();
    assert_value::<ReferenceRasterErrorKindV2>();
    for kind in ReferenceRasterLimitKindV2::ALL {
        assert_limit_kind(kind);
    }
    assert_eq!(ReferenceRasterLimitKindV2::ALL.len(), 1);
    assert_eq!(ReferenceRasterErrorKindV2::ALL.len(), 1);
    assert_eq!(
        ReferenceRasterErrorKindV2::ALL[0],
        ReferenceRasterErrorKindV2::LimitExceeded(ReferenceRasterLimitKindV2::Pixels)
    );
    for kind in ReferenceRasterErrorKindV2::ALL {
        assert_error_kind(kind);
    }

    assert_not!(ReferenceRasterLimitKindV2, AmbiguousIfDefault);
    assert_not!(ReferenceRasterLimitKindV2, AmbiguousIfDisplay);
    assert_not!(ReferenceRasterLimitKindV2, AmbiguousIfHash);
    assert_not!(ReferenceRasterLimitKindV2, AmbiguousIfOrd);
    assert_not!(ReferenceRasterLimitKindV2, AmbiguousIfPartialOrd);
    assert_not!(ReferenceRasterLimitsV2, AmbiguousIfDefault);
    assert_not!(ReferenceRasterLimitsV2, AmbiguousIfDisplay);
    assert_not!(ReferenceRasterLimitsV2, AmbiguousIfHash);
    assert_not!(ReferenceRasterLimitsV2, AmbiguousIfOrd);
    assert_not!(ReferenceRasterLimitsV2, AmbiguousIfPartialOrd);
    assert_not!(ReferenceRasterErrorKindV2, AmbiguousIfDefault);
    assert_not!(ReferenceRasterErrorKindV2, AmbiguousIfDisplay);
    assert_not!(ReferenceRasterErrorKindV2, AmbiguousIfHash);
    assert_not!(ReferenceRasterErrorKindV2, AmbiguousIfOrd);
    assert_not!(ReferenceRasterErrorKindV2, AmbiguousIfPartialOrd);
}

#[test]
fn stored_error_and_owned_raster_have_the_exact_runtime_traits() {
    assert_error::<ReferenceRasterErrorV2>();
    assert_not!(ReferenceRasterErrorV2, AmbiguousIfDefault);
    assert_not!(ReferenceRasterErrorV2, AmbiguousIfHash);
    assert_not!(ReferenceRasterErrorV2, AmbiguousIfOrd);
    assert_not!(ReferenceRasterErrorV2, AmbiguousIfPartialOrd);

    assert_runtime::<ReferenceRasterV2>();
    assert_not!(ReferenceRasterV2, AmbiguousIfClone);
    assert_not!(ReferenceRasterV2, AmbiguousIfCopy);
    assert_not!(ReferenceRasterV2, AmbiguousIfDebug);
    assert_not!(ReferenceRasterV2, AmbiguousIfDefault);
    assert_not!(ReferenceRasterV2, AmbiguousIfDisplay);
    assert_not!(ReferenceRasterV2, AmbiguousIfEq);
    assert_not!(ReferenceRasterV2, AmbiguousIfPartialEq);
    assert_not!(ReferenceRasterV2, AmbiguousIfHash);
    assert_not!(ReferenceRasterV2, AmbiguousIfOrd);
    assert_not!(ReferenceRasterV2, AmbiguousIfPartialOrd);
}

fn assert_value<T>()
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

fn assert_runtime<T>()
where
    T: Send + Sync + Unpin + UnwindSafe + RefUnwindSafe + 'static,
{
}

fn assert_limit_kind(kind: ReferenceRasterLimitKindV2) {
    match kind {
        ReferenceRasterLimitKindV2::Pixels => {}
    }
}

fn assert_error_kind(kind: ReferenceRasterErrorKindV2) {
    match kind {
        ReferenceRasterErrorKindV2::LimitExceeded(limit) => assert_limit_kind(limit),
    }
}

macro_rules! negative_trait {
    ($trait_name:ident, $bound:path) => {
        trait $trait_name<A> {
            fn marker() {}
        }
        impl<T> $trait_name<()> for T {}
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
