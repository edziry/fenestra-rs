use std::num::NonZeroU32;
use std::sync::Arc;

use fenestra_ui_runtime::prototype::{RuntimePaintFrameV2, SubmissionId};
use fenestra_ui_spatial::prototype::ReferenceRasterLimitsV2;
use softbuffer::{Context, Surface};
use winit::event_loop::OwnedDisplayHandle;
use winit::window::Window;

use super::backend::{
    SpatialOwnedSurfacePortV2, SpatialPresenterBufferPortV2, SpatialPresenterSurfacePortV2,
};
use super::port::SpatialPresenterPortV2;
use super::stage::stage_reference_pixels_with_reserver_v2;
use super::types::{
    SpatialPresentErrorKindV2, SpatialPresentationLimitsV2, SpatialRasterInputV2,
    SpatialSurfaceTupleV2,
};

pub(crate) struct SpatialReferencePresenterV2<S = SpatialOwnedSurfacePortV2> {
    surface: S,
    limits: SpatialPresentationLimitsV2,
    last_successful_digest: Option<u64>,
}

impl SpatialReferencePresenterV2 {
    pub(crate) fn from_owned_parts(
        context: Context<OwnedDisplayHandle>,
        surface: Surface<OwnedDisplayHandle, Arc<Window>>,
        window: Arc<Window>,
        limits: SpatialPresentationLimitsV2,
    ) -> Self {
        Self {
            surface: SpatialOwnedSurfacePortV2::new(context, surface, window),
            limits,
            last_successful_digest: None,
        }
    }
}

impl<S> SpatialReferencePresenterV2<S> {
    #[cfg(test)]
    pub(crate) const fn from_surface_port_for_test(
        surface: S,
        limits: SpatialPresentationLimitsV2,
    ) -> Self {
        Self {
            surface,
            limits,
            last_successful_digest: None,
        }
    }

    pub(crate) const fn last_successful_digest(&self) -> Option<u64> {
        self.last_successful_digest
    }
}

impl<S: SpatialPresenterSurfacePortV2> SpatialPresenterPortV2 for SpatialReferencePresenterV2<S> {
    fn present_offer<A>(
        &mut self,
        frame: RuntimePaintFrameV2<'_>,
        surface: SpatialSurfaceTupleV2,
        accept_once: A,
    ) -> Result<u64, SpatialPresentErrorKindV2>
    where
        A: FnOnce() -> Result<SubmissionId, SpatialPresentErrorKindV2>,
    {
        if frame.spatial().viewport() != surface.logical() {
            return Err(SpatialPresentErrorKindV2::ViewportMismatch);
        }
        let raster = frame
            .spatial()
            .rasterize_reference(ReferenceRasterLimitsV2::new(self.limits.reference_pixels()))
            .map_err(|_| SpatialPresentErrorKindV2::ReferenceRaster)?;
        let input = SpatialRasterInputV2::new(
            raster.width(),
            raster.height(),
            raster.stride(),
            raster.bytes(),
        );
        let staged = stage_reference_pixels_with_reserver_v2(
            surface.logical(),
            surface.physical(),
            input,
            self.limits,
            reserve,
        )?
        .ok_or(SpatialPresentErrorKindV2::Invariant)?;
        let width = NonZeroU32::new(staged.physical().width())
            .ok_or(SpatialPresentErrorKindV2::Invariant)?;
        let height = NonZeroU32::new(staged.physical().height())
            .ok_or(SpatialPresentErrorKindV2::Invariant)?;
        self.surface
            .resize(width, height)
            .map_err(|_| SpatialPresentErrorKindV2::Presenter)?;
        let mut buffer = self
            .surface
            .acquire()
            .map_err(|_| SpatialPresentErrorKindV2::Presenter)?;
        buffer
            .copy_pixels(staged.pixels())
            .map_err(|_| SpatialPresentErrorKindV2::Presenter)?;
        buffer
            .pre_present_notify()
            .map_err(|_| SpatialPresentErrorKindV2::PrePresent)?;
        let _submission = accept_once()?;
        buffer
            .present()
            .map_err(|_| SpatialPresentErrorKindV2::Presenter)?;
        let digest = staged.digest();
        self.last_successful_digest = Some(digest);
        Ok(digest)
    }
}

fn reserve(count: usize) -> Result<Vec<u32>, ()> {
    let mut pixels = Vec::new();
    pixels.try_reserve_exact(count).map_err(|_| ())?;
    Ok(pixels)
}
