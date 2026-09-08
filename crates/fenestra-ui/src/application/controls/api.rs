use std::sync::Arc;

use super::super::{Application, interaction::Target};
use super::{data, state};
use crate::{ControlId, ControlRole, ControlSnapshot, Error, Size, StateStyle};

impl Application {
    /// Returns the logical focus target, retained while the native window is inactive.
    #[must_use]
    pub fn focused_control(&self) -> Option<&str> {
        self.interaction
            .focus
            .map(|index| self.nodes[index].name.as_str())
    }

    /// Focuses an enabled, nonempty control intersecting the current viewport.
    ///
    /// Passing `None` clears logical focus. Explicit mutations do not dispatch
    /// callbacks; their result is immediately observable through snapshots.
    pub fn focus(&mut self, name: Option<&str>) -> Result<(), Error> {
        let index = name.map(|name| self.node_index(name)).transpose()?;
        if let Some(index) = index {
            data(&self.nodes[index])?;
            if !self
                .control_targets()?
                .iter()
                .any(|target| target.index == index && target.focusable)
            {
                return Err(Error::InvalidElement {
                    node: self.nodes[index].name.clone(),
                    reason: "focus requires an enabled control intersecting the viewport",
                });
            }
        }
        if self.interaction.focus == index {
            return Ok(());
        }
        let mut interaction = self.interaction.clone();
        interaction.set_focus(index);
        self.commit_with_state(self.nodes.clone(), self.size, interaction)
    }

    /// Returns one owned role, label, state and committed geometry record.
    pub fn control_snapshot(&self, name: &str) -> Result<ControlSnapshot, Error> {
        let index = self.node_index(name)?;
        let control = data(&self.nodes[index])?;
        Ok(ControlSnapshot {
            id: ControlId(u32::try_from(index + 1).map_err(|_| Error::CapacityOverflow)?),
            name: name.into(),
            label: control.label.to_string(),
            role: control.role,
            bounds: self.bounds(name)?,
            state: state(&self.nodes, index, &self.interaction),
        })
    }

    /// Returns control semantics in authored order without any platform types.
    pub fn control_snapshots(&self) -> Result<Vec<ControlSnapshot>, Error> {
        self.nodes
            .iter()
            .filter(|node| node.control.is_some())
            .map(|node| self.control_snapshot(&node.name))
            .collect()
    }

    /// Enables or disables a control, including its derived paint and focus state.
    pub fn set_disabled(&mut self, name: &str, disabled: bool) -> Result<(), Error> {
        let index = self.node_index(name)?;
        if data(&self.nodes[index])?.disabled == disabled {
            return Ok(());
        }
        let mut nodes = self.nodes.clone();
        nodes[index]
            .control
            .as_mut()
            .expect("checked control")
            .disabled = disabled;
        self.commit_state(nodes, self.size)
    }

    /// Publishes a checkbox value and its descendant state styles atomically.
    pub fn set_checked(&mut self, name: &str, checked: bool) -> Result<(), Error> {
        let index = self.node_index(name)?;
        let control = data(&self.nodes[index])?;
        if control.role != ControlRole::Checkbox {
            return Err(Error::InvalidElement {
                node: name.into(),
                reason: "checked requires a checkbox",
            });
        }
        if control.checked == checked {
            return Ok(());
        }
        let mut nodes = self.nodes.clone();
        nodes[index]
            .control
            .as_mut()
            .expect("checked control")
            .checked = checked;
        self.commit_state(nodes, self.size)
    }

    /// Returns the authored state styles, separately from resolved paint colors.
    pub fn state_style(&self, name: &str) -> Result<StateStyle, Error> {
        Ok(self.nodes[self.node_index(name)?].state_style)
    }

    /// Replaces state styles after checking their semantic and text context.
    pub fn set_state_style(&mut self, name: &str, style: StateStyle) -> Result<(), Error> {
        let index = self.node_index(name)?;
        let node = &self.nodes[index];
        let role = node.control_owner.and_then(|owner| {
            self.nodes[owner]
                .control
                .as_ref()
                .map(|control| control.role)
        });
        crate::lower::control::validate_state_style(name, node.kind, role, style)?;
        if node.state_style == style {
            return Ok(());
        }
        let mut nodes = self.nodes.clone();
        nodes[index].state_style = style;
        self.commit_state(nodes, self.size)
    }

    /// Changes a semantic label without replacing visible authored child content.
    pub fn set_control_label(&mut self, name: &str, label: impl AsRef<str>) -> Result<(), Error> {
        let label = label.as_ref();
        let index = self.node_index(name)?;
        if label.trim().is_empty() {
            return Err(Error::InvalidElement {
                node: name.into(),
                reason: "controls require a nonempty semantic label",
            });
        }
        if data(&self.nodes[index])?.label.as_ref() == label {
            return Ok(());
        }
        crate::text::validate_budget(
            self.nodes.iter().enumerate().flat_map(|(slot, node)| {
                [
                    node.text.as_ref().map(|text| text.content.as_ref()),
                    node.control.as_ref().map(|control| {
                        if slot == index {
                            label
                        } else {
                            control.label.as_ref()
                        }
                    }),
                ]
                .into_iter()
                .flatten()
                .map(|text| (text, Size::new(0, 0)))
            }),
            self.limits.text(),
        )?;
        let mut nodes = self.nodes.clone();
        nodes[index]
            .control
            .as_mut()
            .expect("checked control")
            .label = Arc::from(label);
        self.commit_state(nodes, self.size)
    }

    pub(in crate::application) fn control_targets(&self) -> Result<Vec<Target>, Error> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(index, node)| {
                node.control.as_ref().map(|control| {
                    let bounds = self.bounds(&node.name)?;
                    let intersects = bounds.width() > 0
                        && bounds.height() > 0
                        && bounds.x() < i64::from(self.size.width())
                        && bounds.y() < i64::from(self.size.height())
                        && bounds.x() + i64::from(bounds.width()) > 0
                        && bounds.y() + i64::from(bounds.height()) > 0;
                    Ok(Target {
                        index,
                        role: control.role,
                        enabled: !control.disabled,
                        focusable: !control.disabled && intersects,
                    })
                })
            })
            .collect()
    }
}
