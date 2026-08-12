pub(super) enum BoundedAsciiWriterError {
    LineBytes,
    ArtifactBytes,
    InvalidOutput,
}

pub(super) struct BoundedAsciiWriter {
    output: String,
    line_bytes: usize,
    artifact_bytes: usize,
}

impl BoundedAsciiWriter {
    pub(super) const fn new(line_bytes: usize, artifact_bytes: usize) -> Self {
        Self {
            output: String::new(),
            line_bytes,
            artifact_bytes,
        }
    }

    pub(super) fn push_line(&mut self, line: &str) -> Result<(), BoundedAsciiWriterError> {
        if line.len() > self.line_bytes {
            return Err(BoundedAsciiWriterError::LineBytes);
        }
        let bytes = self
            .output
            .len()
            .checked_add(line.len())
            .and_then(|value| value.checked_add(1))
            .ok_or(BoundedAsciiWriterError::InvalidOutput)?;
        if bytes > self.artifact_bytes {
            return Err(BoundedAsciiWriterError::ArtifactBytes);
        }
        if !line.bytes().all(|byte| (0x20..=0x7e).contains(&byte)) {
            return Err(BoundedAsciiWriterError::InvalidOutput);
        }
        self.output.push_str(line);
        self.output.push('\n');
        Ok(())
    }

    pub(super) fn finish(self) -> String {
        self.output
    }
}
