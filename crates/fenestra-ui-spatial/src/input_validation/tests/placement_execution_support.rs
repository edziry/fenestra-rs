use std::collections::VecDeque;
use std::error::Error;
use std::sync::Mutex;

use fenestra_ui_layout::prototype::{
    LayoutAxisV1, LayoutDimensionV1, LayoutEngineErrorV1, LayoutEngineV1, LayoutNodeKeyV1,
    LayoutOutputV1, LayoutPaddingV1, LayoutRecordV1, LayoutRectV1, ValidatedLayoutInputV1,
};

use super::dependency_support;
use super::fixture::RawInputFixture;
use crate::error::SpatialErrorLocationV2;
use crate::limits::{SpatialLimitKindV2, SpatialLimitsV2};
use crate::model::{
    Affine2V2, SpatialAnchorComponentV2, SpatialAnchorTargetV2, SpatialAnchorV2,
    SpatialLocalTransformV2, SpatialNodeKeyV2, SpatialOffsetV2, SpatialPointV2, SpatialScalarV2,
    SpatialViewportV2,
};
use crate::numeric_error::SpatialArithmeticOperationV2;
use crate::resolve_error::{
    SpatialLayoutErrorKindV2, SpatialResolveErrorKindV2, SpatialResolveErrorV2,
};
use crate::topology::{
    SpatialContainerV2, SpatialFreePlacementV2, SpatialLayoutPlacementV2, SpatialNodeV2,
    SpatialPlacementV2,
};

pub(super) const VIEWPORT: SpatialViewportV2 = SpatialViewportV2::new(20, 20);

pub(super) type PlacementFact = (u32, i64, i64, i32, i32, i64, i64, i64, i64);
pub(super) type EngineCallFact = (i32, i32, Vec<(u32, Option<u32>, i32, i32)>);

pub(super) struct ScriptedLayoutEngine {
    responses: Mutex<VecDeque<Result<LayoutOutputV1, LayoutEngineErrorV1>>>,
    calls: Mutex<Vec<EngineCallFact>>,
}

impl ScriptedLayoutEngine {
    pub(super) fn new(responses: Vec<Result<LayoutOutputV1, LayoutEngineErrorV1>>) -> Self {
        Self {
            responses: Mutex::new(responses.into()),
            calls: Mutex::new(Vec::new()),
        }
    }

    pub(super) fn calls(&self) -> Vec<EngineCallFact> {
        self.calls.lock().expect("call log is available").clone()
    }

    pub(super) fn call_count(&self) -> usize {
        self.calls.lock().expect("call log is available").len()
    }
}

impl LayoutEngineV1 for ScriptedLayoutEngine {
    fn compute(
        &self,
        input: ValidatedLayoutInputV1<'_>,
    ) -> Result<LayoutOutputV1, LayoutEngineErrorV1> {
        let viewport = input.viewport();
        let nodes = input
            .nodes()
            .iter()
            .copied()
            .map(|node| {
                let style = node.style();
                (
                    node.key().get(),
                    node.parent().map(LayoutNodeKeyV1::get),
                    style.width().preferred(),
                    style.height().preferred(),
                )
            })
            .collect();
        self.calls.lock().expect("call log is available").push((
            viewport.width(),
            viewport.height(),
            nodes,
        ));
        self.responses
            .lock()
            .expect("response script is available")
            .pop_front()
            .expect("the test supplied one response per expected call")
    }
}

pub(super) fn fixture(nodes: Vec<SpatialNodeV2>) -> RawInputFixture {
    dependency_support::fixture(nodes)
}

pub(super) fn root() -> SpatialNodeV2 {
    SpatialNodeV2::new(
        SpatialNodeKeyV2::new(0),
        None,
        SpatialPlacementV2::Root,
        container(),
    )
}

