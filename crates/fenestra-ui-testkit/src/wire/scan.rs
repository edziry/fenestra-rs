use super::error::{ArtifactDecodeError, ArtifactDecodeErrorKind, ArtifactLimitKind};

pub(crate) const ARTIFACT_BYTES_LIMIT: usize = 524_288;
pub(crate) const LINE_BYTES_LIMIT: usize = 1_024;
pub(crate) const LINES_LIMIT: usize = 4_096;

pub(crate) struct ScannedLine<'a> {
    pub(crate) number: u32,
    pub(crate) text: &'a str,
}

pub(crate) fn scan_lines(bytes: &[u8]) -> Result<Vec<ScannedLine<'_>>, ArtifactDecodeError> {
    validate_ascii(bytes)?;

    let mut starts = Vec::new();
    let mut start = 0_usize;
    let mut terminated_lines = 0_usize;
    let mut first_overlong = None;
    for (index, byte) in bytes.iter().copied().enumerate() {
        if byte != b'\n' {
            continue;
        }
        terminated_lines = terminated_lines.checked_add(1).ok_or_else(lines_exceeded)?;
        let number = terminated_lines;
        let line_bytes = index.checked_sub(start).ok_or_else(lines_exceeded)?;
        if first_overlong.is_none() && line_bytes > LINE_BYTES_LIMIT {
            first_overlong = Some(line_number(number));
        }
        if number <= LINES_LIMIT {
            starts.push((start, index));
        }
        start = index.checked_add(1).ok_or_else(lines_exceeded)?;
    }

    if start < bytes.len() {
        let number = terminated_lines.checked_add(1).ok_or_else(lines_exceeded)?;
        let line_bytes = bytes.len().checked_sub(start).ok_or_else(lines_exceeded)?;
        if first_overlong.is_none() && line_bytes > LINE_BYTES_LIMIT {
            first_overlong = Some(line_number(number));
        }
    }
    if let Some(line) = first_overlong {
        return Err(ArtifactDecodeError::at(
            ArtifactDecodeErrorKind::LimitExceeded(ArtifactLimitKind::LineBytes),
            line,
        ));
    }

    let total_lines = terminated_lines
        .checked_add(usize::from(start < bytes.len()))
        .ok_or_else(lines_exceeded)?;
    if total_lines > LINES_LIMIT {
        return Err(ArtifactDecodeError::at(
            ArtifactDecodeErrorKind::LimitExceeded(ArtifactLimitKind::Lines),
            line_number(LINES_LIMIT + 1),
        ));
    }
    if bytes.is_empty() || start != bytes.len() {
        return Err(ArtifactDecodeError::at(
            ArtifactDecodeErrorKind::MalformedRecord,
            line_number(total_lines.max(1)),
        ));
    }

    let mut lines = Vec::with_capacity(starts.len());
    for (index, (line_start, line_end)) in starts.into_iter().enumerate() {
        let number = line_number(index + 1);
        let text = std::str::from_utf8(&bytes[line_start..line_end])
            .map_err(|_| ArtifactDecodeError::at(ArtifactDecodeErrorKind::InvalidAscii, number))?;
        lines.push(ScannedLine { number, text });
    }
    Ok(lines)
}

fn validate_ascii(bytes: &[u8]) -> Result<(), ArtifactDecodeError> {
    let mut line = 1_usize;
    for byte in bytes {
        match *byte {
            b'\n' => line = line.checked_add(1).ok_or_else(lines_exceeded)?,
            0x20..=0x7e => {}
            _ => {
                return Err(ArtifactDecodeError::at(
                    ArtifactDecodeErrorKind::InvalidAscii,
                    line_number(line),
                ));
            }
        }
    }
    Ok(())
}

fn lines_exceeded() -> ArtifactDecodeError {
    ArtifactDecodeError::new(
        ArtifactDecodeErrorKind::LimitExceeded(ArtifactLimitKind::Lines),
        None,
    )
}

fn line_number(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
