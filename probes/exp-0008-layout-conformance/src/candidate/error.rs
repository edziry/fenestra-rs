use fenestra_ui_layout::prototype::{
    LayoutConstraintFieldV1, LayoutEngineErrorKindV1, LayoutEngineErrorV1, LayoutErrorLocationV1,
    LayoutExtentV1, LayoutOutputFieldV1, LayoutPaddingSideV1,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CandidateProfileErrorKindV1 {
    CoordinateLimit,
    NonFiniteOutput,
    NegativeOutput,
    OutputEdgeLimit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CandidateEdgeV1 {
    Near,
    Far,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CandidateProfileErrorFieldV1 {
    Viewport(LayoutExtentV1),
    Constraint {
        extent: LayoutExtentV1,
        field: LayoutConstraintFieldV1,
    },
    Padding(LayoutPaddingSideV1),
    Gap,
    Output(LayoutOutputFieldV1),
    OutputEdge {
        extent: LayoutExtentV1,
        edge: CandidateEdgeV1,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CandidateProfileErrorV1 {
    kind: CandidateProfileErrorKindV1,
    field: CandidateProfileErrorFieldV1,
    location: LayoutErrorLocationV1,
}

impl CandidateProfileErrorV1 {
    pub(crate) const fn new(
        kind: CandidateProfileErrorKindV1,
        field: CandidateProfileErrorFieldV1,
        location: LayoutErrorLocationV1,
    ) -> Self {
        Self {
            kind,
            field,
            location,
        }
    }

    #[cfg(test)]
    pub(crate) const fn kind(self) -> CandidateProfileErrorKindV1 {
        self.kind
    }

    #[cfg(test)]
    pub(crate) const fn field(self) -> CandidateProfileErrorFieldV1 {
        self.field
    }

    #[cfg(test)]
    pub(crate) const fn location(self) -> LayoutErrorLocationV1 {
        self.location
    }
}

pub(crate) const fn map_candidate_profile_error_v1(
    error: CandidateProfileErrorV1,
) -> LayoutEngineErrorV1 {
    let kind = match error.kind {
        CandidateProfileErrorKindV1::CoordinateLimit => LayoutEngineErrorKindV1::RejectedInput,
        CandidateProfileErrorKindV1::NonFiniteOutput
        | CandidateProfileErrorKindV1::NegativeOutput
        | CandidateProfileErrorKindV1::OutputEdgeLimit => {
            LayoutEngineErrorKindV1::UnrepresentableOutput
        }
    };
    LayoutEngineErrorV1::new(kind, error.location)
}

pub(crate) const fn invariant_error_v1(location: LayoutErrorLocationV1) -> LayoutEngineErrorV1 {
    LayoutEngineErrorV1::new(LayoutEngineErrorKindV1::InvariantViolation, location)
}
