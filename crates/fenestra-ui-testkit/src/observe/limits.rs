use crate::error::{HarnessError, HarnessErrorKind, HarnessLimitKind};
use crate::fixture::HarnessLimitsV1;
use crate::semantic::NodePathV1;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy)]
pub(super) struct ObservationLimitCrossingsV1 {
    live_memberships: bool,
    path_depth: bool,
    normalized_nodes: bool,
    normalized_fragments: bool,
    normalized_properties: bool,
}

impl ObservationLimitCrossingsV1 {
    pub(super) const fn new(
        live_memberships: bool,
        path_depth: bool,
        normalized_nodes: bool,
        normalized_fragments: bool,
        normalized_properties: bool,
    ) -> Self {
        Self {
            live_memberships,
            path_depth,
            normalized_nodes,
            normalized_fragments,
            normalized_properties,
        }
    }

    pub(super) fn record(&mut self, kind: HarnessLimitKind) {
        match kind {
            HarnessLimitKind::LiveMemberships => self.live_memberships = true,
            HarnessLimitKind::PathDepth => self.path_depth = true,
            HarnessLimitKind::NormalizedNodes => self.normalized_nodes = true,
            HarnessLimitKind::NormalizedFragments => self.normalized_fragments = true,
            HarnessLimitKind::NormalizedProperties => self.normalized_properties = true,
            _ => {}
        }
    }
}

pub(super) const fn first_observation_limit_v1(
    crossings: ObservationLimitCrossingsV1,
) -> Option<HarnessLimitKind> {
    if crossings.live_memberships {
        Some(HarnessLimitKind::LiveMemberships)
    } else if crossings.path_depth {
        Some(HarnessLimitKind::PathDepth)
    } else if crossings.normalized_nodes {
        Some(HarnessLimitKind::NormalizedNodes)
    } else if crossings.normalized_fragments {
        Some(HarnessLimitKind::NormalizedFragments)
    } else if crossings.normalized_properties {
        Some(HarnessLimitKind::NormalizedProperties)
    } else {
        None
    }
}

pub(super) fn ensure_path_depth(
    path: &NodePathV1,
    limits: HarnessLimitsV1,
) -> Result<(), HarnessError> {
    if path.depth() > limits.path_depth() {
        Err(HarnessError::limit(HarnessLimitKind::PathDepth))
    } else {
        Ok(())
    }
}

pub(super) fn ensure_next_count(
    current: usize,
    limit: usize,
    kind: HarnessLimitKind,
) -> Result<(), HarnessError> {
    checked_increment(current, limit, kind).map(|_| ())
}

pub(super) fn checked_increment(
    current: usize,
    limit: usize,
    kind: HarnessLimitKind,
) -> Result<usize, HarnessError> {
    let next = current.checked_add(1).ok_or_else(arithmetic_error)?;
    if next > limit {
        Err(HarnessError::limit(kind))
    } else {
        Ok(next)
    }
}

pub(super) fn arithmetic_error() -> HarnessError {
    HarnessError::new(HarnessErrorKind::ArithmeticExhausted)
}

pub(super) fn state_mismatch() -> HarnessError {
    HarnessError::new(HarnessErrorKind::StateMismatch)
}
