use crate::model::ElementKind;
use crate::{ControlRole, Element, Error, StateStyle};

pub(super) fn validate(element: &Element, inherited: Option<ControlRole>) -> Result<(), Error> {
    let role = element.kind.control_role();
    let reject = |reason| Error::InvalidElement {
        node: element.name.clone(),
        reason,
    };
    if role.is_some() && inherited.is_some() {
        return Err(reject("controls cannot contain other controls"));
    }
    if role.is_some()
        && element
            .label
            .as_deref()
            .is_none_or(|label| label.trim().is_empty())
    {
        return Err(reject("controls require a nonempty semantic label"));
    }
    if role.is_none() && element.disabled.is_some() {
        return Err(reject("disabled requires a control"));
    }
    if role != Some(ControlRole::Checkbox) && element.checked.is_some() {
        return Err(reject("checked requires a checkbox"));
    }
    let owner = role.or(inherited);
    if owner.is_some() && element.style.input {
        return Err(reject(
            "control input is derived; descendants cannot enable independent input",
        ));
    }
    validate_state_style(&element.name, element.kind, owner, element.state_style)
}

pub(crate) fn validate_state_style(
    name: &str,
    kind: ElementKind,
    owner: Option<ControlRole>,
    style: StateStyle,
) -> Result<(), Error> {
    let reject = |reason| Error::InvalidElement {
        node: name.into(),
        reason,
    };
    if owner.is_none() && style != StateStyle::new() {
        return Err(reject("state styles require an enclosing control"));
    }
    if (style.checked_background.is_some() || style.checked_color.is_some())
        && owner != Some(ControlRole::Checkbox)
    {
        return Err(reject("checked styles require an enclosing checkbox"));
    }
    if (style.checked_color.is_some() || style.disabled_color.is_some())
        && kind != ElementKind::Text
    {
        return Err(reject("state text colors require a text element"));
    }
    if style.focus_color.is_some() && kind.control_role().is_none() {
        return Err(reject("focus color requires a control"));
    }
    Ok(())
}
