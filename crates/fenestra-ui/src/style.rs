use fenestra_ui_ir::prototype::{InputPolicy, PropertyId, PropertyValue};

use crate::Error;
use crate::lower::{BACKGROUND, GAP, HEIGHT, INPUT, PADDING, WIDTH};
use crate::model::ElementKind;

/// An RGBA color with eight bits per channel and straight alpha.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Color(pub(crate) [u8; 4]);

impl Color {
    /// Creates a color. Alpha zero is transparent and 255 is opaque.
    #[must_use]
    pub const fn rgba8(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self([red, green, blue, alpha])
    }

    /// Returns the red, green, blue, and alpha bytes.
    #[must_use]
    pub const fn to_rgba8(self) -> [u8; 4] {
        self.0
    }
}

/// Fixed viewport dimensions, container spacing, background, and input policy.
///
/// Dimensions and spacing must be nonnegative. Children may extend outside a
/// container's dimensions; the viewport clips rendered output and input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Style {
    pub(crate) width: i32,
    pub(crate) height: i32,
    pub(crate) padding: i32,
    pub(crate) gap: i32,
    pub(crate) background: Color,
    pub(crate) input: bool,
}

impl Style {
    /// Creates a transparent 64 by 64 viewport pixel element with no input.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            width: 64,
            height: 64,
            padding: 0,
            gap: 0,
            background: Color::rgba8(0, 0, 0, 0),
            input: false,
        }
    }

    /// Sets the fixed width in viewport pixels.
    #[must_use]
    pub const fn width(mut self, width: i32) -> Self {
        self.width = width;
        self
    }

    /// Sets the fixed height in viewport pixels.
    #[must_use]
    pub const fn height(mut self, height: i32) -> Self {
        self.height = height;
        self
    }

    /// Sets uniform container padding in viewport pixels.
    #[must_use]
    pub const fn padding(mut self, padding: i32) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the gap between adjacent children in viewport pixels.
    #[must_use]
    pub const fn gap(mut self, gap: i32) -> Self {
        self.gap = gap;
        self
    }

    /// Sets the color painted over the element's bounds.
    #[must_use]
    pub const fn background(mut self, background: Color) -> Self {
        self.background = background;
        self
    }

    /// Enables or disables hit testing over the element's bounds.
    #[must_use]
    pub const fn input(mut self, input: bool) -> Self {
        self.input = input;
        self
    }

    pub(crate) fn validate(self, node: &str, kind: ElementKind) -> Result<(), Error> {
        for (property, value) in [
            ("width", self.width),
            ("height", self.height),
            ("padding", self.padding),
            ("gap", self.gap),
        ] {
            if value < 0 {
                return Err(Error::InvalidStyle {
                    node: node.into(),
                    property,
                    value,
                });
            }
        }
        if kind == ElementKind::Rect && (self.padding != 0 || self.gap != 0) {
            return Err(Error::InvalidElement {
                node: node.into(),
                reason: "rectangles cannot use padding or gap",
            });
        }
        Ok(())
    }

    pub(crate) fn values(self) -> [(PropertyId, PropertyValue); 6] {
        let input = if self.input {
            InputPolicy::Accept
        } else {
            InputPolicy::Ignore
        };
        [
            (WIDTH, PropertyValue::ScalarI32(self.width)),
            (HEIGHT, PropertyValue::ScalarI32(self.height)),
            (PADDING, PropertyValue::ScalarI32(self.padding)),
            (GAP, PropertyValue::ScalarI32(self.gap)),
            (BACKGROUND, PropertyValue::Rgba8(self.background.0)),
            (INPUT, PropertyValue::InputPolicy(input)),
        ]
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::new()
    }
}
