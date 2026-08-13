/// Ordered milestones required by the interactive operator protocol.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InteractiveMilestoneV1 {
    /// A target-compatible hardware adapter was admitted.
    Adapter,
    /// The initial runtime generation was presented and completed.
    InitialPresent,
    /// Native pointer movement was observed.
    PointerMove,
    /// A native primary-button press was observed.
    PointerPress,
    /// The pointer mutation generation was presented and completed.
    MutationPresent,
    /// A distinct nonzero native resize was observed.
    Resize,
    /// The resize generation was presented and completed.
    ResizePresent,
    /// Native presentation became absent.
    Suspend,
    /// Native presentation became available again.
    Restore,
    /// A frame was presented after restoration.
    RestorePresent,
    /// The native window was closed normally.
    Close,
}

impl InteractiveMilestoneV1 {
    /// Complete required milestone order.
    pub const ALL: [Self; 11] = [
        Self::Adapter,
        Self::InitialPresent,
        Self::PointerMove,
        Self::PointerPress,
        Self::MutationPresent,
        Self::Resize,
        Self::ResizePresent,
        Self::Suspend,
        Self::Restore,
        Self::RestorePresent,
        Self::Close,
    ];
}

/// Typed observation supplied to the interactive evidence reducer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InteractiveObservationV1 {
    /// A target-compatible hardware adapter was admitted.
    Adapter,
    /// The initial generation completed presentation.
    InitialPresent {
        /// Presented runtime generation.
        generation: u64,
    },
    /// Native pointer movement was observed.
    PointerMove,
    /// A native primary-button press was observed.
    PointerPress,
    /// The mutation generation completed presentation.
    MutationPresent {
        /// Presented runtime generation.
        generation: u64,
    },
    /// A distinct nonzero resize was observed.
    Resize,
    /// The resize generation completed presentation.
    ResizePresent {
        /// Presented runtime generation.
        generation: u64,
    },
    /// Presentation became absent.
    Suspend,
    /// Presentation became available again.
    Restore,
    /// A generation completed presentation after restoration.
    RestorePresent {
        /// Presented runtime generation.
        generation: u64,
    },
    /// The native window was closed normally.
    Close,
}

impl InteractiveObservationV1 {
    const fn milestone(self) -> InteractiveMilestoneV1 {
        match self {
            Self::Adapter => InteractiveMilestoneV1::Adapter,
            Self::InitialPresent { .. } => InteractiveMilestoneV1::InitialPresent,
            Self::PointerMove => InteractiveMilestoneV1::PointerMove,
            Self::PointerPress => InteractiveMilestoneV1::PointerPress,
            Self::MutationPresent { .. } => InteractiveMilestoneV1::MutationPresent,
            Self::Resize => InteractiveMilestoneV1::Resize,
            Self::ResizePresent { .. } => InteractiveMilestoneV1::ResizePresent,
            Self::Suspend => InteractiveMilestoneV1::Suspend,
            Self::Restore => InteractiveMilestoneV1::Restore,
            Self::RestorePresent { .. } => InteractiveMilestoneV1::RestorePresent,
            Self::Close => InteractiveMilestoneV1::Close,
        }
    }
}

/// Closed terminal classifications for one operator sequence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InteractiveResultV1 {
    /// Every required milestone completed in order.
    Pass,
    /// The environment could not satisfy a closed candidate requirement.
    Adapt,
    /// The operator sequence closed before the required prefix completed.
    Stop,
}

/// Closed evidence-reducer failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InteractiveEvidenceErrorKindV1 {
    /// A milestone did not match the next required milestone.
    Order,
    /// A presented generation violated the milestone generation rule.
    Generation,
    /// An observation followed a terminal result.
    Terminal,
}

/// Bounded reducer for one exact interactive evidence sequence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteractiveEvidenceV1 {
    milestones: Vec<InteractiveMilestoneV1>,
    last_generation: Option<u64>,
    result: Option<InteractiveResultV1>,
}

impl InteractiveEvidenceV1 {
    /// Creates an empty nonterminal evidence sequence.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            milestones: Vec::new(),
            last_generation: None,
            result: None,
        }
    }

    /// Applies one observation atomically.
    #[must_use = "interactive evidence failures must be handled"]
    pub fn observe(
        &mut self,
        observation: InteractiveObservationV1,
    ) -> Result<(), InteractiveEvidenceErrorKindV1> {
        if self.result.is_some() {
            return Err(InteractiveEvidenceErrorKindV1::Terminal);
        }
        let expected = self.next_required();
        if observation == InteractiveObservationV1::Close
            && expected != Some(InteractiveMilestoneV1::Close)
        {
            self.result = Some(InteractiveResultV1::Stop);
            return Ok(());
        }
        if expected != Some(observation.milestone()) {
            return Err(InteractiveEvidenceErrorKindV1::Order);
        }
        let next_generation = validate_generation(self.last_generation, observation)?;
        self.milestones.push(observation.milestone());
        self.last_generation = next_generation;
        if observation == InteractiveObservationV1::Close {
            self.result = Some(InteractiveResultV1::Pass);
        }
        Ok(())
    }

    /// Returns the terminal result when one has been formed.
    #[must_use]
    pub const fn result(&self) -> Option<InteractiveResultV1> {
        self.result
    }

    /// Returns the first milestone not yet completed.
    #[must_use]
    pub fn next_required(&self) -> Option<InteractiveMilestoneV1> {
        InteractiveMilestoneV1::ALL
            .get(self.milestones.len())
            .copied()
    }

    /// Returns the exact accepted milestone prefix.
    #[must_use]
    pub fn milestones(&self) -> &[InteractiveMilestoneV1] {
        &self.milestones
    }
}

impl Default for InteractiveEvidenceV1 {
    fn default() -> Self {
        Self::new()
    }
}

fn validate_generation(
    previous: Option<u64>,
    observation: InteractiveObservationV1,
) -> Result<Option<u64>, InteractiveEvidenceErrorKindV1> {
    match observation {
        InteractiveObservationV1::InitialPresent { generation } if previous.is_none() => {
            Ok(Some(generation))
        }
        InteractiveObservationV1::MutationPresent { generation }
        | InteractiveObservationV1::ResizePresent { generation }
            if previous.is_some_and(|previous| generation > previous) =>
        {
            Ok(Some(generation))
        }
        InteractiveObservationV1::RestorePresent { generation } if previous == Some(generation) => {
            Ok(previous)
        }
        InteractiveObservationV1::InitialPresent { .. }
        | InteractiveObservationV1::MutationPresent { .. }
        | InteractiveObservationV1::ResizePresent { .. }
        | InteractiveObservationV1::RestorePresent { .. } => {
            Err(InteractiveEvidenceErrorKindV1::Generation)
        }
        _ => Ok(previous),
    }
}
