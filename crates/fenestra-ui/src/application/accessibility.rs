use super::{Application, controls, interaction::Transition};
use crate::model::ElementKind;
use crate::{
    AccessibilityAction, AccessibilityActionRequest, AccessibilityId, AccessibilityNode,
    AccessibilityRole, AccessibilityTree, Bounds, Error, Event,
};

impl Application {
    /// Exports accepted control semantics and standalone text without platform types.
    pub fn accessibility_tree(&self) -> Result<AccessibilityTree, Error> {
        let included = self
            .nodes
            .iter()
            .map(|node| node.control_owner.is_none() || node.control.is_some())
            .collect::<Vec<_>>();
        let targets = self.control_targets()?;
        let mut nodes = vec![AccessibilityNode {
            id: AccessibilityId::new(0),
            role: AccessibilityRole::Window,
            name: String::new(),
            label: String::new(),
            bounds: Bounds {
                x: 0,
                y: 0,
                width: self.size.width(),
                height: self.size.height(),
            },
            children: vec![identity(0)?],
            control_state: None,
            focusable: false,
        }];
        for (index, node) in self
            .nodes
            .iter()
            .enumerate()
            .filter(|(index, _)| included[*index])
        {
            let role = match node.kind {
                ElementKind::Button => AccessibilityRole::Button,
                ElementKind::Checkbox => AccessibilityRole::Checkbox,
                ElementKind::Text => AccessibilityRole::Label,
                _ => AccessibilityRole::Group,
            };
            let label = node
                .control
                .as_ref()
                .map(|control| control.label.to_string())
                .or_else(|| node.text.as_ref().map(|text| text.content.to_string()))
                .unwrap_or_default();
            let children = node
                .children
                .iter()
                .filter(|&&child| included[child])
                .map(|&child| identity(child))
                .collect::<Result<Vec<_>, _>>()?;
            nodes.push(AccessibilityNode {
                id: identity(index)?,
                role,
                name: node.name.clone(),
                label,
                bounds: self.bounds(&node.name)?,
                children,
                control_state: node
                    .control
                    .as_ref()
                    .map(|_| controls::state(&self.nodes, index, &self.interaction)),
                focusable: targets
                    .iter()
                    .any(|target| target.index == index && target.focusable),
            });
        }
        Ok(AccessibilityTree {
            generation: self.generation(),
            viewport: self.size,
            focus: self
                .interaction
                .focus
                .map(identity)
                .transpose()?
                .unwrap_or(AccessibilityId::new(0)),
            window_focused: self.interaction.window_focused,
            nodes,
        })
    }

    /// Applies a semantic action and returns notifications after atomic publication.
    ///
    /// Stale, non-control, disabled and offscreen targets are ignored. Activation
    /// cancels pending device arms, does not move focus, and is independent of
    /// keyboard modifiers, composition and native window focus. Rendering failure
    /// preserves accepted state and returns no notifications; callers may retry.
    pub fn dispatch_accessibility_action(
        &mut self,
        request: AccessibilityActionRequest,
    ) -> Result<Vec<Event>, Error> {
        let Some(index) = request
            .target
            .get()
            .checked_sub(1)
            .and_then(|index| usize::try_from(index).ok())
        else {
            return Ok(Vec::new());
        };
        if !self
            .control_targets()?
            .iter()
            .any(|target| target.index == index && target.focusable)
        {
            return Ok(Vec::new());
        }
        let mut next = self.interaction.clone();
        let activate = match request.action {
            AccessibilityAction::Focus => {
                next.set_focus(Some(index));
                None
            }
            AccessibilityAction::Activate => {
                next.clear_arms();
                Some(index)
            }
        };
        self.apply_control_transition(Transition { next, activate })
    }
}

fn identity(index: usize) -> Result<AccessibilityId, Error> {
    u64::try_from(index)
        .ok()
        .and_then(|index| index.checked_add(1))
        .map(AccessibilityId::new)
        .ok_or(Error::CapacityOverflow)
}
