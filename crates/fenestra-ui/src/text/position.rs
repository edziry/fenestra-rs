use crate::Selection;

/// The logical side to which a visual caret attaches at a text boundary.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextAffinity {
    /// Attach to the logically following text, including the next wrapped line.
    #[default]
    Downstream,
    /// Attach to the logically preceding text, including the previous line.
    Upstream,
}

/// A UTF-8 byte offset and its visual attachment direction.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TextPosition {
    byte: usize,
    affinity: TextAffinity,
}

impl TextPosition {
    /// Describes a position; a geometry request validates its grapheme boundary.
    #[must_use]
    pub const fn new(byte: usize, affinity: TextAffinity) -> Self {
        Self { byte, affinity }
    }
    /// Returns the offset in the original, unnormalized UTF-8 text.
    #[must_use]
    pub const fn byte(self) -> usize {
        self.byte
    }
    /// Returns the logical side that determines the visual caret location.
    #[must_use]
    pub const fn affinity(self) -> TextAffinity {
        self.affinity
    }
}

/// Directed selection endpoints with independent visual affinities.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TextSelection {
    anchor: TextPosition,
    focus: TextPosition,
}

impl TextSelection {
    pub(super) fn validate(self, text: &str) -> Result<(), super::TextError> {
        for byte in [self.anchor.byte, self.focus.byte] {
            crate::editing::validate_offset(text, byte)
                .map_err(|_| super::TextError::InvalidTextPosition { byte })?;
        }
        Ok(())
    }
    /// Describes endpoints to be validated against a geometry request's text.
    #[must_use]
    pub const fn new(anchor: TextPosition, focus: TextPosition) -> Self {
        Self { anchor, focus }
    }
    /// Describes a collapsed selection at one visual position.
    #[must_use]
    pub const fn caret(position: TextPosition) -> Self {
        Self::new(position, position)
    }
    /// Returns the fixed endpoint used when extending a selection.
    #[must_use]
    pub const fn anchor(self) -> TextPosition {
        self.anchor
    }
    /// Returns the moving endpoint.
    #[must_use]
    pub const fn focus(self) -> TextPosition {
        self.focus
    }
    /// Returns byte endpoints suitable for an owned text buffer.
    #[must_use]
    pub const fn byte_selection(self) -> Selection {
        Selection::new(self.anchor.byte, self.focus.byte)
    }
}
