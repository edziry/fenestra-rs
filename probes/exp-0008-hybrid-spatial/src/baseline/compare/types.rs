use super::super::EvidenceSectionV2;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EvidenceMismatchV2 {
    pub(crate) case: usize,
    pub(crate) step: usize,
    pub(crate) section: EvidenceSectionV2,
    pub(crate) record: usize,
    pub(crate) field: String,
}

impl EvidenceMismatchV2 {
    pub(crate) fn new(
        case: usize,
        step: usize,
        section: EvidenceSectionV2,
        record: usize,
        field: impl Into<String>,
    ) -> Self {
        Self {
            case,
            step,
            section,
            record,
            field: field.into(),
        }
    }
}
