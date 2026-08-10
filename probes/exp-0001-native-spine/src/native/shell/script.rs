use fenestra_ui_testkit::prototype::HeadlessPointerTargetV1;

use super::super::artifact::NativeProbeResultV1;
use super::super::trace::NativeFailureCauseV1;
use super::super::types::NativePhysicalPointV1;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum NativeRunDirectiveV1 {
    AwaitInitialPublication,
    AwaitRedraw,
    ScriptPrimaryPress { physical: NativePhysicalPointV1 },
    RequestLogicalResize { width: u32, height: u32 },
    ScriptClose,
    Exit(NativeProbeResultV1),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum NativeRunEvidenceV1 {
    InitialPublished {
        runtime_generation: u64,
        surface_generation: u64,
        scale_micros: u32,
    },
    Presented {
        runtime_generation: u64,
        surface_generation: u64,
        frame: u64,
        submission: u64,
        completion_control: u64,
    },
    PointerTarget(HeadlessPointerTargetV1),
    ResizePublished {
        runtime_generation: u64,
        surface_generation: u64,
    },
    Stopped {
        control: u64,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ScriptStateV1 {
    Initial,
    FirstRedraw,
    Pointer,
    Resize,
    SecondRedraw,
    Close,
    Exit(NativeProbeResultV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeReferenceScriptV1 {
    state: ScriptStateV1,
    scale_micros: Option<u32>,
}

impl NativeReferenceScriptV1 {
    pub(crate) const fn new() -> Self {
        Self {
            state: ScriptStateV1::Initial,
            scale_micros: None,
        }
    }

    pub(crate) fn current(self) -> NativeRunDirectiveV1 {
        match self.state {
            ScriptStateV1::Initial => NativeRunDirectiveV1::AwaitInitialPublication,
            ScriptStateV1::FirstRedraw | ScriptStateV1::SecondRedraw => {
                NativeRunDirectiveV1::AwaitRedraw
            }
            ScriptStateV1::Pointer => NativeRunDirectiveV1::ScriptPrimaryPress {
                physical: NativePhysicalPointV1::new(
                    f64::from(self.scale_micros.unwrap_or(0)) * 5.0 / 1_000_000.0,
                    f64::from(self.scale_micros.unwrap_or(0)) * 5.0 / 1_000_000.0,
                ),
            },
            ScriptStateV1::Resize => NativeRunDirectiveV1::RequestLogicalResize {
                width: 360,
                height: 260,
            },
            ScriptStateV1::Close => NativeRunDirectiveV1::ScriptClose,
            ScriptStateV1::Exit(result) => NativeRunDirectiveV1::Exit(result),
        }
    }

    pub(crate) fn advance(
        &mut self,
        evidence: NativeRunEvidenceV1,
    ) -> Result<NativeRunDirectiveV1, NativeFailureCauseV1> {
        let (next, scale) = match (self.state, evidence) {
            (
                ScriptStateV1::Initial,
                NativeRunEvidenceV1::InitialPublished {
                    runtime_generation: 1,
                    surface_generation: 0,
                    scale_micros,
                },
            ) if scale_micros > 0 => (ScriptStateV1::FirstRedraw, Some(scale_micros)),
            (
                ScriptStateV1::FirstRedraw,
                NativeRunEvidenceV1::Presented {
                    runtime_generation: 1,
                    surface_generation: 0,
                    frame: 0,
                    submission: 0,
                    completion_control: 0,
                },
            ) => (ScriptStateV1::Pointer, self.scale_micros),
            (
                ScriptStateV1::Pointer,
                NativeRunEvidenceV1::PointerTarget(HeadlessPointerTargetV1::StaticControl),
            ) => (ScriptStateV1::Resize, self.scale_micros),
            (
                ScriptStateV1::Resize,
                NativeRunEvidenceV1::ResizePublished {
                    runtime_generation: 2,
                    surface_generation: 1,
                },
            ) => (ScriptStateV1::SecondRedraw, self.scale_micros),
            (
                ScriptStateV1::SecondRedraw,
                NativeRunEvidenceV1::Presented {
                    runtime_generation: 2,
                    surface_generation: 1,
                    frame: 1,
                    submission: 1,
                    completion_control: 1,
                },
            ) => (ScriptStateV1::Close, self.scale_micros),
            (ScriptStateV1::Close, NativeRunEvidenceV1::Stopped { control: 2 }) => (
                ScriptStateV1::Exit(NativeProbeResultV1::Pass),
                self.scale_micros,
            ),
            _ => return Err(NativeFailureCauseV1::Invariant),
        };
        self.state = next;
        self.scale_micros = scale;
        Ok(self.current())
    }

    pub(crate) fn finish(
        &mut self,
        result: NativeProbeResultV1,
    ) -> Result<NativeRunDirectiveV1, NativeFailureCauseV1> {
        if result == NativeProbeResultV1::Pass || matches!(self.state, ScriptStateV1::Exit(_)) {
            return Err(NativeFailureCauseV1::Invariant);
        }
        self.state = ScriptStateV1::Exit(result);
        Ok(self.current())
    }
}

pub(crate) const fn classify_native_failure_v1(cause: NativeFailureCauseV1) -> NativeProbeResultV1 {
    match cause {
        NativeFailureCauseV1::EnvironmentScaleChanged
        | NativeFailureCauseV1::SurfaceRepaintUnavailable => NativeProbeResultV1::Adapt,
        NativeFailureCauseV1::InvalidScale
        | NativeFailureCauseV1::InvalidPoint
        | NativeFailureCauseV1::Arithmetic
        | NativeFailureCauseV1::WidthLimit
        | NativeFailureCauseV1::HeightLimit
        | NativeFailureCauseV1::PixelLimit
        | NativeFailureCauseV1::ByteLimit
        | NativeFailureCauseV1::UnsupportedAlpha
        | NativeFailureCauseV1::Storage
        | NativeFailureCauseV1::Runtime
        | NativeFailureCauseV1::Oracle
        | NativeFailureCauseV1::Scheduler
        | NativeFailureCauseV1::PrePresent
        | NativeFailureCauseV1::Presenter
        | NativeFailureCauseV1::Trace
        | NativeFailureCauseV1::Timeout
        | NativeFailureCauseV1::Invariant => NativeProbeResultV1::Stop,
    }
}
