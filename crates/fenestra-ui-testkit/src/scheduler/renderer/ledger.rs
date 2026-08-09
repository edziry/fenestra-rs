use fenestra_ui_runtime::prototype::{CompletionWatermark, SchedulerTick, SubmissionId};

use super::{
    FakeRendererCapacityV1, FakeRendererErrorKindV1, FakeRendererErrorV1, SyntheticResourceIdV1,
    SyntheticResourceUseV1,
};

pub(super) const RETIREMENT_ENVELOPE_BYTES: usize = 32;

#[derive(Clone)]
struct RetirementRecordV1 {
    id: SyntheticResourceIdV1,
    synthetic_bytes: usize,
    last_use: Option<SubmissionId>,
    earliest_tick: SchedulerTick,
    latest_tick: SchedulerTick,
}

impl RetirementRecordV1 {
    fn accounted_bytes(&self) -> Result<usize, FakeRendererErrorV1> {
        RETIREMENT_ENVELOPE_BYTES
            .checked_add(self.synthetic_bytes)
            .ok_or_else(capacity_error)
    }
}

#[derive(Clone)]
pub(super) struct RetirementLedgerV1 {
    records: Vec<RetirementRecordV1>,
    accounted_bytes: usize,
}

impl RetirementLedgerV1 {
    pub(super) const fn new() -> Self {
        Self {
            records: Vec::new(),
            accounted_bytes: 0,
        }
    }

    pub(super) fn project_offer(
        &self,
        resources: &[SyntheticResourceUseV1],
        tick: SchedulerTick,
        capacity: FakeRendererCapacityV1,
    ) -> Result<Self, FakeRendererErrorV1> {
        let mut projected_items = self.records.len();
        let mut projected_bytes = self.accounted_bytes;
        for (index, resource) in resources.iter().copied().enumerate() {
            if resources[..index]
                .iter()
                .any(|prior| prior.id() == resource.id())
            {
                return Err(capacity_error());
            }
            if let Some(record) = self
                .records
                .iter()
                .find(|record| record.id == resource.id())
            {
                if record.synthetic_bytes != resource.synthetic_bytes() {
                    return Err(capacity_error());
                }
            } else {
                projected_items = projected_items.checked_add(1).ok_or_else(capacity_error)?;
                projected_bytes = projected_bytes
                    .checked_add(
                        RETIREMENT_ENVELOPE_BYTES
                            .checked_add(resource.synthetic_bytes())
                            .ok_or_else(capacity_error)?,
                    )
                    .ok_or_else(capacity_error)?;
            }
        }
        if projected_items > capacity.max_items() {
            return Err(capacity_error());
        }
        if projected_bytes > capacity.max_bytes() {
            return Err(capacity_error());
        }

        let mut projected = self.clone();
        for resource in resources {
            if let Some(record) = projected
                .records
                .iter_mut()
                .find(|record| record.id == resource.id())
            {
                record.latest_tick = tick;
            } else {
                projected.records.push(RetirementRecordV1 {
                    id: resource.id(),
                    synthetic_bytes: resource.synthetic_bytes(),
                    last_use: None,
                    earliest_tick: tick,
                    latest_tick: tick,
                });
            }
        }
        projected.accounted_bytes = projected_bytes;
        Ok(projected)
    }

    pub(super) fn bind_submission(
        &mut self,
        resources: &[SyntheticResourceUseV1],
        submission: SubmissionId,
    ) {
        for resource in resources {
            if let Some(record) = self
                .records
                .iter_mut()
                .find(|record| record.id == resource.id())
            {
                record.last_use = Some(submission);
            }
        }
    }

    pub(super) fn project_completion(
        &self,
        watermark: CompletionWatermark,
    ) -> Result<Self, FakeRendererErrorV1> {
        let mut projected = self.clone();
        projected.records.retain(|record| {
            record.last_use.is_none_or(|submission| {
                submission.epoch() != watermark.epoch() || submission.token() > watermark.token()
            })
        });
        projected.accounted_bytes = projected.recompute_bytes()?;
        Ok(projected)
    }

    pub(super) fn items(&self) -> usize {
        self.records.len()
    }

    pub(super) const fn accounted_bytes(&self) -> usize {
        self.accounted_bytes
    }

    pub(super) fn earliest_tick(&self) -> Option<SchedulerTick> {
        self.records.iter().map(|record| record.earliest_tick).min()
    }

    pub(super) fn latest_tick(&self) -> Option<SchedulerTick> {
        self.records.iter().map(|record| record.latest_tick).max()
    }

    fn recompute_bytes(&self) -> Result<usize, FakeRendererErrorV1> {
        self.records.iter().try_fold(0usize, |total, record| {
            total
                .checked_add(record.accounted_bytes()?)
                .ok_or_else(capacity_error)
        })
    }
}

fn capacity_error() -> FakeRendererErrorV1 {
    FakeRendererErrorV1::new(FakeRendererErrorKindV1::CapacityExceeded)
}
