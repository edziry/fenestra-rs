use std::collections::BTreeSet;

use fenestra_ui_ir::prototype::ValidatedSpatialProgramV2;
use fenestra_ui_layout::prototype::LayoutEngineV1;
use fenestra_ui_runtime::prototype::{RuntimeCapacity, RuntimeInitializationError, UiRuntime};
use fenestra_ui_spatial::prototype::{SpatialLimitsV2, SpatialViewportV2};

use super::source::all_source;
use super::support::{method_attributes, names, public_methods};

#[allow(clippy::type_complexity)]
#[test]
fn runtime_ir_constructor_signatures_are_exact() {
    let _: fn(
        ValidatedSpatialProgramV2,
        SpatialViewportV2,
        SpatialLimitsV2,
        RuntimeCapacity,
    ) -> Result<UiRuntime, RuntimeInitializationError> = UiRuntime::new_spatial_ir;
    let _: fn(
        ValidatedSpatialProgramV2,
        SpatialViewportV2,
        SpatialLimitsV2,
        RuntimeCapacity,
        Box<dyn LayoutEngineV1>,
    ) -> Result<UiRuntime, RuntimeInitializationError> =
        UiRuntime::new_spatial_ir_with_layout_engine;
}

#[test]
fn ui_runtime_has_exact_ten_method_additive_surface() {
    assert_eq!(
        public_methods(&all_source(), "UiRuntime"),
        names(&[
            "begin_transaction",
            "commit",
            "committed",
            "new",
            "new_headless",
            "new_headless_with_layout_engine",
            "new_spatial",
            "new_spatial_ir",
            "new_spatial_ir_with_layout_engine",
            "new_spatial_with_layout_engine",
        ])
    );
}

#[test]
fn runtime_ir_constructors_have_only_the_frozen_attributes() {
    let source = all_source();
    assert_eq!(
        method_attributes(&source, "new_spatial_ir"),
        BTreeSet::new()
    );
    assert_eq!(
        method_attributes(&source, "new_spatial_ir_with_layout_engine"),
        names(&["#[doc(hidden)]"])
    );
    for name in ["new_spatial_ir", "new_spatial_ir_with_layout_engine"] {
        assert!(!source.contains(&format!("pub const fn {name}(")));
        assert!(
            !source[..source
                .find(&format!("pub fn {name}("))
                .expect("constructor")]
                .lines()
                .rev()
                .take_while(|line| line.trim().starts_with("#[") || line.trim().is_empty())
                .any(|line| line.contains("must_use"))
        );
    }
}
