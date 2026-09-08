use crate::{Color, ControlState};

/// Optional control-state colors without an implicit color theme.
///
/// Background overrides apply in checked, hovered, pressed, then disabled
/// order. Text colors apply checked then disabled. An absent override preserves
/// the previous color. Focus decoration is independent of both color sequences.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StateStyle {
    pub(crate) hover_background: Option<Color>,
    pub(crate) pressed_background: Option<Color>,
    pub(crate) checked_background: Option<Color>,
    pub(crate) disabled_background: Option<Color>,
    pub(crate) checked_color: Option<Color>,
    pub(crate) disabled_color: Option<Color>,
    pub(crate) focus_color: Option<Color>,
}

impl StateStyle {
    /// Creates a state style with no color overrides.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            hover_background: None,
            pressed_background: None,
            checked_background: None,
            disabled_background: None,
            checked_color: None,
            disabled_color: None,
            focus_color: None,
        }
    }

    /// Sets the background while the control is hovered.
    #[must_use]
    pub const fn hover_background(mut self, color: Color) -> Self {
        self.hover_background = Some(color);
        self
    }

    /// Sets the background while the control is pressed.
    #[must_use]
    pub const fn pressed_background(mut self, color: Color) -> Self {
        self.pressed_background = Some(color);
        self
    }

    /// Sets the background while the owning checkbox is checked.
    #[must_use]
    pub const fn checked_background(mut self, color: Color) -> Self {
        self.checked_background = Some(color);
        self
    }

    /// Sets the background while the control is disabled.
    #[must_use]
    pub const fn disabled_background(mut self, color: Color) -> Self {
        self.disabled_background = Some(color);
        self
    }

    /// Sets the text color while the owning checkbox is checked.
    #[must_use]
    pub const fn checked_color(mut self, color: Color) -> Self {
        self.checked_color = Some(color);
        self
    }

    /// Sets the text color while the control is disabled.
    #[must_use]
    pub const fn disabled_color(mut self, color: Color) -> Self {
        self.disabled_color = Some(color);
        self
    }

    /// Sets the owning control's focus decoration color.
    #[must_use]
    pub const fn focus_color(mut self, color: Color) -> Self {
        self.focus_color = Some(color);
        self
    }

    pub(crate) fn resolve_background(self, base: Color, state: ControlState) -> Color {
        let mut color = base;
        for (active, value) in [
            (state.checked == Some(true), self.checked_background),
            (state.hovered, self.hover_background),
            (state.pressed, self.pressed_background),
            (state.disabled, self.disabled_background),
        ] {
            if active && let Some(value) = value {
                color = value;
            }
        }
        color
    }

    pub(crate) fn resolve_color(self, base: Color, state: ControlState) -> Color {
        let mut color = base;
        for (active, value) in [
            (state.checked == Some(true), self.checked_color),
            (state.disabled, self.disabled_color),
        ] {
            if active && let Some(value) = value {
                color = value;
            }
        }
        color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: Color = Color::rgba8(10, 20, 30, 255);
    const CHECKED: Color = Color::rgba8(40, 50, 60, 255);
    const HOVERED: Color = Color::rgba8(70, 80, 90, 255);
    const PRESSED: Color = Color::rgba8(100, 110, 120, 255);
    const DISABLED: Color = Color::rgba8(130, 140, 150, 128);
    const FOCUS: Color = Color::rgba8(160, 170, 180, 255);

    fn state(bits: u8) -> ControlState {
        ControlState {
            checked: Some(bits & 1 != 0),
            hovered: bits & 2 != 0,
            pressed: bits & 4 != 0,
            disabled: bits & 8 != 0,
            focused: bits & 16 != 0,
        }
    }

    #[test]
    fn unset_state_colors_preserve_base_and_supply_no_implicit_theme() {
        const STYLE: StateStyle = StateStyle::new();
        assert_eq!(STYLE, StateStyle::default());
        for bits in 0..32 {
            assert_eq!(STYLE.resolve_background(BASE, state(bits)), BASE);
            assert_eq!(STYLE.resolve_color(BASE, state(bits)), BASE);
        }
        assert_eq!(STYLE.focus_color, None);
    }

    #[test]
    fn background_precedence_is_checked_hovered_pressed_disabled() {
        const STYLE: StateStyle = StateStyle::new()
            .checked_background(CHECKED)
            .hover_background(HOVERED)
            .pressed_background(PRESSED)
            .disabled_background(DISABLED)
            .focus_color(FOCUS);
        for bits in 0..32 {
            let expected = match bits & 15 {
                8..=15 => DISABLED,
                4..=7 => PRESSED,
                2..=3 => HOVERED,
                1 => CHECKED,
                _ => BASE,
            };
            assert_eq!(STYLE.resolve_background(BASE, state(bits)), expected);
        }
        assert_eq!(STYLE.focus_color, Some(FOCUS));
    }

    #[test]
    fn absent_higher_priority_overrides_keep_the_last_supplied_color() {
        let style = StateStyle::new()
            .checked_background(CHECKED)
            .hover_background(HOVERED);
        assert_eq!(style.resolve_background(BASE, state(15)), HOVERED);
        assert_eq!(style.resolve_background(BASE, state(13)), CHECKED);
        let unchecked = ControlState {
            checked: None,
            ..state(0)
        };
        assert_eq!(style.resolve_background(BASE, unchecked), BASE);
    }

    #[test]
    fn text_color_uses_checked_then_disabled_and_focus_stays_separate() {
        const STYLE: StateStyle = StateStyle::new()
            .checked_color(CHECKED)
            .disabled_color(DISABLED)
            .focus_color(FOCUS);
        for bits in 0..32 {
            let expected = if bits & 8 != 0 {
                DISABLED
            } else if bits & 1 != 0 {
                CHECKED
            } else {
                BASE
            };
            assert_eq!(STYLE.resolve_color(BASE, state(bits)), expected);
        }
        let style = StateStyle::new().checked_color(CHECKED);
        assert_eq!(style.resolve_color(BASE, state(31)), CHECKED);
    }
}
