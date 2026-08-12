use fenestra_ui_runtime::prototype::{RuntimeInitializationErrorKind, UiRuntime};

use crate::RuntimeSpatialErrorV2;
use crate::spatial_support::program::{MappingPlan, ProgramSpy, SourcePlan};
use crate::spatial_support::{VIEWPORT, limits, styled_program};
use crate::support::headless::runtime_capacity;

#[test]
fn manual_callback_success_and_resolver_failures_keep_the_existing_lane() {
    let (program, state) = ProgramSpy::new(SourcePlan::Canonical, MappingPlan::Canonical);
    let runtime = UiRuntime::new_spatial(
        styled_program(),
        Box::new(program),
        VIEWPORT,
        limits(),
        runtime_capacity(),
    )
    .expect("manual callback fixture should still initialize");
    assert_eq!(state.calls(), 1);
    assert_eq!(runtime.committed().generation().get(), 0);

    let (malformed, malformed_state) = ProgramSpy::new(
        SourcePlan::MalformedCanonicalOnCall(1),
        MappingPlan::Canonical,
    );
    let error = match UiRuntime::new_spatial(
        styled_program(),
        Box::new(malformed),
        VIEWPORT,
        limits(),
        runtime_capacity(),
    ) {
        Ok(_) => panic!("malformed manual source should fail"),
        Err(error) => error,
    };
    let RuntimeInitializationErrorKind::Spatial(RuntimeSpatialErrorV2::Resolve(resolve)) =
        error.kind()
    else {
        panic!("manual failures must not be reclassified as IR failures");
    };
    assert_eq!(malformed_state.calls(), 1);
    assert_eq!(
        resolve.kind(),
        fenestra_ui_spatial::prototype::SpatialResolveErrorKindV2::Input(
            fenestra_ui_spatial::prototype::SpatialInputErrorKindV2::InvalidRootKey,
        )
    );
}
