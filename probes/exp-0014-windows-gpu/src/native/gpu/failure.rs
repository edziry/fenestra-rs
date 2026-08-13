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
