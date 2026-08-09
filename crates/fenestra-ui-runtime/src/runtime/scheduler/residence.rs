use super::{
    QueueCapacity, SchedulerError, SchedulerErrorKind, SchedulerLane, SchedulerState,
    SchedulerTick, UiScheduler,
};

impl UiScheduler {
    pub(super) fn observe_tick(&mut self, tick: SchedulerTick) -> Result<(), SchedulerError> {
        if self.last_tick.is_some_and(|last| tick < last) {
            return Err(SchedulerError::new(
                SchedulerErrorKind::TickRegression,
                None,
            ));
        }
        self.last_tick = Some(tick);
        Ok(())
    }

    pub(super) fn begin_regular_turn(&mut self, tick: SchedulerTick) -> Result<(), SchedulerError> {
        self.observe_tick(tick)?;
        if self.observe_residence(tick)?.is_some() {
            return Err(self.residence_error());
        }
        Ok(())
    }

    pub(super) fn observe_residence(
        &mut self,
        tick: SchedulerTick,
    ) -> Result<Option<SchedulerLane>, SchedulerError> {
        if self.terminal_pressure.is_some() {
            return Ok(self.terminal_pressure);
        }
        let deferred_crossed = match self.deferred.as_ref() {
            Some(work) => residence_crossed(tick, work.accepted_tick, self.capacity.deferred())?,
            None => false,
        };
        let controls_crossed = match self.controls.earliest_tick() {
            Some(accepted_tick) => {
                residence_crossed(tick, accepted_tick, self.capacity.controls())?
            }
            None => false,
        };
        let visual_crossed = match self.visual.as_ref() {
            Some(state) => {
                let (earliest, _) = state.ticks();
                residence_crossed(tick, earliest, self.capacity.visual())?
            }
            None => false,
        };
        let in_flight_crossed = match self.in_flight.front() {
            Some(frame) => residence_crossed(tick, frame.accepted_tick, self.capacity.in_flight())?,
            None => false,
        };
        self.terminal_pressure = if deferred_crossed {
            Some(SchedulerLane::Deferred)
        } else if controls_crossed {
            Some(SchedulerLane::Controls)
        } else if visual_crossed {
            Some(SchedulerLane::Visual)
        } else if in_flight_crossed {
            Some(SchedulerLane::InFlight)
        } else {
            None
        };
        if self.terminal_pressure.is_some() {
            self.state = SchedulerState::Faulted;
        }
        Ok(self.terminal_pressure)
    }

    pub(super) fn residence_error(&self) -> SchedulerError {
        match self.terminal_pressure {
            Some(lane) => SchedulerError::new(SchedulerErrorKind::ResidenceExceeded(lane), None),
            None => SchedulerError::new(SchedulerErrorKind::ArithmeticExhausted, None),
        }
    }
}

fn residence_crossed(
    tick: SchedulerTick,
    earliest: SchedulerTick,
    capacity: QueueCapacity,
) -> Result<bool, SchedulerError> {
    let age = tick
        .get()
        .checked_sub(earliest.get())
        .ok_or_else(|| SchedulerError::new(SchedulerErrorKind::ArithmeticExhausted, None))?;
    Ok(age > capacity.residence_ticks())
}
