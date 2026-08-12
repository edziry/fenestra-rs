use std::sync::Arc;

use fenestra_ui_layout::prototype::{LayoutAxisV1, LayoutDimensionV1, LayoutPaddingV1};
use fenestra_ui_runtime::prototype::{
    FrameWork, QueueCapacity, RuntimeSpatialBuildViewV2, RuntimeSpatialInputV2,
    RuntimeSpatialProgramV2, SchedulerAction, SchedulerCapacity, SchedulerInput,
    SchedulerInputResult, SchedulerTick, UiRuntime, UiScheduler,
};
use fenestra_ui_spatial::prototype::{
    Affine2V2, REGISTERED_SPATIAL_LIMITS_V2, SpatialBrushContentV2, SpatialBrushKeyV2,
    SpatialBrushV2, SpatialContainerV2, SpatialCoverageV2, SpatialFillRuleV2,
    SpatialLayoutPlacementV2, SpatialLocalTransformV2, SpatialNodeKeyV2, SpatialNodeV2,
    SpatialOwnedInputV2, SpatialPaintContentV2, SpatialPaintV2, SpatialPlacementV2, SpatialPointV2,
    SpatialRgba8V2, SpatialScalarV2, SpatialShapeGeometryV2, SpatialShapeKeyV2, SpatialShapeV2,
    SpatialViewportV2,
};
use fenestra_ui_testkit::prototype::HeadlessFixtureV1;

pub(super) const INITIAL_VIEWPORT: SpatialViewportV2 = SpatialViewportV2::new(1, 1);
pub(super) const LOGICAL_VIEWPORT: SpatialViewportV2 = SpatialViewportV2::new(4, 1);
pub(super) const REFERENCE_RGBA: [u8; 16] =
    [0, 0, 0, 0, 64, 32, 16, 128, 1, 2, 3, 255, 255, 128, 1, 255];
pub(super) const LOGICAL_PACKED: [u32; 4] = [0x0000_0000, 0x0040_2010, 0x0001_0203, 0x00ff_8001];

pub(super) fn spatial_scheduler() -> UiScheduler {
    let fixture = HeadlessFixtureV1::build().expect("registered logical fixture should validate");
    let runtime = UiRuntime::new_spatial(
        fixture.style().clone(),
        Box::new(PixelProgram),
        INITIAL_VIEWPORT,
        REGISTERED_SPATIAL_LIMITS_V2,
        fixture.runtime_capacity().with_retained_generations(4),
    )
    .expect("spatial presentation fixture should initialize");
    UiScheduler::new(runtime, scheduler_capacity()).expect("scheduler limits should validate")
}

pub(super) fn offer_at(
    scheduler: &mut UiScheduler,
    viewport: SpatialViewportV2,
    tick: u64,
) -> FrameWork {
    let mut transaction = scheduler.begin_transaction();
    transaction
        .resize_spatial(viewport)
        .expect("different viewport should stage");
    scheduler
        .commit(transaction, SchedulerTick::new(tick))
        .expect("different viewport should publish");
    assert_eq!(
        scheduler
            .next_action(SchedulerTick::new(tick))
            .expect("request tick should be accepted"),
        Some(SchedulerAction::RequestFrame)
    );
    assert_eq!(
        scheduler
            .process_input(SchedulerInput::FrameReady, SchedulerTick::new(tick + 1),)
            .expect("frame ready should be accepted"),
        SchedulerInputResult::FrameReady
    );
    take_offer(scheduler, tick + 1)
}

pub(super) fn take_offer(scheduler: &mut UiScheduler, tick: u64) -> FrameWork {
    let Some(SchedulerAction::OfferFrame(work)) = scheduler
        .next_action(SchedulerTick::new(tick))
        .expect("offer tick should be accepted")
    else {
        panic!("one spatial frame should be offered");
    };
    work
}

pub(super) fn replace_rejected_offer(
    scheduler: &mut UiScheduler,
    viewport: SpatialViewportV2,
    tick: u64,
) -> FrameWork {
    let mut transaction = scheduler.begin_transaction();
    transaction
        .resize_spatial(viewport)
        .expect("replacement viewport should stage");
    scheduler
        .commit(transaction, SchedulerTick::new(tick))
        .expect("replacement viewport should publish");
    take_offer(scheduler, tick)
}

