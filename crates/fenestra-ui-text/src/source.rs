use std::borrow::Cow;

/// CRLF is one hard break for shaping; public positions remain raw UTF-8 bytes.
pub(crate) struct Source<'a> {
    text: Cow<'a, str>,
    breaks: Vec<(usize, usize)>,
}

impl<'a> Source<'a> {
    pub fn new(raw: &'a str) -> Self {
        let mut breaks = Vec::new();
        for (raw_offset, _) in raw.match_indices("\r\n") {
            breaks.push((raw_offset, raw_offset - breaks.len()));
        }
        let text = if breaks.is_empty() {
            Cow::Borrowed(raw)
        } else {
            Cow::Owned(raw.replace("\r\n", "\n"))
        };
        Self { text, breaks }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn to_layout(&self, byte: usize) -> usize {
        byte - self.breaks.partition_point(|&(raw, _)| raw < byte)
    }

    pub fn to_source(&self, byte: usize) -> usize {
        byte + self.breaks.partition_point(|&(_, shaped)| shaped < byte)
    }
}

#[cfg(test)]
mod tests {
    use super::Source;

    #[test]
    fn crlf_projection_round_trips_raw_editing_boundaries_without_normalizing_other_text() {
        let raw = "a\r\nb\n\rc\r\n";
        let source = Source::new(raw);
        assert_eq!(source.text(), "a\nb\n\rc\n");
        for byte in [0, 1, 3, 4, 5, 6, 7, 9] {
            assert_eq!(source.to_source(source.to_layout(byte)), byte);
        }
        assert_eq!(raw, "a\r\nb\n\rc\r\n");
    }
}
