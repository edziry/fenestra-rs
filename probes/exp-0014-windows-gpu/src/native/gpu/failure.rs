use std::sync::{Arc, Mutex};

use crate::GpuPresentErrorKindV1;

#[derive(Clone)]
pub(super) struct GpuFailureStateV1 {
    failure: Arc<Mutex<Option<GpuPresentErrorKindV1>>>,
}

impl GpuFailureStateV1 {
    pub(super) fn new() -> Self {
        Self {
            failure: Arc::new(Mutex::new(None)),
        }
    }

    pub(super) fn record(&self, failure: GpuPresentErrorKindV1) {
        let Ok(mut current) = self.failure.lock() else {
            return;
        };
        if current.is_none() || failure == GpuPresentErrorKindV1::OutOfMemory {
            *current = Some(failure);
        }
    }

    pub(super) fn take(&self) -> Option<GpuPresentErrorKindV1> {
        self.failure
            .lock()
            .map_or(Some(GpuPresentErrorKindV1::Renderer), |mut current| {
                current.take()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::GpuFailureStateV1;
    use crate::GpuPresentErrorKindV1;

    #[test]
    fn out_of_memory_has_priority_and_failures_are_consumed() {
        let failures = GpuFailureStateV1::new();
        failures.record(GpuPresentErrorKindV1::Renderer);
        failures.record(GpuPresentErrorKindV1::OutOfMemory);
        failures.record(GpuPresentErrorKindV1::Surface);

        assert_eq!(failures.take(), Some(GpuPresentErrorKindV1::OutOfMemory));
        assert_eq!(failures.take(), None);
    }
}
