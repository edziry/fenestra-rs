use std::error::Error;
use std::sync::Arc;

use fenestra_ui_layout::prototype::ReferenceStackEngineV1;
use fenestra_ui_spatial::prototype::{
    SpatialLimitsV2, SpatialNodeKeyV2, SpatialOwnedInputV2, SpatialResolveErrorV2,
    SpatialViewportV2, resolve_spatial_v2,
};

use crate::RuntimeSpatialErrorV2;

#[test]
fn runtime_spatial_errors_have_exact_redacted_text() {
    for (error, label) in [
        (
            RuntimeSpatialErrorV2::ViewportMismatch,
            "runtime-spatial-error(viewport-mismatch)",
        ),
        (
            RuntimeSpatialErrorV2::MappingLengthMismatch,
            "runtime-spatial-error(mapping-length-mismatch)",
        ),
        (
            RuntimeSpatialErrorV2::MissingLogicalNode {
                key: SpatialNodeKeyV2::new(17),
            },
            "runtime-spatial-error(missing-logical-node)",
        ),
        (
            RuntimeSpatialErrorV2::DuplicateLogicalNode {
                key: SpatialNodeKeyV2::new(29),
            },
            "runtime-spatial-error(duplicate-logical-node)",
        ),
        (
            RuntimeSpatialErrorV2::Resolve(resolve_error()),
            "runtime-spatial-error(resolve)",
        ),
    ] {
        assert_eq!(error.to_string(), label);
        assert_eq!(
            format!("{error:?}"),
            format!("RuntimeSpatialErrorV2({label})")
        );
        assert!(error.source().is_none());
    }
}

#[test]
fn runtime_spatial_errors_retain_only_typed_nonidentity_payloads() {
    let resolve = resolve_error();
    let values = [
        RuntimeSpatialErrorV2::ViewportMismatch,
        RuntimeSpatialErrorV2::MappingLengthMismatch,
        RuntimeSpatialErrorV2::MissingLogicalNode {
            key: SpatialNodeKeyV2::new(1),
        },
        RuntimeSpatialErrorV2::DuplicateLogicalNode {
            key: SpatialNodeKeyV2::new(2),
        },
        RuntimeSpatialErrorV2::Resolve(resolve),
    ];
    for value in values {
        match value {
            RuntimeSpatialErrorV2::ViewportMismatch
            | RuntimeSpatialErrorV2::MappingLengthMismatch => {}
            RuntimeSpatialErrorV2::MissingLogicalNode { key }
            | RuntimeSpatialErrorV2::DuplicateLogicalNode { key } => {
                assert!(key.get() > 0);
            }
            RuntimeSpatialErrorV2::Ir(_) => panic!("manual inputs do not produce IR errors"),
            RuntimeSpatialErrorV2::Resolve(actual) => assert_eq!(actual, resolve),
        }
    }
}

fn resolve_error() -> SpatialResolveErrorV2 {
    let input = SpatialOwnedInputV2::new(
        SpatialViewportV2::new(1, 1),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        Box::new([]),
    );
    match resolve_spatial_v2(
        &ReferenceStackEngineV1::new(),
        Arc::new(input),
        SpatialLimitsV2::new([usize::MAX; 30]),
    ) {
        Err(error) => error,
        Ok(_) => panic!("empty topology must fail"),
    }
}
