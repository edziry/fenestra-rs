use std::sync::Arc;

use fenestra_ui_runtime::prototype::{RuntimePaintFrameV2, SubmissionId};
use fenestra_ui_spatial::prototype::ReferenceRasterLimitsV2;
use softbuffer::{Context, Surface};
use winit::event_loop::OwnedDisplayHandle;
use winit::window::Window;

use super::super::super::spatial_v2::{
    SpatialPresentErrorKindV2, SpatialPresentationOutcomeV2, SpatialPresenterPortV2,
    SpatialReferencePresenterV2, SpatialSurfaceTupleV2, present_spatial_offer_v2,
    spatial_pre_present_notify_source_v2,
};
use super::fixture::{
    LOGICAL_PACKED, LOGICAL_VIEWPORT, REFERENCE_RGBA, offer_at, spatial_scheduler,
};
use super::support::{
    BackendFault, BackendPhase, FakeSurface, limits, reference_presenter, surface,
};

#[test]
fn private_port_semantic_call_is_sealed_to_the_runtime_paint_frame() {
    fn typecheck<P: SpatialPresenterPortV2>(
        presenter: &mut P,
        frame: RuntimePaintFrameV2<'_>,
        surface: SpatialSurfaceTupleV2,
        submission: SubmissionId,
    ) {
        let result = presenter.present_offer(frame, surface, || Ok(submission));
        let _: Result<u64, SpatialPresentErrorKindV2> = result;
    }

    let _ = typecheck::<SpatialReferencePresenterV2<FakeSurface>>;
}

#[test]
fn softbuffer_adapter_consumes_exact_owned_native_resource_types() {
    type OwnedContext = Context<OwnedDisplayHandle>;
    type OwnedSurface = Surface<OwnedDisplayHandle, Arc<Window>>;
    type OwnedConstructor = fn(
        OwnedContext,
        OwnedSurface,
        Arc<Window>,
        super::super::super::spatial_v2::SpatialPresentationLimitsV2,
    ) -> SpatialReferencePresenterV2;

    let constructor: OwnedConstructor = SpatialReferencePresenterV2::from_owned_parts;
    let notify: fn(&Window) = spatial_pre_present_notify_source_v2();

    let _ = constructor;
    assert!(std::ptr::fn_addr_eq(
        notify,
        Window::pre_present_notify as fn(&Window),
    ));
}

#[test]
fn reference_presenter_reports_exact_frame_tables_and_premultiplied_pixels() {
    let mut scheduler = spatial_scheduler();
    let work = offer_at(&mut scheduler, LOGICAL_VIEWPORT, 10);
    let frame = work
        .paint_frame()
        .expect("spatial offer should carry paint data");
    let spatial = frame.spatial();
    let raster = spatial
        .rasterize_reference(ReferenceRasterLimitsV2::new(4))
        .expect("fixture should rasterize within its exact bound");

    assert_eq!(frame.generation(), work.generation());
    assert_eq!(spatial.viewport(), LOGICAL_VIEWPORT);
    assert_eq!(spatial.shapes().len(), 4);
    assert_eq!(spatial.brushes().len(), 4);
    assert_eq!(spatial.paint_items().len(), 4);
    assert_eq!(spatial.resolved_paints().len(), 4);
    assert_eq!(
        (raster.width(), raster.height(), raster.stride()),
        (4, 1, 16)
    );
    assert_eq!(raster.bytes(), REFERENCE_RGBA);

    let (mut presenter, state) = reference_presenter(BackendFault::None, limits());
    let outcome = present_spatial_offer_v2(
        &mut scheduler,
        &work,
        surface(4, 1, LOGICAL_VIEWPORT),
        &mut presenter,
        fenestra_ui_runtime::prototype::SchedulerTick::new(12),
    )
    .expect("reference presentation should complete");
    let SpatialPresentationOutcomeV2::Completed(receipt) = outcome else {
        panic!("nonzero surface should complete one frame");
    };

    assert_eq!(receipt.generation(), work.generation());
    assert_eq!(presenter.last_successful_digest(), Some(receipt.digest()));
    let state = state.borrow();
    assert_eq!(
        state.phases,
        [
            BackendPhase::Resize(4, 1),
            BackendPhase::Acquire,
            BackendPhase::Copy(4),
            BackendPhase::Notify,
            BackendPhase::Present,
        ]
    );
    assert_eq!(state.pixels, LOGICAL_PACKED);
    assert_eq!(scheduler.stats().controls().items(), 1);
    assert_eq!(scheduler.stats().in_flight().items(), 1);
}
