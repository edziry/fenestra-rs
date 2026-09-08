use accesskit::{ActionData, ActionRequest, Point};

use super::*;
use crate::{AccessibilityNode, Bounds, ControlState};

fn node(id: u64, role: AccessibilityRole, children: &[u64]) -> AccessibilityNode {
    AccessibilityNode {
        id: AccessibilityId::new(id),
        role,
        name: format!("node-{id}"),
        label: format!("Label {id}"),
        bounds: Bounds {
            x: 4,
            y: 8,
            width: 20,
            height: 12,
        },
        children: children
            .iter()
            .map(|&id| AccessibilityId::new(id))
            .collect(),
        control_state: None,
        focusable: false,
    }
}

fn tree() -> AccessibilityTree {
    let mut button = node(3, AccessibilityRole::Button, &[]);
    button.control_state = Some(ControlState::default());
    button.focusable = true;
    let mut checkbox = node(4, AccessibilityRole::Checkbox, &[]);
    checkbox.control_state = Some(ControlState {
        disabled: true,
        checked: Some(true),
        ..ControlState::default()
    });
    AccessibilityTree {
        generation: 7,
        viewport: Size::new(100, 80),
        focus: AccessibilityId::new(3),
        window_focused: false,
        nodes: vec![
            node(0, AccessibilityRole::Window, &[1]),
            node(1, AccessibilityRole::Group, &[2, 3, 4]),
            node(2, AccessibilityRole::Label, &[]),
            button,
            checkbox,
        ],
    }
}

fn request(action: Action, id: u64) -> ActionRequest {
    ActionRequest {
        action,
        target_tree: TreeId::ROOT,
        target_node: NodeId(id),
        data: None,
    }
}

#[test]
fn full_tree_preserves_order_labels_physical_bounds_and_logical_focus() {
    let tree = tree();
    let update = tree_update(Some(&tree), "Preferences", tree.viewport());
    assert_eq!(update.tree_id, TreeId::ROOT);
    assert_eq!(update.tree.unwrap().root, NodeId(0));
    assert_eq!(update.focus, NodeId(3));
    assert_eq!(
        update.nodes.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        [NodeId(0), NodeId(1), NodeId(2), NodeId(3), NodeId(4)]
    );
    let root = &update.nodes[0].1;
    assert_eq!(root.role(), Role::Window);
    assert_eq!(root.label(), Some("Preferences"));
    assert!(root.clips_children());
    assert_eq!(root.children(), [NodeId(1)]);
    let group = &update.nodes[1].1;
    assert_eq!(group.role(), Role::GenericContainer);
    assert_eq!(group.children(), [NodeId(2), NodeId(3), NodeId(4)]);
    let label = &update.nodes[2].1;
    assert_eq!(label.role(), Role::Label);
    assert_eq!(label.value(), Some("Label 2"));
    assert_eq!(label.label(), None);
    assert_eq!(label.author_id(), Some("node-2"));
    assert_eq!(label.bounds(), Some(Rect::new(4.0, 8.0, 24.0, 20.0)));
    assert_eq!(label.transform(), None);
}

#[test]
fn control_state_and_current_eligibility_define_platform_actions() {
    let mut tree = tree();
    let update = tree_update(Some(&tree), "Controls", tree.viewport());
    let button = &update.nodes[3].1;
    assert_eq!(button.role(), Role::Button);
    assert_eq!(button.label(), Some("Label 3"));
    assert!(button.supports_action(Action::Focus));
    assert!(button.supports_action(Action::Click));
    let checkbox = &update.nodes[4].1;
    assert_eq!(checkbox.role(), Role::CheckBox);
    assert!(checkbox.is_disabled());
    assert_eq!(checkbox.toggled(), Some(Toggled::True));
    assert!(!checkbox.supports_action(Action::Click));
    tree.nodes[4].control_state = Some(ControlState {
        checked: Some(false),
        ..ControlState::default()
    });
    tree.nodes[4].focusable = true;
    tree.nodes[3].focusable = false;
    let update = tree_update(Some(&tree), "Controls", tree.viewport());
    assert!(!update.nodes[3].1.supports_action(Action::Focus));
    assert_eq!(update.nodes[4].1.toggled(), Some(Toggled::False));
    assert!(update.nodes[4].1.supports_action(Action::Click));
}

#[test]
fn absent_semantics_withdraws_children_and_focus_with_a_complete_window_root() {
    let update = tree_update(None, "Legacy", Size::new(48, 32));
    assert_eq!(update.focus, NodeId(0));
    assert_eq!(update.nodes.len(), 1);
    assert!(update.tree.is_some());
    assert!(update.nodes[0].1.children().is_empty());
    assert_eq!(update.nodes[0].1.label(), Some("Legacy"));
    assert_eq!(
        update.nodes[0].1.bounds(),
        Some(Rect::new(0.0, 0.0, 48.0, 32.0))
    );
}

#[test]
fn only_current_control_focus_and_click_requests_cross_the_owned_boundary() {
    let tree = tree();
    for (platform, owned) in [
        (Action::Focus, AccessibilityAction::Focus),
        (Action::Click, AccessibilityAction::Activate),
    ] {
        assert_eq!(
            action_request(&tree, &request(platform, 3)),
            Some(AccessibilityActionRequest {
                target: AccessibilityId::new(3),
                action: owned,
            })
        );
        for id in [0, 1, 2, 4, u64::MAX] {
            assert_eq!(action_request(&tree, &request(platform, id)), None);
        }
    }
    assert_eq!(action_request(&tree, &request(Action::Blur, 3)), None);
    let mut with_data = request(Action::Click, 3);
    with_data.data = Some(ActionData::ScrollToPoint(Point::new(1.0, 1.0)));
    assert_eq!(action_request(&tree, &with_data), None);
    let mut foreign_tree = request(Action::Focus, 3);
    foreign_tree.target_tree = TreeId("12345678-1234-1234-1234-123456789abc".parse().unwrap());
    assert_eq!(action_request(&tree, &foreign_tree), None);
}
