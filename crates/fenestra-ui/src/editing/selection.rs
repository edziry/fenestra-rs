use std::ops::Range;

/// Directional selection endpoints expressed as UTF-8 byte offsets.
///
/// The anchor stays fixed when extending a selection; the focus moves.
/// Construction is independent of text. [`super::TextBuffer::set_selection`]
/// validates both endpoints against the buffer's extended grapheme boundaries.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Selection {
    anchor: usize,
    focus: usize,
}

impl Selection {
    /// Creates directional endpoints to be validated by a text buffer.
    #[must_use]
    pub const fn new(anchor: usize, focus: usize) -> Self {
        Self { anchor, focus }
    }

    /// Creates an empty selection at the supplied byte offset.
    #[must_use]
    pub const fn caret(offset: usize) -> Self {
        Self::new(offset, offset)
    }

    /// Returns the fixed byte offset used when extending a selection.
    #[must_use]
    pub const fn anchor(self) -> usize {
        self.anchor
    }

    /// Returns the moving byte offset.
    #[must_use]
    pub const fn focus(self) -> usize {
        self.focus
    }

    /// Returns endpoints ordered for slicing, independent of direction.
    #[must_use]
    pub fn range(self) -> Range<usize> {
        self.anchor.min(self.focus)..self.anchor.max(self.focus)
    }

    /// Returns whether both endpoints identify the same caret position.
    #[must_use]
    pub const fn is_caret(self) -> bool {
        self.anchor == self.focus
    }
}
