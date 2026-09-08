use crate::Error;

pub(super) struct Fill {
    pub(super) index: usize,
    pub(super) minimum: u32,
    pub(super) maximum: u32,
    pub(super) weight: u32,
}

/// Reserves minima, then divides remaining space by weight up to each maximum.
pub(super) fn allocate(fills: &[Fill], available: u32) -> Result<Vec<u32>, Error> {
    let mut sizes: Vec<_> = fills.iter().map(|fill| fill.minimum).collect();
    let reserved = sizes.iter().try_fold(0u64, |sum, &size| {
        sum.checked_add(u64::from(size))
            .ok_or(Error::CapacityOverflow)
    })?;
    let mut remaining = u64::from(available).saturating_sub(reserved);
    let mut active: Vec<_> = (0..fills.len())
        .filter(|&index| fills[index].minimum < fills[index].maximum)
        .collect();
    while remaining > 0 && !active.is_empty() {
        let weights = active.iter().try_fold(0u64, |sum, &index| {
            sum.checked_add(u64::from(fills[index].weight))
                .ok_or(Error::CapacityOverflow)
        })?;
        // The available extent fits i32 and validated weights fit u16, so the
        // product fits u64. Freeze capped shares before allocating any others.
        let capped: Vec<_> = active
            .iter()
            .copied()
            .filter(|&index| {
                let share = remaining * u64::from(fills[index].weight) / weights;
                share >= u64::from(fills[index].maximum - fills[index].minimum)
            })
            .collect();
        if !capped.is_empty() {
            for index in capped {
                remaining -= u64::from(fills[index].maximum - fills[index].minimum);
                sizes[index] = fills[index].maximum;
            }
            active.retain(|&index| sizes[index] < fills[index].maximum);
            continue;
        }

        let mut leftovers = remaining;
        for &index in &active {
            let share = remaining * u64::from(fills[index].weight) / weights;
            sizes[index] += share as u32;
            leftovers -= share;
        }
        // Largest fractional remainders conserve every available pixel. Equal
        // fractions use the original child order, independent of sort details.
        active.sort_by(|&left, &right| {
            let remainder = |index: usize| remaining * u64::from(fills[index].weight) % weights;
            remainder(right)
                .cmp(&remainder(left))
                .then_with(|| left.cmp(&right))
        });
        for &index in active.iter().take(leftovers as usize) {
            sizes[index] += 1;
        }
        break;
    }
    Ok(sizes)
}
