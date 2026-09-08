use fenestra_ui_ir::prototype::{InputPolicy, PropertyId, PropertyValue};

use crate::lower::{BACKGROUND, GAP, HEIGHT, INPUT, PADDING, WIDTH};
use crate::model::ElementKind;
use crate::{Dimension, Error, Size};

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

/// Dimension policies, container spacing, background, and input policy.
///
/// Pixel preferences, limits, and spacing must be nonnegative. Minimum and
/// maximum limits apply to every dimension policy. Children may extend outside
/// a container's dimensions; the viewport clips rendered output and input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Style {
    pub(crate) width: Dimension,
    pub(crate) height: Dimension,
    pub(crate) min_width: i32,
    pub(crate) max_width: i32,
    pub(crate) min_height: i32,
    pub(crate) max_height: i32,
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
            width: Dimension::Px(64),
            height: Dimension::Px(64),
            min_width: 0,
            max_width: i32::MAX,
            min_height: 0,
            max_height: i32::MAX,
            padding: 0,
            gap: 0,
            background: Color::rgba8(0, 0, 0, 0),
            input: false,
        }
    }

    /// Sets a pixel width preference, clamped by the minimum and maximum width.
    #[must_use]
    pub const fn width(mut self, width: i32) -> Self {
        self.width = Dimension::Px(width);
        self
    }

    /// Sets a pixel height preference, clamped by the minimum and maximum height.
    #[must_use]
    pub const fn height(mut self, height: i32) -> Self {
        self.height = Dimension::Px(height);
        self
    }

    /// Sets the horizontal dimension policy.
    #[must_use]
    pub const fn width_mode(mut self, mode: Dimension) -> Self {
        self.width = mode;
        self
    }

    /// Sets the vertical dimension policy.
    #[must_use]
    pub const fn height_mode(mut self, mode: Dimension) -> Self {
        self.height = mode;
        self
    }

    /// Sets the inclusive nonnegative minimum width.
    #[must_use]
    pub const fn min_width(mut self, value: i32) -> Self {
        self.min_width = value;
        self
    }

    /// Sets the inclusive maximum width, which must not be below the minimum.
    #[must_use]
    pub const fn max_width(mut self, value: i32) -> Self {
        self.max_width = value;
        self
    }

    /// Sets the inclusive nonnegative minimum height.
    #[must_use]
    pub const fn min_height(mut self, value: i32) -> Self {
        self.min_height = value;
        self
    }

    /// Sets the inclusive maximum height, which must not be below the minimum.
    #[must_use]
    pub const fn max_height(mut self, value: i32) -> Self {
        self.max_height = value;
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
        for (property, dimension) in [("width", self.width), ("height", self.height)] {
            match dimension {
                Dimension::Px(value) if value < 0 => {
                    return Err(Error::InvalidStyle {
                        node: node.into(),
                        property,
                        value,
                    });
                }
                Dimension::Fill(weight) if !(1..=65_535).contains(&weight) => {
                    return Err(Error::InvalidElement {
                        node: node.into(),
                        reason: "fill weights must be between 1 and 65535",
                    });
                }
                _ => {}
            }
        }
        for (property, value) in [
            ("padding", self.padding),
            ("gap", self.gap),
            ("min_width", self.min_width),
            ("max_width", self.max_width),
            ("min_height", self.min_height),
            ("max_height", self.max_height),
        ] {
            if value < 0 {
                return Err(Error::InvalidStyle {
                    node: node.into(),
                    property,
                    value,
                });
            }
        }
        if self.min_width > self.max_width || self.min_height > self.max_height {
            return Err(Error::InvalidElement {
                node: node.into(),
                reason: "dimension minimum must not exceed its maximum",
            });
        }
        if matches!(kind, ElementKind::Rect | ElementKind::Text)
            && (self.padding != 0 || self.gap != 0)
        {
            return Err(Error::InvalidElement {
                node: node.into(),
                reason: "leaf elements cannot use padding or gap",
            });
        }
        Ok(())
    }

    pub(crate) fn values(self, size: Size) -> [(PropertyId, PropertyValue); 6] {
        let input = if self.input {
            InputPolicy::Accept
        } else {
            InputPolicy::Ignore
        };
        [
            (WIDTH, PropertyValue::ScalarI32(size.width() as i32)),
            (HEIGHT, PropertyValue::ScalarI32(size.height() as i32)),
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
