use crate::{Style, TextStyle};

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

/// A rectangle, text leaf, or ordered row or column of child elements.
///
/// Names use ASCII letters, digits, and underscores, cannot begin with a digit,
/// and must be unique across the view. Child order determines layout and paint
/// order. Each element has fixed viewport pixel dimensions; containers do not resize
/// themselves to fit their children.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Element {
    pub(crate) name: String,
    pub(crate) kind: ElementKind,
    pub(crate) style: Style,
    pub(crate) children: Vec<Element>,
    pub(crate) text: Option<String>,
    pub(crate) text_style: Option<TextStyle>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ElementKind {
    Row,
    Column,
    Rect,
    Text,
}

impl Element {
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

    /// Creates a text leaf. Its fixed-size raster wraps and clips its content.
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
        }
    }
}
