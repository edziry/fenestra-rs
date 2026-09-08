use super::super::Application;
use super::state;
use crate::{ControlRole, Error, Event, InputEvent};

impl Application {
    /// Processes owned input and returns notifications after accepted publication.
    ///
    /// The native host uses this same path. Headless callers can pass its events
    /// to their ordinary application handler. `Click` remains a press notification;
    /// semantic controls activate through `Activated` or `CheckedChanged` instead.
    pub fn dispatch_input(&mut self, input: InputEvent) -> Result<Vec<Event>, Error> {
        let pointer = match input {
            InputEvent::PointerMoved { x, y } => Some((x, y)),
            _ => self.interaction.pointer,
        };
        let target = pointer
            .and_then(|(x, y)| self.hit_test(x, y))
            .map(str::to_owned);
        let hit = target
            .as_deref()
            .map(|name| self.node_index(name))
            .transpose()?
            .and_then(|index| self.nodes[index].control_owner);
        let transition = self
            .interaction
            .reduce(&input, &self.control_targets()?, hit);
        let old_focus = self.interaction.focus;
        let mut nodes = self.nodes.clone();
        let action = transition.activate.map(|index| {
            let node = &mut nodes[index];
            let control = node
                .control
                .as_mut()
                .expect("reducer only activates controls");
            if control.role == ControlRole::Checkbox {
                control.checked = !control.checked;
                Event::CheckedChanged {
                    target: node.name.clone(),
                    checked: control.checked,
                }
            } else {
                Event::Activated {
                    target: node.name.clone(),
                }
            }
        });
        let changed = old_focus != transition.next.focus
            || action
                .as_ref()
                .is_some_and(|event| matches!(event, Event::CheckedChanged { .. }))
            || nodes.iter().enumerate().any(|(index, node)| {
                node.control.is_some()
                    && state(&self.nodes, index, &self.interaction)
                        != state(&nodes, index, &transition.next)
            });
        if changed {
            self.commit_with_state(nodes, self.size, transition.next)?;
        } else {
            // Cursor coordinates and held-key bookkeeping can change without
            // changing any accepted control state or requesting a new generation.
            self.interaction = transition.next;
        }
        let raw = match input {
            InputEvent::PointerMoved { x, y } => Event::PointerMoved { x, y },
            InputEvent::PointerPressed => Event::Click { target },
            InputEvent::PointerReleased => Event::PointerReleased { target },
            InputEvent::PointerLeft => Event::PointerLeft,
            InputEvent::SpacePressed => Event::SpacePressed,
            InputEvent::KeyboardInput(input) => Event::KeyboardInput(input),
            InputEvent::ModifiersChanged(modifiers) => Event::ModifiersChanged(modifiers),
            InputEvent::Focused(focused) => Event::Focused(focused),
            InputEvent::Ime(ime) => Event::Ime(ime),
            InputEvent::CloseRequested => Event::CloseRequested,
        };
        let mut events = vec![raw];
        if old_focus != self.interaction.focus {
            events.push(Event::FocusChanged {
                target: self.focused_control().map(str::to_owned),
            });
        }
        events.extend(action);
        Ok(events)
    }
}
