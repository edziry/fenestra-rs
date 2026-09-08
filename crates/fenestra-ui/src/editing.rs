use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

mod error;
mod selection;

pub use error::EditingError;
pub use selection::Selection;

/// Owned editable UTF-8 text with an inclusive byte limit.
///
/// Selection endpoints always lie on extended grapheme boundaries. Movement
/// follows logical string order, independent of shaping, bidirectional layout,
/// lines, fonts, and native input. Text is retained without normalization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextBuffer {
    text: String,
    selection: Selection,
    max_bytes: usize,
}

impl TextBuffer {
    /// Owns the supplied text and places the caret at its end.
    ///
    /// Rejects text whose UTF-8 byte length exceeds `max_bytes`.
    pub fn new(text: impl Into<String>, max_bytes: usize) -> Result<Self, EditingError> {
        let text = text.into();
        if text.len() > max_bytes {
            return Err(EditingError::LimitExceeded {
                limit: max_bytes,
                actual: text.len(),
            });
        }
        Ok(Self {
            selection: Selection::caret(text.len()),
            text,
            max_bytes,
        })
    }

    /// Returns the complete text, without exposing mutable storage.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the current selection in UTF-8 byte offsets.
    #[must_use]
    pub const fn selection(&self) -> Selection {
        self.selection
    }

    /// Returns the inclusive content byte limit.
    #[must_use]
    pub const fn max_bytes(&self) -> usize {
        self.max_bytes
    }

    /// Returns selected text in logical string order.
    #[must_use]
    pub fn selected_text(&self) -> &str {
        &self.text[self.selection.range()]
    }

    /// Sets the selection after validating both endpoints.
    ///
    /// Out-of-range offsets, UTF-8 interiors, and extended grapheme interiors
    /// are rejected without changing either the text or the selection.
    pub fn set_selection(&mut self, selection: Selection) -> Result<(), EditingError> {
        self.validate_offset(selection.anchor())?;
        self.validate_offset(selection.focus())?;
        self.selection = selection;
        Ok(())
    }

    /// Replaces the selected range, or inserts at the caret.
    ///
    /// If the edit joins neighboring graphemes, the caret advances to the first
    /// boundary at or after the inserted text's end. Exceeding the byte limit
    /// leaves both the text and the selection unchanged.
    pub fn replace_selection(&mut self, replacement: &str) -> Result<(), EditingError> {
        let range = self.selection.range();
        let actual = (self.text.len() - range.len())
            .checked_add(replacement.len())
            .ok_or(EditingError::CapacityOverflow)?;
        if actual > self.max_bytes {
            return Err(EditingError::LimitExceeded {
                limit: self.max_bytes,
                actual,
            });
        }
        self.replace_range(range, replacement);
        Ok(())
    }

    /// Deletes the selection, or the preceding extended grapheme.
    ///
    /// At the start of an empty selection this does nothing. A newly joined
    /// grapheme uses the same caret rule as [`Self::replace_selection`].
    pub fn backspace(&mut self) {
        let range = if self.selection.is_caret() {
            self.previous_boundary(self.selection.focus())..self.selection.focus()
        } else {
            self.selection.range()
        };
        self.replace_range(range, "");
    }

    /// Deletes the selection, or the following extended grapheme.
    ///
    /// At the end of an empty selection this does nothing. A newly joined
    /// grapheme uses the same caret rule as [`Self::replace_selection`].
    pub fn delete_forward(&mut self) {
        let range = if self.selection.is_caret() {
            self.selection.focus()..self.next_boundary(self.selection.focus())
        } else {
            self.selection.range()
        };
        self.replace_range(range, "");
    }

    /// Moves to the preceding grapheme in logical string order.
    ///
    /// When `extend` is true, only the focus moves. Otherwise, a nonempty
    /// selection collapses to its lower byte offset without an extra step.
    /// This does not implement visual leftward movement for bidirectional text.
    pub fn move_left(&mut self, extend: bool) {
        let offset = if !extend && !self.selection.is_caret() {
            self.selection.range().start
        } else {
            self.previous_boundary(self.selection.focus())
        };
        self.move_focus(offset, extend);
    }

    /// Moves to the following grapheme in logical string order.
    ///
    /// When `extend` is true, only the focus moves. Otherwise, a nonempty
    /// selection collapses to its higher byte offset without an extra step.
    /// This does not implement visual rightward movement for bidirectional text.
    pub fn move_right(&mut self, extend: bool) {
        let offset = if !extend && !self.selection.is_caret() {
            self.selection.range().end
        } else {
            self.next_boundary(self.selection.focus())
        };
        self.move_focus(offset, extend);
    }

    /// Moves to the start of the entire buffer, preserving the anchor if extended.
    pub fn move_to_start(&mut self, extend: bool) {
        self.move_focus(0, extend);
    }

    /// Moves to the end of the entire buffer, preserving the anchor if extended.
    pub fn move_to_end(&mut self, extend: bool) {
        self.move_focus(self.text.len(), extend);
    }

    /// Selects all text with the anchor at zero and the focus at the end.
    pub fn select_all(&mut self) {
        self.selection = Selection::new(0, self.text.len());
    }

    fn validate_offset(&self, offset: usize) -> Result<(), EditingError> {
        if offset > self.text.len() {
            return Err(EditingError::OffsetOutOfBounds {
                offset,
                len: self.text.len(),
            });
        }
        if offset != self.text.len() && !self.text.grapheme_indices(true).any(|(i, _)| i == offset)
        {
            return Err(EditingError::InvalidGraphemeBoundary { offset });
        }
        Ok(())
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.text
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .take_while(|&i| i < offset)
            .last()
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.text
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .find(|&i| i > offset)
            .unwrap_or(self.text.len())
    }

    fn move_focus(&mut self, offset: usize, extend: bool) {
        self.selection = if extend {
            Selection::new(self.selection.anchor(), offset)
        } else {
            Selection::caret(offset)
        };
    }

    fn replace_range(&mut self, range: Range<usize>, replacement: &str) {
        let inserted_end = range.start + replacement.len();
        self.text.replace_range(range, replacement);
        // Edits can change segmentation on either side, including distant
        // regional indicator pairs, so inspect the complete resulting string.
        let caret = self
            .text
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .find(|&i| i >= inserted_end)
            .unwrap_or(self.text.len());
        self.selection = Selection::caret(caret);
    }
}
