use super::*;

use crate::content_error::SpatialContentErrorKindV2;
use crate::error::{SpatialDependencyErrorKindV2, SpatialInputErrorKindV2};
use crate::numeric_error::{SpatialArithmeticOperationV2, SpatialTransformErrorKindV2};
use crate::resolve_error::{SpatialLayoutErrorKindV2, SpatialOutputErrorKindV2};

#[test]
fn non_limit_error_categories_have_exact_redacted_formatting() {
    let location = SpatialErrorLocationV2::Node { index: u32::MAX };
    let cases = [
        (
            SpatialResolveErrorKindV2::Input(SpatialInputErrorKindV2::EmptyInput),
            "input",
        ),
        (
            SpatialResolveErrorKindV2::Content(SpatialContentErrorKindV2::NonFlatAtMaximumDepth),
            "content",
        ),
        (
            SpatialResolveErrorKindV2::Dependency(SpatialDependencyErrorKindV2::Cycle),
            "dependency",
        ),
        (
            SpatialResolveErrorKindV2::Transform(SpatialTransformErrorKindV2::SingularTransform),
            "transform",
        ),
        (
            SpatialResolveErrorKindV2::Layout(SpatialLayoutErrorKindV2::BridgeInvariant),
            "layout",
        ),
        (
            SpatialResolveErrorKindV2::Arithmetic(SpatialArithmeticOperationV2::TargetOffsetX),
            "arithmetic",
        ),
        (
            SpatialResolveErrorKindV2::Output(SpatialOutputErrorKindV2::InvalidAabb),
            "output",
        ),
    ];

    for (kind, category) in cases {
        let error = make_resolve_error(kind, location);
        let display = format!("spatial-resolve-error({category})");

        assert_eq!(error.kind(), kind);
        assert_eq!(error.location(), location);
        assert_eq!(error.observed(), None);
        assert_eq!(error.maximum(), None);
        assert_eq!(error.to_string(), display);
        assert_eq!(
            format!("{error:?}"),
            format!("SpatialResolveErrorV2({display})")
        );
        assert!(Error::source(&error).is_none());
    }
}
