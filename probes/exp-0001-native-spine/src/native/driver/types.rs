use fenestra_ui_runtime::prototype::{ControlSequence, FrameId, RuntimeGeneration, SubmissionId};

use super::super::raster::CpuFrameV1;
use super::super::surface::NativeSurfaceGenerationV1;
use super::super::trace::NativeFailureCauseV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeDriverActionV1 {
    Idle,
    RequestFrame {
        generation: RuntimeGeneration,
        surface_generation: NativeSurfaceGenerationV1,
    },
    Suspended {
        generation: RuntimeGeneration,
        surface_generation: NativeSurfaceGenerationV1,
    },
    StopRenderer {
        control: ControlSequence,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeRedrawResultV1 {
    Ignored,
    Presented {
        frame: FrameId,
        submission: SubmissionId,
        completion_control: ControlSequence,
    },
}

pub(crate) trait PresenterPortV1 {
    fn present_offer<A>(
        &mut self,
        frame: CpuFrameV1,
        accept_once: A,
    ) -> Result<(), NativeFailureCauseV1>
    where
        A: FnOnce() -> Result<SubmissionId, NativeFailureCauseV1>;
}
