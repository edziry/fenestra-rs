use std::fmt;

use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::window::Window;

use crate::Bounds;

/// Desired input-method ownership for one native window.
///
/// Construct an explicit disabled context or an active editor with a checked
/// physical caret area. The editor identity is opaque and window-scoped;
/// every `u64`, including zero, is valid. When it changes, the host disables
/// and re-enables IME before supplying the new area.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImeContext(Option<ActiveContext>);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ActiveContext {
    editor_id: u64,
    caret: Bounds,
}

impl ImeContext {
    /// Disables IME regardless of the window's static opt-in policy.
    #[must_use]
    pub const fn disabled() -> Self {
        Self(None)
    }

    /// Activates an editor at a nonempty physical caret rectangle.
    ///
    /// Negative positions are permitted. Both extents and the exclusive right
    /// and bottom edges must fit the signed 32-bit platform coordinate range.
    pub fn active(
        editor_id: u64,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> Result<Self, ImeContextError> {
        if width == 0 || height == 0 {
            return Err(ImeContextError::EmptyCaret);
        }
        if width > i32::MAX as u32
            || height > i32::MAX as u32
            || i64::from(x) + i64::from(width) > i64::from(i32::MAX)
            || i64::from(y) + i64::from(height) > i64::from(i32::MAX)
        {
            return Err(ImeContextError::CaretOutOfRange);
        }
        Ok(Self(Some(ActiveContext {
            editor_id,
            caret: Bounds {
                x: i64::from(x),
                y: i64::from(y),
                width,
                height,
            },
        })))
    }

    /// Returns the active editor's opaque identity, or `None` when disabled.
    #[must_use]
    pub const fn editor_id(self) -> Option<u64> {
        match self.0 {
            Some(active) => Some(active.editor_id),
            None => None,
        }
    }

    /// Returns the checked physical caret area, or `None` when disabled.
    #[must_use]
    pub const fn caret(self) -> Option<Bounds> {
        match self.0 {
            Some(active) => Some(active.caret),
            None => None,
        }
    }
}

/// Rejection of a native input-method caret rectangle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImeContextError {
    /// A caret rectangle must have nonzero width and height.
    EmptyCaret,
    /// An extent or exclusive edge exceeds signed platform coordinates.
    CaretOutOfRange,
}

impl fmt::Display for ImeContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::EmptyCaret => "IME caret must have nonzero width and height",
            Self::CaretOutOfRange => "IME caret exceeds signed platform coordinates",
        })
    }
}

impl std::error::Error for ImeContextError {}

pub(super) trait ImeSink {
    fn allowed(&mut self, allowed: bool);
    fn area(&mut self, caret: Bounds);
}

pub(super) struct WindowImeSink<'a>(pub(super) &'a Window);

impl ImeSink for WindowImeSink<'_> {
    fn allowed(&mut self, allowed: bool) {
        self.0.set_ime_allowed(allowed);
    }

    fn area(&mut self, caret: Bounds) {
        self.0.set_ime_cursor_area(
            PhysicalPosition::new(caret.x() as i32, caret.y() as i32),
            PhysicalSize::new(caret.width(), caret.height()),
        );
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ImeBridge {
    legacy_allowed: bool,
    desired: Option<ImeContext>,
    applied: Option<EffectiveContext>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EffectiveContext {
    Disabled,
    LegacyEnabled,
    Active(ActiveContext),
}

impl ImeBridge {
    pub(super) fn new(legacy_allowed: bool) -> Self {
        Self {
            legacy_allowed,
            desired: None,
            applied: None,
        }
    }

    pub(super) fn refresh<E>(
        &mut self,
        desired: Result<Option<ImeContext>, E>,
        drawable: bool,
        focused: bool,
        force_area: bool,
        sink: &mut impl ImeSink,
    ) -> Result<(), E> {
        let desired = desired?;
        let next = match desired {
            None if self.legacy_allowed => EffectiveContext::LegacyEnabled,
            Some(ImeContext(Some(active))) if drawable && focused => {
                EffectiveContext::Active(active)
            }
            _ => EffectiveContext::Disabled,
        };
        let enabled = next != EffectiveContext::Disabled;
        let previous_enabled = self
            .applied
            .map(|value| value != EffectiveContext::Disabled);
        let transfer = previous_enabled == Some(true)
            && enabled
            && self.desired.and_then(ImeContext::editor_id)
                != desired.and_then(ImeContext::editor_id);
        if transfer {
            sink.allowed(false);
        }
        if previous_enabled != Some(enabled) || transfer {
            sink.allowed(enabled);
        }
        if let EffectiveContext::Active(active) = next
            && (self.applied != Some(next) || force_area)
        {
            // Wayland discards cursor-area updates while IME is disabled.
            sink.area(active.caret);
        }
        self.desired = desired;
        self.applied = Some(next);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
