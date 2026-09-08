// Fenestra regression for the locally patched AT-SPI state translation.
// Licensed under the same MIT OR Apache-2.0 terms as this vendored package.

use std::sync::{Arc, Mutex};

use accesskit::{
    Action, ActionHandler, ActionRequest, Node, NodeId, Toggled, TreeId, TreeInfo, TreeUpdate,
};

use crate::{
    Adapter, AdapterCallback, AppContext, Event, FullNodeId, InterfaceSet, ObjectEvent, State,
    WindowBounds,
};

#[derive(Clone, Default)]
struct Callback(Arc<Mutex<Vec<(State, bool)>>>);

impl AdapterCallback for Callback {
    fn register_interfaces(&self, _: &Adapter, _: FullNodeId, _: InterfaceSet) {}

    fn unregister_interfaces(&self, _: &Adapter, _: FullNodeId, _: InterfaceSet) {}

    fn emit_event(&self, _: &Adapter, event: Event) {
        if let Event::Object {
            event: ObjectEvent::StateChanged(state, value),
            ..
        } = event
        {
            self.0.lock().unwrap().push((state, value));
        }
    }
}

struct NoActions;

impl ActionHandler for NoActions {
    fn do_action(&mut self, _: ActionRequest) {}
}

fn update(role: accesskit::Role, disabled: bool) -> TreeUpdate {
    let mut root = Node::new(accesskit::Role::Window);
    root.set_children([NodeId(1)]);
    let mut control = Node::new(role);
    control.set_label("Control");
    if disabled {
        control.set_disabled();
    } else {
        control.add_action(Action::Focus);
        control.add_action(Action::Click);
    }
    if role == accesskit::Role::CheckBox {
        control.set_toggled(Toggled::True);
    }
    TreeUpdate {
        nodes: vec![(NodeId(0), root), (NodeId(1), control)],
        tree: Some(TreeInfo::new(NodeId(0))),
        tree_id: TreeId::ROOT,
        focus: NodeId(0),
    }
}

fn adapter(role: accesskit::Role, disabled: bool, callback: Callback) -> Adapter {
    Adapter::new(
        &AppContext::new(Some("Fenestra state regression".into())),
        callback,
        update(role, disabled),
        true,
        WindowBounds::default(),
        NoActions,
    )
}

fn control(adapter: &Adapter) -> crate::PlatformNode {
    let root = adapter.platform_node(adapter.root_id());
    adapter.platform_node(root.child_at_index(0).unwrap().unwrap())
}

#[test]
fn disabled_controls_are_neither_enabled_nor_sensitive() {
    for role in [
        accesskit::Role::Button,
        accesskit::Role::DefaultButton,
        accesskit::Role::CheckBox,
    ] {
        let adapter = adapter(role, true, Callback::default());
        let state = control(&adapter).state();
        assert!(!state.contains(State::Enabled), "{role:?}: {state:?}");
        assert!(!state.contains(State::Sensitive), "{role:?}: {state:?}");
    }
}

#[test]
fn enabled_controls_retain_their_roles_actions_and_checked_state() {
    for (role, expected) in [
        (accesskit::Role::Button, crate::Role::Button),
        (accesskit::Role::DefaultButton, crate::Role::Button),
        (accesskit::Role::CheckBox, crate::Role::CheckBox),
    ] {
        let adapter = adapter(role, false, Callback::default());
        let control = control(&adapter);
        let state = control.state();
        assert_eq!(control.role().unwrap(), expected);
        assert!(state.contains(State::Enabled | State::Sensitive));
        assert!(control.supports_action().unwrap());
        assert_eq!(
            state.contains(State::Checked),
            role == accesskit::Role::CheckBox
        );
    }
}

#[test]
fn disabled_button_transitions_publish_matching_state_changes() {
    let callback = Callback::default();
    let mut adapter = adapter(accesskit::Role::Button, false, callback.clone());
    for disabled in [true, false] {
        callback.0.lock().unwrap().clear();
        adapter.update(update(accesskit::Role::Button, disabled));
        let state = control(&adapter).state();
        assert_eq!(state.contains(State::Enabled), !disabled);
        assert_eq!(state.contains(State::Sensitive), !disabled);
        let events = callback.0.lock().unwrap();
        assert!(events.contains(&(State::Enabled, !disabled)));
        assert!(events.contains(&(State::Sensitive, !disabled)));
    }
}
