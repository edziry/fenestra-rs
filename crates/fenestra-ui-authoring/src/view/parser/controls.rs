use crate::source_v2::PhysicalOriginV2;

use super::{Diagnostic, Element, Kind, Parser};

impl Parser {
    pub(super) fn control_property(
        &self,
        element: &Element,
        property: &str,
        scope: Option<Kind>,
        physical: PhysicalOriginV2,
    ) -> Result<(), Diagnostic> {
        let message = match property {
            "label" | "disabled" if !element.kind.is_control() => {
                Some("property requires a control element")
            }
            "checked" if element.kind != Kind::Checkbox => {
                Some("checked requires a checkbox element")
            }
            "input" if element.kind.is_control() => {
                Some("control input is derived; omit the input property")
            }
            "hover_background"
            | "pressed_background"
            | "checked_background"
            | "disabled_background"
            | "checked_color"
            | "disabled_color"
            | "focus_color"
                if scope.is_none() =>
            {
                Some("state style property requires a control or its descendant")
            }
            "checked_color" if element.kind != Kind::Text || scope != Some(Kind::Checkbox) => {
                Some("checked_color requires a text element inside a checkbox control")
            }
            "disabled_color" if element.kind != Kind::Text => {
                Some("disabled_color requires a text element inside a control")
            }
            "checked_background" if scope != Some(Kind::Checkbox) => {
                Some("checked_background requires a checkbox control or its descendant")
            }
            "focus_color" if !element.kind.is_control() => {
                Some("focus_color requires a control element")
            }
            _ => None,
        };
        match message {
            Some(message) => Err(Diagnostic::new(message, physical)),
            None => Ok(()),
        }
    }

    pub(super) fn boolean(&mut self) -> Result<bool, Diagnostic> {
        let token = self.take()?;
        match token.label() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(Diagnostic::new("expected true or false", token.physical)),
        }
    }
}
