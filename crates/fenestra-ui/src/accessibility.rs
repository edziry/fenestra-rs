use crate::{Bounds, ControlState, Size};

/// An application-local semantic identity; zero is reserved for the window.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AccessibilityId(u64);

impl AccessibilityId {
    /// Represents a semantic identity, including stale or unknown external targets.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the application-local identity.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// The supported semantic roles, independent of operating-system adapters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccessibilityRole {
    /// The synthetic native window root.
    Window,
    /// An ordered container of semantic children.
    Group,
    /// Read-only text outside a control's composed content.
    Label,
    /// A command with an explicit semantic label.
    Button,
    /// A two-state choice with an explicit semantic label.
    Checkbox,
}

/// A semantic request from an accessibility client, without forged device input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccessibilityAction {
    /// Move logical focus to an eligible control.
    Focus,
    /// Invoke a button or toggle a checkbox without moving logical focus.
    Activate,
}

/// An owned action targeting an application-local semantic identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccessibilityActionRequest {
    /// The target identity from an accepted accessibility tree.
    pub target: AccessibilityId,
    /// The requested semantic behavior.
    pub action: AccessibilityAction,
}

/// An immutable semantic node from an accepted application tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessibilityNode {
    pub(crate) id: AccessibilityId,
    pub(crate) role: AccessibilityRole,
    pub(crate) name: String,
    pub(crate) label: String,
    pub(crate) bounds: Bounds,
    pub(crate) children: Vec<AccessibilityId>,
    pub(crate) control_state: Option<ControlState>,
    pub(crate) focusable: bool,
}

impl AccessibilityNode {
    /// Returns the stable identity; control IDs have the same numeric value.
    #[must_use]
    pub const fn id(&self) -> AccessibilityId {
        self.id
    }
    /// Returns the semantic role.
    #[must_use]
    pub const fn role(&self) -> AccessibilityRole {
        self.role
    }
    /// Returns the authored element name, or an empty name for the window root.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Returns explicit control labeling or complete read-only text content.
    ///
    /// The native adapter replaces the synthetic window's empty label with its title.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }
    /// Returns unclipped bounds in physical viewport pixels.
    #[must_use]
    pub const fn bounds(&self) -> Bounds {
        self.bounds
    }
    /// Returns semantic child identities in authored order.
    #[must_use]
    pub fn children(&self) -> &[AccessibilityId] {
        &self.children
    }
    /// Returns control state, or `None` for windows, groups and labels.
    #[must_use]
    pub const fn control_state(&self) -> Option<ControlState> {
        self.control_state
    }
    /// Whether a control currently accepts focus and activation requests.
    #[must_use]
    pub const fn focusable(&self) -> bool {
        self.focusable
    }
}

/// An owned semantic tree from the same accepted state as geometry and paint.
///
/// Nodes retain authored identities and order. Composed control descendants
/// are represented by their owner's explicit label rather than duplicate nodes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessibilityTree {
    pub(crate) generation: u64,
    pub(crate) viewport: Size,
    pub(crate) focus: AccessibilityId,
    pub(crate) window_focused: bool,
    pub(crate) nodes: Vec<AccessibilityNode>,
}

impl AccessibilityTree {
    /// Returns the accepted application generation.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }
    /// Returns the physical viewport size.
    #[must_use]
    pub const fn viewport(&self) -> Size {
        self.viewport
    }
    /// Returns retained logical focus, or window identity zero when absent.
    #[must_use]
    pub const fn focus(&self) -> AccessibilityId {
        self.focus
    }
    /// Whether the application has received active window focus.
    #[must_use]
    pub const fn window_focused(&self) -> bool {
        self.window_focused
    }
    /// Returns the window root followed by included elements in authored preorder.
    #[must_use]
    pub fn nodes(&self) -> &[AccessibilityNode] {
        &self.nodes
    }
}
