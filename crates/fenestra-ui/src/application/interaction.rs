use crate::{
    ControlRole, ControlState, ImeEvent, InputEvent, Key, KeyState, KeyboardInput, Modifiers,
};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Interaction {
    pub(super) focus: Option<usize>,
    pub(super) hover: Option<usize>,
    pub(super) pointer_arm: Option<usize>,
    pub(super) space_arm: Option<usize>,
    pub(super) enter_arm: Option<usize>,
    pub(super) pointer: Option<(i32, i32)>,
    pub(super) tab_down: bool,
    pub(super) window_focused: bool,
    pub(super) composing: bool,
    space_down: bool,
    enter_down: bool,
    pointer_down: bool,
}

impl Default for Interaction {
    fn default() -> Self {
        Self {
            focus: None,
            hover: None,
            pointer_arm: None,
            space_arm: None,
            enter_arm: None,
            pointer: None,
            tab_down: false,
            window_focused: true,
            composing: false,
            space_down: false,
            enter_down: false,
            pointer_down: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Target {
    pub(super) index: usize,
    pub(super) role: ControlRole,
    pub(super) enabled: bool,
    pub(super) focusable: bool,
}

pub(super) struct Transition {
    pub(super) next: Interaction,
    pub(super) activate: Option<usize>,
}

impl Interaction {
    pub(super) fn reduce(
        &self,
        input: &InputEvent,
        targets: &[Target],
        hit: Option<usize>,
    ) -> Transition {
        let mut next = self.clone();
        next.reconcile_disabled(targets);
        let mut activate = None;
        let eligible_hit = eligible(targets, hit).map(|target| target.index);
        match input {
            InputEvent::PointerMoved { x, y } if next.window_focused => {
                next.pointer = Some((*x, *y));
                next.hover = eligible_hit;
            }
            InputEvent::PointerPressed if next.window_focused => {
                next.hover = eligible_hit;
                if !std::mem::replace(&mut next.pointer_down, true) && !next.composing {
                    if let Some(index) = eligible_hit {
                        next.set_focus(Some(index));
                        next.pointer_arm = Some(index);
                    } else if target(targets, hit).is_none() {
                        next.set_focus(None);
                    }
                }
            }
            InputEvent::PointerReleased => {
                let arm = next.pointer_arm.take();
                next.pointer_down = false;
                if next.window_focused {
                    next.hover = eligible_hit;
                    if !next.composing && arm.is_some() && arm == eligible_hit {
                        activate = arm;
                    }
                }
            }
            InputEvent::PointerLeft => next.clear_pointer(),
            InputEvent::KeyboardInput(input) => activate = next.keyboard(input, targets),
            InputEvent::ModifiersChanged(modifiers) if shortcut(*modifiers) => {
                next.clear_key_arms()
            }
            InputEvent::Focused(focused) => {
                next.window_focused = *focused;
                if !focused {
                    next.clear_pointer();
                    next.clear_key_arms();
                    next.space_down = false;
                    next.enter_down = false;
                    next.tab_down = false;
                    next.composing = false;
                }
            }
            InputEvent::Ime(ImeEvent::Preedit { text, .. }) => {
                next.clear_key_arms();
                next.composing = next.window_focused && !text.is_empty();
            }
            InputEvent::Ime(_) => next.composing = false,
            _ => {}
        }
        Transition { next, activate }
    }

    pub(super) fn clear_arms(&mut self) {
        self.pointer_arm = None;
        self.clear_key_arms();
    }

    pub(super) fn set_focus(&mut self, focus: Option<usize>) {
        if self.focus != focus {
            self.focus = focus;
            self.clear_arms();
        }
    }

    pub(super) fn state(
        &self,
        index: usize,
        disabled: bool,
        checked: Option<bool>,
    ) -> ControlState {
        let visible = self.window_focused && !disabled;
        let focused = self.focus == Some(index);
        let hovered = self.hover == Some(index);
        let pointer_pressed = self.pointer_arm == Some(index) && hovered;
        let key_pressed =
            focused && (self.space_arm == Some(index) || self.enter_arm == Some(index));
        ControlState {
            disabled,
            checked,
            focused: visible && focused,
            hovered: visible && hovered,
            pressed: visible && !self.composing && (pointer_pressed || key_pressed),
        }
    }

    pub(super) fn reconcile_disabled(&mut self, targets: &[Target]) {
        let enabled = |index| target(targets, index).is_some_and(|target| target.enabled);
        if !enabled(self.focus) {
            self.set_focus(None);
        }
        self.hover = self.hover.filter(|index| enabled(Some(*index)));
        self.pointer_arm = self.pointer_arm.filter(|index| enabled(Some(*index)));
        self.space_arm = self.space_arm.filter(|index| enabled(Some(*index)));
        self.enter_arm = self.enter_arm.filter(|index| enabled(Some(*index)));
    }

    fn clear_pointer(&mut self) {
        self.pointer = None;
        self.hover = None;
        self.pointer_arm = None;
        self.pointer_down = false;
    }

    fn clear_key_arms(&mut self) {
        self.space_arm = None;
        self.enter_arm = None;
    }

    fn keyboard(&mut self, input: &KeyboardInput, targets: &[Target]) -> Option<usize> {
        let space =
            input.key == Key::Space || matches!(&input.key, Key::Character(text) if text == " ");
        if input.is_synthetic && input.state == KeyState::Pressed {
            return None;
        }
        if shortcut(input.modifiers) {
            self.clear_key_arms();
        }
        let accepts = self.window_focused
            && !self.composing
            && !shortcut(input.modifiers)
            && !input.is_synthetic
            && !input.repeat;
        if input.state == KeyState::Released {
            if space {
                self.space_down = false;
                let arm = self.space_arm.take();
                return arm.filter(|&index| {
                    accepts && self.focus == Some(index) && eligible(targets, Some(index)).is_some()
                });
            }
            if input.key == Key::Enter {
                self.enter_down = false;
                self.enter_arm = None;
            } else if input.key == Key::Tab {
                self.tab_down = false;
            }
            return None;
        }
        if input.repeat || !self.window_focused {
            return None;
        }
        if input.key == Key::Escape {
            self.clear_arms();
            return None;
        }
        let held = if space {
            std::mem::replace(&mut self.space_down, true)
        } else if input.key == Key::Enter {
            std::mem::replace(&mut self.enter_down, true)
        } else if input.key == Key::Tab {
            std::mem::replace(&mut self.tab_down, true)
        } else {
            return None;
        };
        // A cancelled or ineligible down still owns its hold until a release.
        if held || !accepts {
            return None;
        }
        if input.key == Key::Tab {
            self.cycle_focus(targets, input.modifiers.shift);
        } else if let Some(target) = eligible(targets, self.focus) {
            if space {
                self.space_arm = Some(target.index);
            } else if target.role == ControlRole::Button {
                self.enter_arm = Some(target.index);
                return self.enter_arm;
            }
        }
        None
    }

    fn cycle_focus(&mut self, targets: &[Target], reverse: bool) {
        let current = targets
            .iter()
            .position(|target| Some(target.index) == self.focus);
        let next = (1..=targets.len()).find_map(|offset| {
            let position = match (current, reverse) {
                (Some(position), false) => (position + offset) % targets.len(),
                (Some(position), true) => (position + targets.len() - offset) % targets.len(),
                (None, false) => offset - 1,
                (None, true) => targets.len() - offset,
            };
            let target = &targets[position];
            (target.enabled && target.focusable).then_some(target.index)
        });
        self.set_focus(next);
    }
}

fn target(targets: &[Target], index: Option<usize>) -> Option<&Target> {
    let index = index?;
    targets.iter().find(|target| target.index == index)
}

fn eligible(targets: &[Target], index: Option<usize>) -> Option<&Target> {
    target(targets, index).filter(|target| target.enabled && target.focusable)
}

fn shortcut(modifiers: Modifiers) -> bool {
    modifiers.control || modifiers.alt || modifiers.super_key
}

#[cfg(test)]
mod tests;
