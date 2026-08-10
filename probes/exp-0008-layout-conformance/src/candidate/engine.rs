#[cfg(test)]
use std::sync::Arc;
#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};

use fenestra_ui_layout::prototype::{
    LayoutEngineErrorV1, LayoutEngineV1, LayoutErrorLocationV1, LayoutNodeV1, LayoutOutputV1,
    ValidatedLayoutInputV1,
};

use super::admission::validate_candidate_input_v1;
use super::backend::solve_taffy_layout_v1;
use super::conversion::{CandidateRawRecordV1, convert_candidate_output_v1};
use super::error::{invariant_error_v1, map_candidate_profile_error_v1};

/// Creates call-local Taffy trees for the bounded version-1 stack probe.
#[derive(Debug, Default)]
pub struct TaffyStackEngineV1 {
    #[cfg(test)]
    backend_counter: Option<Arc<AtomicUsize>>,
}

impl TaffyStackEngineV1 {
    /// Creates a candidate adapter with no retained layout state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            #[cfg(test)]
            backend_counter: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn new_with_backend_counter(counter: Arc<AtomicUsize>) -> Self {
        Self {
            backend_counter: Some(counter),
        }
    }

    fn enter_backend(&self) {
        #[cfg(test)]
        if let Some(counter) = &self.backend_counter {
            counter.fetch_add(1, Ordering::SeqCst);
        }
    }
}

impl LayoutEngineV1 for TaffyStackEngineV1 {
    fn compute(
        &self,
        input: ValidatedLayoutInputV1<'_>,
    ) -> Result<LayoutOutputV1, LayoutEngineErrorV1> {
        validate_candidate_input_v1(input.viewport(), input.nodes())
            .map_err(map_candidate_profile_error_v1)?;

        let raw_records = solve_taffy_layout_v1(input, || self.enter_backend())?;
        validate_raw_structure_v1(input.nodes(), &raw_records)?;
        convert_candidate_output_v1(input.nodes(), &raw_records)
            .map_err(map_candidate_profile_error_v1)
    }
}

fn validate_raw_structure_v1(
    nodes: &[LayoutNodeV1],
    records: &[CandidateRawRecordV1],
) -> Result<(), LayoutEngineErrorV1> {
    if records.len() != nodes.len() {
        return Err(invariant_error_v1(LayoutErrorLocationV1::Output));
    }
    for (index, (node, record)) in nodes
        .iter()
        .copied()
        .zip(records.iter().copied())
        .enumerate()
    {
        if record.key() != node.key() {
            return Err(invariant_error_v1(output_record_location(index)));
        }
    }
    Ok(())
}

fn output_record_location(index: usize) -> LayoutErrorLocationV1 {
    match u32::try_from(index) {
        Ok(index) => LayoutErrorLocationV1::OutputRecord { index },
        Err(_) => LayoutErrorLocationV1::Output,
    }
}
