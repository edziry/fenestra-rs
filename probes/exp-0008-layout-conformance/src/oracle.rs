use std::fmt;

use fenestra_ui_layout::prototype::LayoutRecordV1;

/// First unequal record property in comparison priority order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutRecordMismatchKindV1 {
    /// Record cardinality differs.
    Count,
    /// Pass-local keys differ at one ordinal.
    Key,
    /// Horizontal origins differ at one ordinal.
    X,
    /// Vertical origins differ at one ordinal.
    Y,
    /// Widths differ at one ordinal.
    Width,
    /// Heights differ at one ordinal.
    Height,
}

impl LayoutRecordMismatchKindV1 {
    /// Every mismatch kind in deterministic comparison order.
    pub const ALL: [Self; 6] = [
        Self::Count,
        Self::Key,
        Self::X,
        Self::Y,
        Self::Width,
        Self::Height,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::Count => "count",
            Self::Key => "key",
            Self::X => "x",
            Self::Y => "y",
            Self::Width => "width",
            Self::Height => "height",
        }
    }
}

/// First privacy-safe difference between two ordered record slices.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct LayoutRecordMismatchV1 {
    kind: LayoutRecordMismatchKindV1,
    ordinal: usize,
}

impl LayoutRecordMismatchV1 {
    const fn new(kind: LayoutRecordMismatchKindV1, ordinal: usize) -> Self {
        Self { kind, ordinal }
    }

    /// Returns the first unequal property.
    #[must_use]
    pub const fn kind(self) -> LayoutRecordMismatchKindV1 {
        self.kind
    }

    /// Returns the first affected zero-based record ordinal.
    #[must_use]
    pub const fn ordinal(self) -> usize {
        self.ordinal
    }
}

impl fmt::Display for LayoutRecordMismatchV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "layout-record-mismatch({}@{})",
            self.kind.label(),
            self.ordinal
        )
    }
}

impl fmt::Debug for LayoutRecordMismatchV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "LayoutRecordMismatchV1({self})")
    }
}

/// Returns the first record mismatch using count then record-major priority.
#[must_use]
pub fn compare_layout_records_v1(
    expected: &[LayoutRecordV1],
    observed: &[LayoutRecordV1],
) -> Option<LayoutRecordMismatchV1> {
    if expected.len() != observed.len() {
        return Some(LayoutRecordMismatchV1::new(
            LayoutRecordMismatchKindV1::Count,
            expected.len().min(observed.len()),
        ));
    }

    for (ordinal, (expected, observed)) in expected.iter().zip(observed).enumerate() {
        let expected_bounds = expected.bounds();
        let observed_bounds = observed.bounds();
        let kind = if expected.key() != observed.key() {
            Some(LayoutRecordMismatchKindV1::Key)
        } else if expected_bounds.x() != observed_bounds.x() {
            Some(LayoutRecordMismatchKindV1::X)
        } else if expected_bounds.y() != observed_bounds.y() {
            Some(LayoutRecordMismatchKindV1::Y)
        } else if expected_bounds.width() != observed_bounds.width() {
            Some(LayoutRecordMismatchKindV1::Width)
        } else if expected_bounds.height() != observed_bounds.height() {
            Some(LayoutRecordMismatchKindV1::Height)
        } else {
            None
        };

        if let Some(kind) = kind {
            return Some(LayoutRecordMismatchV1::new(kind, ordinal));
        }
    }

    None
}
