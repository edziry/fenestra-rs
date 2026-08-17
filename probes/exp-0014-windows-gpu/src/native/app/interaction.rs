use fenestra_ui_ir::prototype::{PropertyId, PropertyValue};
use winit::event_loop::ActiveEventLoop;

use super::{NativeGpuApplicationV1, viewport};
use crate::{
    ArtifactEventV1, ArtifactTerminalV1, GpuSurfaceExtentV1, InteractiveMilestoneV1,
    InteractiveProbeErrorKindV1,
};

const TONE_PROPERTY: PropertyId = PropertyId::new(4);
const MUTATED_TONE: PropertyValue = PropertyValue::Rgba8([80, 40, 24, 255]);

impl NativeGpuApplicationV1 {
    pub(super) fn pointer_move(&mut self) -> Result<(), InteractiveProbeErrorKindV1> {
        if self.next_required() == Some(InteractiveMilestoneV1::PointerMove) {
            self.observe(ArtifactEventV1::PointerMove)?;
            self.update_title();
        }
        Ok(())
    }

    pub(super) fn pointer_press(&mut self) -> Result<(), InteractiveProbeErrorKindV1> {
        if self.next_required() != Some(InteractiveMilestoneV1::PointerPress) {
            return Ok(());
        }
        self.observe(ArtifactEventV1::PointerPress)?;
        let scheduler = self
            .scheduler
            .as_mut()
            .ok_or(InteractiveProbeErrorKindV1::Runtime)?;
        let snapshot = scheduler.committed();
        let mut transaction = scheduler.begin_transaction();
        transaction
            .set_property(snapshot.root(), TONE_PROPERTY, MUTATED_TONE)
            .map_err(|_| InteractiveProbeErrorKindV1::Runtime)?;
        let tick = self.take_tick()?;
        self.scheduler
            .as_mut()
            .ok_or(InteractiveProbeErrorKindV1::Runtime)?
            .commit(transaction, tick)
            .map_err(|_| InteractiveProbeErrorKindV1::Runtime)?;
        self.arm_next_action(tick)
    }

    pub(super) fn resized(&mut self, event_loop: &ActiveEventLoop, extent: GpuSurfaceExtentV1) {
        let result = if extent.width() == 0 || extent.height() == 0 {
            self.suspend()
        } else if self.next_required() == Some(InteractiveMilestoneV1::Restore) && self.suspended {
            self.restore(extent)
        } else {
            self.resize_nonzero(extent)
        };
        if result.is_err() {
            self.abort(event_loop, InteractiveProbeErrorKindV1::Runtime);
        }
    }

    fn resize_nonzero(
        &mut self,
        extent: GpuSurfaceExtentV1,
    ) -> Result<(), InteractiveProbeErrorKindV1> {
        self.gpu
            .as_mut()
            .ok_or(InteractiveProbeErrorKindV1::Runtime)?
            .resize(extent);
        let next_required = self.next_required();
        let required_resize = stage_required_resize(
            &mut self.pending_resize,
            next_required,
            self.last_present_extent,
            extent,
        );
        if self.runtime_extent != Some(extent) {
            let scheduler = self
                .scheduler
                .as_mut()
                .ok_or(InteractiveProbeErrorKindV1::Runtime)?;
            let mut transaction = scheduler.begin_transaction();
            transaction
                .resize_spatial(viewport(extent))
                .map_err(|_| InteractiveProbeErrorKindV1::Runtime)?;
            let tick = self.take_tick()?;
            self.scheduler
                .as_mut()
                .ok_or(InteractiveProbeErrorKindV1::Runtime)?
                .commit(transaction, tick)
                .map_err(|_| InteractiveProbeErrorKindV1::Runtime)?;
            self.runtime_extent = Some(extent);
            if required_resize {
                self.arm_next_action(tick)?;
            }
        }
        Ok(())
    }

    pub(super) fn suspend(&mut self) -> Result<(), InteractiveProbeErrorKindV1> {
        if self.next_required() == Some(InteractiveMilestoneV1::Suspend) {
            self.observe(ArtifactEventV1::Suspend)?;
            self.suspended = true;
            self.redraw_armed = false;
            self.update_title();
        }
        Ok(())
    }

    pub(super) fn restore(
        &mut self,
        extent: GpuSurfaceExtentV1,
    ) -> Result<(), InteractiveProbeErrorKindV1> {
        if self.runtime_extent != Some(extent) {
            return Err(InteractiveProbeErrorKindV1::Runtime);
        }
        self.gpu
            .as_mut()
            .ok_or(InteractiveProbeErrorKindV1::Runtime)?
            .resize(extent);
        self.observe(ArtifactEventV1::Restore)?;
        self.suspended = false;
        let tick = self.take_tick()?;
        self.scheduler
            .as_mut()
            .ok_or(InteractiveProbeErrorKindV1::Runtime)?
            .request_current_frame(tick)
            .map_err(|_| InteractiveProbeErrorKindV1::Runtime)?;
        self.arm_next_action(tick)
    }

    pub(super) fn close(&mut self, event_loop: &ActiveEventLoop) {
        let terminal = if self.next_required() == Some(InteractiveMilestoneV1::Close) {
            ArtifactTerminalV1::Pass
        } else {
            ArtifactTerminalV1::Stop
        };
        if let Err(error) = self.observe(ArtifactEventV1::Close) {
            self.abort(event_loop, error);
            return;
        }
        self.finish(event_loop, terminal);
    }
}

fn stage_required_resize(
    pending: &mut Option<GpuSurfaceExtentV1>,
    next_required: Option<InteractiveMilestoneV1>,
    last_present: Option<GpuSurfaceExtentV1>,
    extent: GpuSurfaceExtentV1,
) -> bool {
    if next_required != Some(InteractiveMilestoneV1::Resize) {
        return false;
    }
    *pending = (last_present != Some(extent)).then_some(extent);
    pending.is_some()
}

#[cfg(test)]
mod tests {
    use super::stage_required_resize;
    use crate::{GpuSurfaceExtentV1, InteractiveMilestoneV1};

    #[test]
    fn resize_drag_stages_the_latest_extent_before_evidence() {
        let previous = GpuSurfaceExtentV1::new(640, 420);
        let first = GpuSurfaceExtentV1::new(700, 460);
        let latest = GpuSurfaceExtentV1::new(760, 500);
        let mut pending = None;

        assert!(stage_required_resize(
            &mut pending,
            Some(InteractiveMilestoneV1::Resize),
            Some(previous),
            first,
        ));
        assert!(stage_required_resize(
            &mut pending,
            Some(InteractiveMilestoneV1::Resize),
            Some(previous),
            latest,
        ));
        assert_eq!(pending, Some(latest));
        assert!(!stage_required_resize(
            &mut pending,
            Some(InteractiveMilestoneV1::Resize),
            Some(previous),
            previous,
        ));
        assert_eq!(pending, None);
    }
}