pub(super) fn reject(scheduler: &mut UiScheduler, work: &FrameWork, tick: u64) {
    assert_eq!(
        scheduler
            .process_input(
                SchedulerInput::RejectFrame(work.id()),
                SchedulerTick::new(tick),
            )
            .expect("offer rejection should be accepted"),
        SchedulerInputResult::FrameRejected(work.id())
    );
}

fn scheduler_capacity() -> SchedulerCapacity {
    SchedulerCapacity::new(
        QueueCapacity::new(1, 80, 8),
        QueueCapacity::new(4, 128, 8),
        QueueCapacity::new(1, 40, 8),
        QueueCapacity::new(2, 80, 8),
    )
}

struct PixelProgram;

impl RuntimeSpatialProgramV2 for PixelProgram {
    fn build(
        &self,
        runtime: RuntimeSpatialBuildViewV2<'_>,
        viewport: SpatialViewportV2,
    ) -> RuntimeSpatialInputV2 {
        RuntimeSpatialInputV2::new(source(viewport), vec![runtime.root()].into_boxed_slice())
    }
}

fn source(viewport: SpatialViewportV2) -> Arc<SpatialOwnedInputV2> {
    let container =
        SpatialContainerV2::new(LayoutAxisV1::Column, LayoutPaddingV1::new(0, 0, 0, 0), 0);
    let sentinel = SpatialNodeV2::new(
        SpatialNodeKeyV2::new(0),
        None,
        SpatialPlacementV2::Root,
        container,
    );
    let width = i32::try_from(viewport.width()).expect("fixture width should fit");
    let height = i32::try_from(viewport.height()).expect("fixture height should fit");
    let owner = SpatialNodeV2::new(
        SpatialNodeKeyV2::new(1),
        Some(SpatialNodeKeyV2::new(0)),
        SpatialPlacementV2::Layout(SpatialLayoutPlacementV2::new(
            LayoutDimensionV1::new(width, width, width),
            LayoutDimensionV1::new(height, height, height),
            SpatialLocalTransformV2::new(Affine2V2::identity(), point(0, 0)),
        )),
        container,
    );
    let colors = [
        SpatialRgba8V2::new(255, 128, 64, 0),
        SpatialRgba8V2::new(128, 64, 32, 128),
        SpatialRgba8V2::new(1, 2, 3, 255),
        SpatialRgba8V2::new(255, 128, 1, 255),
    ];
    let shapes = (0_u32..4)
        .map(|index| {
            SpatialShapeV2::new(
                SpatialShapeKeyV2::new(index),
                SpatialNodeKeyV2::new(1),
                SpatialShapeGeometryV2::Rect {
                    origin: point(i64::from(index), 0),
                    width: scalar(1),
                    height: scalar(1),
                },
            )
        })
        .collect::<Vec<_>>();
    let brushes = colors
        .into_iter()
        .enumerate()
        .map(|(index, color)| {
            SpatialBrushV2::new(
                SpatialBrushKeyV2::new(u32::try_from(index).expect("fixture ordinal should fit")),
                SpatialBrushContentV2::Solid { color },
            )
        })
        .collect::<Vec<_>>();
    let paints = (0_u32..4)
        .map(|index| {
            SpatialPaintV2::new(
                SpatialNodeKeyV2::new(1),
                index,
                SpatialPaintContentV2::CoveragePaint {
                    coverage: SpatialCoverageV2::Fill {
                        shape: SpatialShapeKeyV2::new(index),
                        rule: SpatialFillRuleV2::NonZero,
                    },
                    brush: SpatialBrushKeyV2::new(index),
                    opacity: u8::MAX,
                    clip: None,
                },
            )
        })
        .collect::<Vec<_>>();
    Arc::new(SpatialOwnedInputV2::new(
        viewport,
        vec![sentinel, owner].into_boxed_slice(),
        Box::new([]),
        Box::new([]),
        Box::new([]),
        shapes.into_boxed_slice(),
        Box::new([]),
        Box::new([]),
        brushes.into_boxed_slice(),
        Box::new([]),
        paints.into_boxed_slice(),
        Box::new([]),
        Box::new([]),
    ))
}

fn point(x: i64, y: i64) -> SpatialPointV2 {
    SpatialPointV2::new(scalar(x), scalar(y))
}

fn scalar(value: i64) -> SpatialScalarV2 {
    SpatialScalarV2::new(value * SpatialScalarV2::SCALE)
}
