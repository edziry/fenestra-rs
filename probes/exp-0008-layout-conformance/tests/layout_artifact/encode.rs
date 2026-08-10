use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum LayoutArtifactLimitKindV1 {
    Records,
    LineBytes,
    ArtifactBytes,
}

impl LayoutArtifactLimitKindV1 {
    pub(super) const ALL: [Self; 3] = [Self::Records, Self::LineBytes, Self::ArtifactBytes];

    const fn label(self) -> &'static str {
        match self {
            Self::Records => "records",
            Self::LineBytes => "line-bytes",
            Self::ArtifactBytes => "artifact-bytes",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum LayoutArtifactErrorKindV1 {
    InvalidRecord,
    LimitExceeded(LayoutArtifactLimitKindV1),
}

impl LayoutArtifactErrorKindV1 {
    pub(super) const ALL: [Self; 4] = [
        Self::InvalidRecord,
        Self::LimitExceeded(LayoutArtifactLimitKindV1::Records),
        Self::LimitExceeded(LayoutArtifactLimitKindV1::LineBytes),
        Self::LimitExceeded(LayoutArtifactLimitKindV1::ArtifactBytes),
    ];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct LayoutArtifactLimitsV1 {
    values: [usize; 3],
}

impl LayoutArtifactLimitsV1 {
    pub(super) const fn new(records: usize, line_bytes: usize, artifact_bytes: usize) -> Self {
        Self {
            values: [records, line_bytes, artifact_bytes],
        }
    }

    pub(super) const fn limit(self, kind: LayoutArtifactLimitKindV1) -> usize {
        self.values[kind as usize]
    }
}

pub(super) const REGISTERED_LAYOUT_ARTIFACT_LIMITS_V1: LayoutArtifactLimitsV1 =
    LayoutArtifactLimitsV1::new(512, 512, 65_536);

pub(super) struct LayoutArtifactErrorV1 {
    kind: LayoutArtifactErrorKindV1,
    observed: Option<usize>,
    maximum: Option<usize>,
}

impl LayoutArtifactErrorV1 {
    const fn invalid_record() -> Self {
        Self {
            kind: LayoutArtifactErrorKindV1::InvalidRecord,
            observed: None,
            maximum: None,
        }
    }

    const fn limit(kind: LayoutArtifactLimitKindV1, observed: usize, maximum: usize) -> Self {
        Self {
            kind: LayoutArtifactErrorKindV1::LimitExceeded(kind),
            observed: Some(observed),
            maximum: Some(maximum),
        }
    }

    pub(super) const fn kind(&self) -> LayoutArtifactErrorKindV1 {
        self.kind
    }

    pub(super) const fn observed(&self) -> Option<usize> {
        self.observed
    }

    pub(super) const fn maximum(&self) -> Option<usize> {
        self.maximum
    }
}

impl fmt::Display for LayoutArtifactErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.kind, self.observed, self.maximum) {
            (LayoutArtifactErrorKindV1::InvalidRecord, None, None) => {
                formatter.write_str("layout-artifact-invalid-record")
            }
            (LayoutArtifactErrorKindV1::LimitExceeded(kind), Some(observed), Some(maximum)) => {
                write!(
                    formatter,
                    "layout-artifact-limit-exceeded({};observed={observed};maximum={maximum})",
                    kind.label()
                )
            }
            _ => formatter.write_str("layout-artifact-error"),
        }
    }
}

impl fmt::Debug for LayoutArtifactErrorV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "LayoutArtifactErrorV1({self})")
    }
}

impl Error for LayoutArtifactErrorV1 {}

pub(super) struct LayoutArtifactV1 {
    source: Box<str>,
}

impl LayoutArtifactV1 {
    pub(super) fn as_str(&self) -> &str {
        &self.source
    }

    pub(super) fn as_bytes(&self) -> &[u8] {
        self.source.as_bytes()
    }
}

impl fmt::Debug for LayoutArtifactV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LayoutArtifactV1")
            .field("bytes", &self.source.len())
            .finish()
    }
}

pub(super) fn encode_layout_artifact_v1(
    lines: &[String],
    limits: LayoutArtifactLimitsV1,
) -> Result<LayoutArtifactV1, LayoutArtifactErrorV1> {
    let records_limit = limits.limit(LayoutArtifactLimitKindV1::Records);
    if lines.len() > records_limit {
        return Err(LayoutArtifactErrorV1::limit(
            LayoutArtifactLimitKindV1::Records,
            lines.len(),
            records_limit,
        ));
    }
    if lines.is_empty() {
        return Err(LayoutArtifactErrorV1::invalid_record());
    }

    let line_bytes_limit = limits.limit(LayoutArtifactLimitKindV1::LineBytes);
    let mut artifact_bytes = Some(0usize);
    for line in lines {
        if line.is_empty() || !line.bytes().all(|byte| (0x20..=0x7e).contains(&byte)) {
            return Err(LayoutArtifactErrorV1::invalid_record());
        }
        if line.len() > line_bytes_limit {
            return Err(LayoutArtifactErrorV1::limit(
                LayoutArtifactLimitKindV1::LineBytes,
                line.len(),
                line_bytes_limit,
            ));
        }
        let encoded_line_bytes = line.len().checked_add(1);
        artifact_bytes = artifact_bytes.and_then(|bytes| {
            encoded_line_bytes.and_then(|line_bytes| bytes.checked_add(line_bytes))
        });
    }

    let artifact_bytes_limit = limits.limit(LayoutArtifactLimitKindV1::ArtifactBytes);
    let artifact_bytes = artifact_bytes.ok_or_else(|| artifact_bytes_error(limits))?;
    if artifact_bytes > artifact_bytes_limit {
        return Err(LayoutArtifactErrorV1::limit(
            LayoutArtifactLimitKindV1::ArtifactBytes,
            artifact_bytes,
            artifact_bytes_limit,
        ));
    }

    let mut source = String::with_capacity(artifact_bytes);
    for line in lines {
        source.push_str(line);
        source.push('\n');
    }
    Ok(LayoutArtifactV1 {
        source: source.into_boxed_str(),
    })
}

fn artifact_bytes_error(limits: LayoutArtifactLimitsV1) -> LayoutArtifactErrorV1 {
    LayoutArtifactErrorV1::limit(
        LayoutArtifactLimitKindV1::ArtifactBytes,
        usize::MAX,
        limits.limit(LayoutArtifactLimitKindV1::ArtifactBytes),
    )
}
