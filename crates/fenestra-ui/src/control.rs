use std::sync::Arc;

use crate::Bounds;

/// The interaction and semantic role of a drawn control.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlRole {
    /// A command activated by pointer release, Enter, or Space.
    Button,
    /// A two-state choice toggled by pointer release or Space.
    Checkbox,
}

/// A stable identifier within one application's static control tree.
///
/// Identifiers are not interchangeable between applications.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ControlId(pub(crate) u32);

impl ControlId {
    /// Returns the application-local numeric identifier.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// The accepted interaction state used by paint and semantic snapshots.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ControlState {
    pub(crate) disabled: bool,
    pub(crate) checked: Option<bool>,
    pub(crate) focused: bool,
    pub(crate) hovered: bool,
    pub(crate) pressed: bool,
}

impl ControlState {
    /// Whether the control rejects focus and activation.
    #[must_use]
    pub const fn disabled(self) -> bool {
        self.disabled
    }
    /// The checkbox value, or `None` for a button.
    #[must_use]
    pub const fn checked(self) -> Option<bool> {
        self.checked
    }
    /// Whether the control has focus in the active window.
    #[must_use]
    pub const fn focused(self) -> bool {
        self.focused
    }
    /// Whether the pointer is over this enabled control.
    #[must_use]
    pub const fn hovered(self) -> bool {
        self.hovered
    }
    /// Whether a matching pointer or activation key is held.
    #[must_use]
    pub const fn pressed(self) -> bool {
        self.pressed
    }
}

/// An owned semantic record from the same accepted state as control painting.
///
/// This record is a platform-independent boundary, not a native accessibility
/// adapter or proof of assistive-technology support.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ControlSnapshot {
    pub(crate) id: ControlId,
    pub(crate) name: String,
    pub(crate) label: String,
    pub(crate) role: ControlRole,
    pub(crate) bounds: Bounds,
    pub(crate) state: ControlState,
}

impl ControlSnapshot {
    /// Returns the stable application-local identifier.
    #[must_use]
    pub const fn id(&self) -> ControlId {
        self.id
    }
    /// Returns the authored name used by events and mutations.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Returns the semantic label independently of visible child content.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }
    /// Returns the interaction role.
    #[must_use]
    pub const fn role(&self) -> ControlRole {
        self.role
    }
    /// Returns committed world bounds, before viewport clipping.
    #[must_use]
    pub const fn bounds(&self) -> Bounds {
        self.bounds
    }
    /// Returns the accepted interaction state.
    #[must_use]
    pub const fn state(&self) -> ControlState {
        self.state
    }
}

#[derive(Clone)]
pub(crate) struct ControlData {
    pub(crate) role: ControlRole,
    pub(crate) label: Arc<str>,
    pub(crate) disabled: bool,
    pub(crate) checked: bool,
}
