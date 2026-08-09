use super::error::CapacityKind;

pub(crate) enum StateEditError {
    Capacity(CapacityKind),
    Invariant,
}

pub(crate) struct StructuralTracker {
    used: usize,
    limit: usize,
}

impl StructuralTracker {
    pub(crate) fn new(limit: usize) -> Self {
        Self { used: 0, limit }
    }

    pub(crate) fn reserve(&mut self, amount: usize) -> Result<(), StateEditError> {
        let next = self
            .used
            .checked_add(amount)
            .ok_or(StateEditError::Capacity(CapacityKind::StructuralChanges))?;
        if next > self.limit {
            return Err(StateEditError::Capacity(CapacityKind::StructuralChanges));
        }
        self.used = next;
        Ok(())
    }

    pub(crate) fn remaining(&self) -> usize {
        self.limit.saturating_sub(self.used)
    }
}
