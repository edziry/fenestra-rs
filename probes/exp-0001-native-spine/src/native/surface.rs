use fenestra_ui_runtime::prototype::HeadlessSurface;

use super::types::{NativeContractErrorKindV1, NativePhysicalExtentV1, NativeScaleFactorV1};

const MAX_PHYSICAL_AXIS: u32 = 4_096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct NativeSurfaceGenerationV1(u64);

impl NativeSurfaceGenerationV1 {
    pub(super) const fn get(self) -> u64 {
        self.0
    }

    fn next(self) -> Result<Self, NativeContractErrorKindV1> {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or(NativeContractErrorKindV1::ArithmeticExhausted)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum NativeSurfaceChangeV1 {
    Initialized,
    NoChange,
    LogicalResize,
    NativeOnly,
    Suspended,
    Restored,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct NativeSurfaceTupleV1 {
    generation: NativeSurfaceGenerationV1,
    physical: NativePhysicalExtentV1,
    scale: NativeScaleFactorV1,
    logical: HeadlessSurface,
}

impl NativeSurfaceTupleV1 {
    pub(super) const fn generation(self) -> NativeSurfaceGenerationV1 {
        self.generation
    }

    pub(super) const fn physical(self) -> NativePhysicalExtentV1 {
        self.physical
    }

    pub(super) const fn scale(self) -> NativeScaleFactorV1 {
        self.scale
    }

    pub(super) const fn logical_surface(self) -> HeadlessSurface {
        self.logical
    }

    const fn is_suspended(self) -> bool {
        self.logical.width() == 0 || self.logical.height() == 0
    }
}

pub(super) struct NativeSurfaceStateV1 {
    accepted: Option<NativeSurfaceTupleV1>,
    pending: Option<NativeSurfaceTupleV1>,
}

impl NativeSurfaceStateV1 {
    pub(super) const fn new() -> Self {
        Self {
            accepted: None,
            pending: None,
        }
    }

    pub(super) fn observe(
        &mut self,
        physical: NativePhysicalExtentV1,
        scale: f64,
    ) -> Result<NativeSurfaceChangeV1, NativeContractErrorKindV1> {
        let scale = NativeScaleFactorV1::try_from_f64(scale)?;
        if physical.width() > MAX_PHYSICAL_AXIS {
            return Err(NativeContractErrorKindV1::LimitExceeded(
                super::types::NativeLimitKindV1::Width,
            ));
        }
        if physical.height() > MAX_PHYSICAL_AXIS {
            return Err(NativeContractErrorKindV1::LimitExceeded(
                super::types::NativeLimitKindV1::Height,
            ));
        }
        let logical = scale.logical_surface(physical)?;
        if self
            .accepted
            .is_some_and(|accepted| accepted.scale != scale)
        {
            return Err(NativeContractErrorKindV1::EnvironmentScaleChanged);
        }
        if self
            .pending
            .is_some_and(|pending| pending.physical == physical && pending.scale == scale)
        {
            return Ok(NativeSurfaceChangeV1::NoChange);
        }
        let Some(accepted) = self.accepted else {
            self.pending = Some(NativeSurfaceTupleV1 {
                generation: NativeSurfaceGenerationV1(0),
                physical,
                scale,
                logical,
            });
            return Ok(NativeSurfaceChangeV1::Initialized);
        };
        if accepted.physical == physical && accepted.scale == scale {
            self.pending = None;
            return Ok(NativeSurfaceChangeV1::NoChange);
        }
        let generation = accepted.generation.next()?;
        let next = NativeSurfaceTupleV1 {
            generation,
            physical,
            scale,
            logical,
        };
        let change = classify(accepted.logical, logical);
        self.pending = Some(next);
        Ok(change)
    }

    pub(super) const fn accepted_tuple(&self) -> Option<NativeSurfaceTupleV1> {
        self.accepted
    }

    pub(super) const fn input_tuple(&self) -> Option<NativeSurfaceTupleV1> {
        self.accepted
    }

    pub(super) const fn pending_tuple(&self) -> Option<NativeSurfaceTupleV1> {
        self.pending
    }

    pub(super) const fn pending_count(&self) -> usize {
        if self.pending.is_some() { 1 } else { 0 }
    }

    pub(super) fn promote_pending(
        &mut self,
        expected: NativeSurfaceTupleV1,
    ) -> Result<NativeSurfaceTupleV1, NativeContractErrorKindV1> {
        if self.pending != Some(expected) {
            return Err(NativeContractErrorKindV1::Invariant);
        }
        let promoted = self
            .pending
            .take()
            .ok_or(NativeContractErrorKindV1::Invariant)?;
        self.accepted = Some(promoted);
        Ok(promoted)
    }

    pub(super) fn accepted_is_suspended(&self) -> bool {
        self.accepted
            .is_some_and(NativeSurfaceTupleV1::is_suspended)
    }

    pub(super) fn pending_is_suspended(&self) -> bool {
        self.pending.is_some_and(NativeSurfaceTupleV1::is_suspended)
    }

    #[cfg(test)]
    pub(super) fn force_generation_for_test(&mut self, generation: u64) {
        if let Some(accepted) = &mut self.accepted {
            accepted.generation = NativeSurfaceGenerationV1(generation);
        }
    }
}

fn classify(previous: HeadlessSurface, current: HeadlessSurface) -> NativeSurfaceChangeV1 {
    let was_suspended = previous.width() == 0 || previous.height() == 0;
    let is_suspended = current.width() == 0 || current.height() == 0;
    if !was_suspended && is_suspended {
        NativeSurfaceChangeV1::Suspended
    } else if was_suspended && !is_suspended {
        NativeSurfaceChangeV1::Restored
    } else if previous != current {
        NativeSurfaceChangeV1::LogicalResize
    } else {
        NativeSurfaceChangeV1::NativeOnly
    }
}
