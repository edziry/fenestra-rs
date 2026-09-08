use crate::control::ControlData;
use crate::lower::FlatView;
use crate::{ControlRole, ControlState, Error, Style};

use super::{
    NamedNode, TextState,
    interaction::{Interaction, Target},
};

mod api;
mod dispatch;

pub(super) fn state(nodes: &[NamedNode], index: usize, interaction: &Interaction) -> ControlState {
    let control = nodes[index].control.as_ref().expect("known control owner");
    interaction.state(
        index,
        control.disabled,
        (control.role == ControlRole::Checkbox).then_some(control.checked),
    )
}

pub(super) fn initial_styles(flat: &FlatView<'_>, texts: &mut [Option<TextState>]) -> Vec<Style> {
    flat.nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let element = node.element;
            let state = node
                .control_owner
                .map(|owner| {
                    let control = flat.nodes[owner].element;
                    ControlState {
                        disabled: control.disabled.unwrap_or(false),
                        checked: (control.kind.control_role() == Some(ControlRole::Checkbox))
                            .then_some(control.checked.unwrap_or(false)),
                        ..ControlState::default()
                    }
                })
                .unwrap_or_default();
            if let Some(text) = &mut texts[index] {
                text.effective_style = text.style.color(
                    element
                        .state_style
                        .resolve_color(text.style.color_value(), state),
                );
            }
            element
                .style
                .background(
                    element
                        .state_style
                        .resolve_background(element.style.background, state),
                )
                .input(element.kind.control_role().is_some() || element.style.input)
        })
        .collect()
}

pub(super) fn prepare(nodes: &mut [NamedNode], interaction: &mut Interaction) {
    let targets = nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            node.control.as_ref().map(|control| Target {
                index,
                role: control.role,
                enabled: !control.disabled,
                focusable: !control.disabled,
            })
        })
        .collect::<Vec<_>>();
    interaction.reconcile_disabled(&targets);
    for index in 0..nodes.len() {
        let state = nodes[index]
            .control_owner
            .map(|owner| state(nodes, owner, interaction))
            .unwrap_or_default();
        let node = &mut nodes[index];
        node.effective_style = node
            .style
            .background(
                node.state_style
                    .resolve_background(node.style.background, state),
            )
            .input(node.control.is_some() || node.style.input);
        if let Some(text) = &mut node.text {
            let effective = text.style.color(
                node.state_style
                    .resolve_color(text.style.color_value(), state),
            );
            if text.effective_style != effective {
                // State variants only change paint color. Keep measurements
                // based on authored typography so hover cannot change hit geometry.
                text.layout = None;
                text.effective_style = effective;
            }
        }
    }
}

pub(super) fn data(node: &NamedNode) -> Result<&ControlData, Error> {
    node.control.as_ref().ok_or_else(|| Error::InvalidElement {
        node: node.name.clone(),
        reason: "operation requires a control",
    })
}
