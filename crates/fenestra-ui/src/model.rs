use crate::{Color, ControlRole, Dimension, StateStyle, Style, TextStyle};

/// A named, static tree of native UI elements.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct View {
    pub(crate) name: String,
    pub(crate) root: Element,
}

impl View {
    /// Creates a view. Names and styles are checked when building an application.
    #[must_use]
    pub fn new(name: impl Into<String>, root: Element) -> Self {
        Self {
            name: name.into(),
            root,
        }
    }

    /// Returns the name identifying the view.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the root element.
    #[must_use]
    pub const fn root(&self) -> &Element {
        &self.root
    }
}

/// A rectangle, text leaf, ordered container, button, or checkbox.
///
/// Names use ASCII letters, digits, and underscores, cannot begin with a digit,
/// and must be unique across the view. Child order determines layout and paint
/// order. Dimensions use fixed pixels, intrinsic content size, or weighted
/// available space, as selected by [`Style`] and [`crate::Dimension`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Element {
    pub(crate) name: String,
    pub(crate) kind: ElementKind,
    pub(crate) style: Style,
    pub(crate) children: Vec<Element>,
    pub(crate) text: Option<String>,
    pub(crate) text_style: Option<TextStyle>,
    pub(crate) label: Option<String>,
    pub(crate) disabled: Option<bool>,
    pub(crate) checked: Option<bool>,
    pub(crate) state_style: StateStyle,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ElementKind {
    Row,
    Column,
    Rect,
    Text,
    Button,
    Checkbox,
}

impl ElementKind {
    pub(crate) const fn control_role(self) -> Option<ControlRole> {
        match self {
            Self::Button => Some(ControlRole::Button),
            Self::Checkbox => Some(ControlRole::Checkbox),
            _ => None,
        }
    }
}

impl Element {
    /// Creates a focusable command container with an explicit semantic label.
    ///
    /// Visible content is authored with children. A button lays out its children
    /// as a column and rejects nested controls or input-enabled descendants.
    #[must_use]
    pub fn button(name: impl Into<String>, label: impl Into<String>) -> Self {
        let mut element = Self::new(name, ElementKind::Button);
        element.label = Some(label.into());
        element.style = Style::new()
            .width_mode(Dimension::Auto)
            .height_mode(Dimension::Auto)
            .padding(12)
            .gap(8)
            .background(Color::rgba8(48, 128, 192, 255));
        element
    }

    /// Creates a two-state control whose visible children form a row.
    ///
    /// Use state styles on children to show its checked value. The label is
    /// semantic metadata; no hidden label or indicator nodes are inserted.
    #[must_use]
    pub fn checkbox(name: impl Into<String>, label: impl Into<String>) -> Self {
        let mut element = Self::new(name, ElementKind::Checkbox);
        element.label = Some(label.into());
        element.style = Style::new()
            .width_mode(Dimension::Auto)
            .height_mode(Dimension::Auto)
            .padding(8)
            .gap(8);
        element
    }

    /// Sets initial disabled state on a button or checkbox.
    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = Some(disabled);
        self
    }

    /// Sets the initial checkbox value; other element kinds reject this setter.
    #[must_use]
    pub const fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    /// Selects paint variations from the closest enclosing control's state.
    #[must_use]
    pub const fn state_style(mut self, style: StateStyle) -> Self {
        self.state_style = style;
        self
    }

    /// Creates a container that places children from left to right.
    #[must_use]
    pub fn row(name: impl Into<String>) -> Self {
        Self::new(name, ElementKind::Row)
    }

    /// Creates a container that places children from top to bottom.
    #[must_use]
    pub fn column(name: impl Into<String>) -> Self {
        Self::new(name, ElementKind::Column)
    }

    /// Creates a rectangle. Rectangles cannot contain children or use padding or gap.
    #[must_use]
    pub fn rect(name: impl Into<String>) -> Self {
        Self::new(name, ElementKind::Rect)
    }

    /// Creates a text leaf that wraps and clips content to its resolved dimensions.
    #[must_use]
    pub fn text(name: impl Into<String>, content: impl Into<String>) -> Self {
        let mut element = Self::new(name, ElementKind::Text);
        element.text = Some(content.into());
        element
    }

    /// Sets typography on a text leaf; other element kinds reject this property.
    #[must_use]
    pub const fn text_style(mut self, style: TextStyle) -> Self {
        self.text_style = Some(style);
        self
    }

    /// Replaces the element's style.
    #[must_use]
    pub const fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Appends a child in layout and paint order.
    #[must_use]
    pub fn child(mut self, child: Self) -> Self {
        self.children.push(child);
        self
    }

    /// Returns the name used to address this element in an application.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    fn new(name: impl Into<String>, kind: ElementKind) -> Self {
        Self {
            name: name.into(),
            kind,
            style: Style::new(),
            children: Vec::new(),
            text: None,
            text_style: None,
            label: None,
            disabled: None,
            checked: None,
            state_style: StateStyle::new(),
        }
    }
}
