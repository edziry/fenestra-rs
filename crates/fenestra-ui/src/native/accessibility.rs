use accesskit::{Action, Node, NodeId, Rect, Role, Toggled, TreeId, TreeInfo, TreeUpdate};
use accesskit_winit::{Adapter, Event};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::Window;

use crate::{
    AccessibilityAction, AccessibilityActionRequest, AccessibilityId, AccessibilityRole,
    AccessibilityTree, Size,
};

#[cfg(test)]
mod tests;

pub(super) struct Accessibility {
    adapter: Adapter,
    published: Option<(Option<AccessibilityTree>, Size)>,
}

impl Accessibility {
    pub(super) fn new(
        event_loop: &ActiveEventLoop,
        window: &Window,
        proxy: EventLoopProxy<Event>,
    ) -> Self {
        let mut adapter = Adapter::with_event_loop_proxy(event_loop, window, proxy);
        // Unix initializes host bounds and focus from events, not its constructor.
        adapter.process_event(window, &WindowEvent::Resized(window.inner_size()));
        adapter.process_event(window, &WindowEvent::Focused(window.has_focus()));
        Self {
            adapter,
            published: None,
        }
    }

    pub(super) fn process_event(&mut self, window: &Window, event: &WindowEvent) {
        self.adapter.process_event(window, event);
    }

    pub(super) fn invalidate(&mut self) {
        self.published = None;
    }

    pub(super) fn update(&mut self, tree: Option<AccessibilityTree>, title: &str, size: Size) {
        if self
            .published
            .as_ref()
            .is_some_and(|(previous, previous_size)| previous == &tree && *previous_size == size)
        {
            return;
        }
        let mut published = false;
        self.adapter.update_if_active(|| {
            published = true;
            tree_update(tree.as_ref(), title, size)
        });
        if published {
            self.published = Some((tree, size));
        }
    }
}

fn tree_update(tree: Option<&AccessibilityTree>, title: &str, size: Size) -> TreeUpdate {
    let nodes = if let Some(tree) = tree {
        tree.nodes()
            .iter()
            .map(|source| {
                let role = match source.role() {
                    AccessibilityRole::Window => Role::Window,
                    AccessibilityRole::Group => Role::GenericContainer,
                    AccessibilityRole::Label => Role::Label,
                    AccessibilityRole::Button => Role::Button,
                    AccessibilityRole::Checkbox => Role::CheckBox,
                };
                let mut node = Node::new(role);
                if role == Role::Window {
                    node.set_label(title);
                    node.set_clips_children();
                } else if role == Role::Label {
                    node.set_value(source.label());
                } else {
                    node.set_label(source.label());
                }
                node.set_author_id(source.name());
                let bounds = source.bounds();
                let x = bounds.x() as f64;
                let y = bounds.y() as f64;
                node.set_bounds(Rect::new(
                    x,
                    y,
                    x + f64::from(bounds.width()),
                    y + f64::from(bounds.height()),
                ));
                node.set_children(
                    source
                        .children()
                        .iter()
                        .map(|id| NodeId(id.get()))
                        .collect::<Vec<_>>(),
                );
                if let Some(state) = source.control_state() {
                    if state.disabled() {
                        node.set_disabled();
                    }
                    if let Some(checked) = state.checked() {
                        node.set_toggled(if checked {
                            Toggled::True
                        } else {
                            Toggled::False
                        });
                    }
                    if source.focusable() {
                        node.add_action(Action::Focus);
                        node.add_action(Action::Click);
                    }
                }
                (NodeId(source.id().get()), node)
            })
            .collect()
    } else {
        let mut root = Node::new(Role::Window);
        root.set_label(title);
        root.set_bounds(Rect::new(
            0.0,
            0.0,
            f64::from(size.width()),
            f64::from(size.height()),
        ));
        root.set_clips_children();
        vec![(NodeId(0), root)]
    };
    TreeUpdate {
        nodes,
        tree: Some(TreeInfo::new(NodeId(0))),
        tree_id: TreeId::ROOT,
        focus: NodeId(tree.map_or(0, |tree| tree.focus().get())),
    }
}

pub(super) fn action_request(
    tree: &AccessibilityTree,
    request: &accesskit::ActionRequest,
) -> Option<AccessibilityActionRequest> {
    let action = action_kind(request)?;
    let target = AccessibilityId::new(request.target_node.0);
    tree.nodes().iter().find(|node| {
        node.id() == target
            && node.focusable()
            && matches!(
                node.role(),
                AccessibilityRole::Button | AccessibilityRole::Checkbox
            )
    })?;
    Some(AccessibilityActionRequest { target, action })
}

pub(super) fn action_kind(request: &accesskit::ActionRequest) -> Option<AccessibilityAction> {
    if request.target_tree != TreeId::ROOT || request.data.is_some() {
        return None;
    }
    match request.action {
        Action::Focus => Some(AccessibilityAction::Focus),
        Action::Click => Some(AccessibilityAction::Activate),
        _ => None,
    }
}
