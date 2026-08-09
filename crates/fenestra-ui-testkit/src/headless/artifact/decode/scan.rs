use super::super::error::{
    HeadlessArtifactDecodeErrorKindV1 as ErrorKind, HeadlessArtifactDecodeErrorV1,
    HeadlessArtifactLimitKindV1 as LimitKind,
};

const ARTIFACT_BYTES_LIMIT: usize = 65_536;
const LINE_BYTES_LIMIT: usize = 1_024;
const LINES_LIMIT: usize = 512;

pub(super) struct ScannedLineV1<'source> {
    pub(super) number: u32,
    pub(super) text: &'source str,
}

pub(super) struct ScannedArtifactV1<'source> {
    lines: Vec<ScannedLineV1<'source>>,
    artifact_bytes: usize,
    max_line_bytes: usize,
}

impl<'source> ScannedArtifactV1<'source> {
    pub(super) fn lines(&self) -> &[ScannedLineV1<'source>] {
        &self.lines
    }

    pub(super) const fn artifact_bytes(&self) -> usize {
        self.artifact_bytes
    }

    pub(super) const fn max_line_bytes(&self) -> usize {
        self.max_line_bytes
    }
}

pub(super) fn scan_artifact_v1(
    bytes: &[u8],
) -> Result<ScannedArtifactV1<'_>, HeadlessArtifactDecodeErrorV1> {
    if bytes.len() > ARTIFACT_BYTES_LIMIT {
        return Err(limit_exceeded(LimitKind::ArtifactBytes, None));
    }
    validate_ascii(bytes)?;
    let metrics = measure_lines(bytes)?;
    if let Some(line) = metrics.first_overlong {
        return Err(limit_exceeded(LimitKind::LineBytes, Some(line)));
    }
    if metrics.line_count > LINES_LIMIT {
        return Err(limit_exceeded(
            LimitKind::Lines,
            Some(line_number(LINES_LIMIT + 1)),
        ));
    }
    if !bytes.ends_with(b"\n") {
        return Err(HeadlessArtifactDecodeErrorV1::at(
            ErrorKind::MissingFinalLineFeed,
            line_number(metrics.line_count.max(1)),
        ));
    }

    let mut lines = Vec::with_capacity(metrics.line_count);
    let mut start = 0_usize;
    for (index, byte) in bytes.iter().copied().enumerate() {
        if byte != b'\n' {
            continue;
        }
        let text = std::str::from_utf8(&bytes[start..index]).map_err(|_| {
            HeadlessArtifactDecodeErrorV1::at(ErrorKind::InvalidAscii, line_number(lines.len() + 1))
        })?;
        lines.push(ScannedLineV1 {
            number: line_number(lines.len() + 1),
            text,
        });
        start = index.checked_add(1).ok_or_else(lines_exhausted)?;
    }
    Ok(ScannedArtifactV1 {
        lines,
        artifact_bytes: bytes.len(),
        max_line_bytes: metrics.max_line_bytes,
    })
}

fn validate_ascii(bytes: &[u8]) -> Result<(), HeadlessArtifactDecodeErrorV1> {
    let mut line = 1_usize;
    for byte in bytes {
        match *byte {
            b'\n' => line = line.checked_add(1).ok_or_else(lines_exhausted)?,
            0x20..=0x7e => {}
            _ => {
                return Err(HeadlessArtifactDecodeErrorV1::at(
                    ErrorKind::InvalidAscii,
                    line_number(line),
                ));
            }
        }
    }
    Ok(())
}

fn measure_lines(bytes: &[u8]) -> Result<LineMetricsV1, HeadlessArtifactDecodeErrorV1> {
    let mut start = 0_usize;
    let mut terminated = 0_usize;
    let mut max_line_bytes = 0_usize;
    let mut first_overlong = None;
    for (index, byte) in bytes.iter().copied().enumerate() {
        if byte != b'\n' {
            continue;
        }
        terminated = terminated.checked_add(1).ok_or_else(lines_exhausted)?;
        let line_bytes = index.checked_sub(start).ok_or_else(lines_exhausted)?;
        max_line_bytes = max_line_bytes.max(line_bytes);
        if first_overlong.is_none() && line_bytes > LINE_BYTES_LIMIT {
            first_overlong = Some(line_number(terminated));
        }
        start = index.checked_add(1).ok_or_else(lines_exhausted)?;
    }
    let has_unterminated = start < bytes.len();
    if has_unterminated {
        let line_bytes = bytes.len().checked_sub(start).ok_or_else(lines_exhausted)?;
        max_line_bytes = max_line_bytes.max(line_bytes);
        if first_overlong.is_none() && line_bytes > LINE_BYTES_LIMIT {
            first_overlong = Some(line_number(terminated + 1));
        }
    }
    let line_count = terminated
        .checked_add(usize::from(has_unterminated))
        .ok_or_else(lines_exhausted)?;
    Ok(LineMetricsV1 {
        line_count,
        max_line_bytes,
        first_overlong,
    })
}

struct LineMetricsV1 {
    line_count: usize,
    max_line_bytes: usize,
    first_overlong: Option<u32>,
}

fn limit_exceeded(limit: LimitKind, line: Option<u32>) -> HeadlessArtifactDecodeErrorV1 {
    HeadlessArtifactDecodeErrorV1::new(ErrorKind::LimitExceeded(limit), line)
}

fn lines_exhausted() -> HeadlessArtifactDecodeErrorV1 {
    limit_exceeded(LimitKind::Lines, None)
}

fn line_number(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