pub(super) fn layout(key: u32, parent: u32, width: i32, height: i32) -> SpatialNodeV2 {
    SpatialNodeV2::new(
        SpatialNodeKeyV2::new(key),
        Some(SpatialNodeKeyV2::new(parent)),
        SpatialPlacementV2::Layout(SpatialLayoutPlacementV2::new(
            fixed(width),
            fixed(height),
            identity(),
        )),
        container(),
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn free(
    key: u32,
    parent: u32,
    width: i32,
    height: i32,
    self_x: SpatialAnchorComponentV2,
    self_y: SpatialAnchorComponentV2,
    target: SpatialAnchorTargetV2,
    target_x: SpatialAnchorComponentV2,
    target_y: SpatialAnchorComponentV2,
    offset_x: i64,
    offset_y: i64,
) -> SpatialNodeV2 {
    SpatialNodeV2::new(
        SpatialNodeKeyV2::new(key),
        Some(SpatialNodeKeyV2::new(parent)),
        SpatialPlacementV2::Free(SpatialFreePlacementV2::new(
            width,
            height,
            SpatialAnchorV2::new(self_x, self_y),
            target,
            SpatialAnchorV2::new(target_x, target_y),
            SpatialOffsetV2::new(
                SpatialScalarV2::new(offset_x),
                SpatialScalarV2::new(offset_y),
            ),
            identity(),
        )),
        container(),
    )
}

pub(super) fn start_free(key: u32, parent: u32, target: SpatialAnchorTargetV2) -> SpatialNodeV2 {
    use SpatialAnchorComponentV2::Start;
    free(
        key, parent, 10, 10, Start, Start, target, Start, Start, 0, 0,
    )
}

pub(super) const fn node_target(key: u32) -> SpatialAnchorTargetV2 {
    SpatialAnchorTargetV2::Node(SpatialNodeKeyV2::new(key))
}

pub(super) fn output(records: &[(u32, i32, i32, i32, i32)]) -> LayoutOutputV1 {
    LayoutOutputV1::new(
        records
            .iter()
            .map(|&(key, x, y, width, height)| {
                LayoutRecordV1::new(
                    LayoutNodeKeyV1::new(key),
                    LayoutRectV1::new(x, y, width, height),
                )
            })
            .collect(),
    )
}

pub(super) fn limits() -> SpatialLimitsV2 {
    SpatialLimitsV2::new([usize::MAX; SpatialLimitKindV2::ALL.len()])
}

pub(super) fn logical(value: i32) -> i64 {
    i64::from(value) * SpatialScalarV2::SCALE
}

#[allow(clippy::too_many_arguments)]
pub(super) fn placement(
    index: u32,
    x: i64,
    y: i64,
    width: i32,
    height: i32,
    far_x: i64,
    far_y: i64,
    local_x: i64,
    local_y: i64,
) -> PlacementFact {
    (index, x, y, width, height, far_x, far_y, local_x, local_y)
}

pub(super) fn expect_valid<T>(result: Result<T, SpatialResolveErrorV2>) -> T {
    match result {
        Ok(proof) => proof,
        Err(error) => panic!("expected placement execution success, got {error:?}"),
    }
}

pub(super) fn expect_layout<T>(
    result: Result<T, SpatialResolveErrorV2>,
    kind: SpatialLayoutErrorKindV2,
    location: SpatialErrorLocationV2,
) {
    expect_error(
        result,
        SpatialResolveErrorKindV2::Layout(kind),
        location,
        "layout",
    );
}

pub(super) fn expect_arithmetic<T>(
    result: Result<T, SpatialResolveErrorV2>,
    operation: SpatialArithmeticOperationV2,
    node: u32,
) {
    expect_error(
        result,
        SpatialResolveErrorKindV2::Arithmetic(operation),
        SpatialErrorLocationV2::Node { index: node },
        "arithmetic",
    );
}

fn expect_error<T>(
    result: Result<T, SpatialResolveErrorV2>,
    kind: SpatialResolveErrorKindV2,
    location: SpatialErrorLocationV2,
    label: &str,
) {
    let error = match result {
        Ok(_) => panic!("expected placement execution failure"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), kind);
    assert_eq!(error.location(), location);
    assert_eq!(error.observed(), None);
    assert_eq!(error.maximum(), None);
    assert_eq!(error.to_string(), format!("spatial-resolve-error({label})"));
    assert_eq!(
        format!("{error:?}"),
        format!("SpatialResolveErrorV2(spatial-resolve-error({label}))")
    );
    assert!(Error::source(&error).is_none());
}

fn fixed(value: i32) -> LayoutDimensionV1 {
    LayoutDimensionV1::new(value, value, value)
}

fn identity() -> SpatialLocalTransformV2 {
    let zero = SpatialScalarV2::new(0);
    let one = SpatialScalarV2::new(SpatialScalarV2::SCALE);
    SpatialLocalTransformV2::new(
        Affine2V2::new(one, zero, zero, one, zero, zero),
        SpatialPointV2::new(zero, zero),
    )
}

fn container() -> SpatialContainerV2 {
    SpatialContainerV2::new(LayoutAxisV1::Column, LayoutPaddingV1::new(0, 0, 0, 0), 0)
}
