use fenestra_ui_ir::prototype::{ComponentTypeId, PropertyValue, TemplateNodeId};
use fenestra_ui_runtime::prototype::{FragmentId, NodeId};
use fenestra_ui_spatial::prototype::SpatialViewportV2;

use crate::RuntimeSpatialBuildViewV2;
use crate::support::headless::{
    COLOR, COMPONENT, CONTROL, CONTROL_STYLE_COLOR, FIRST_KEY, HEIGHT, ITEM, ITEM_STYLE_COLOR,
    ITEMS, SECOND_KEY, WIDTH,
};

#[derive(Clone, Copy)]
pub struct ForeignIds {
    pub node: NodeId,
    pub fragment: FragmentId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LogicalNodes {
    pub root: NodeId,
    pub container: NodeId,
    pub control: NodeId,
    pub first_item: NodeId,
    pub second_item: NodeId,
    pub items: FragmentId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildFacts {
    pub viewport: SpatialViewportV2,
    pub nodes: LogicalNodes,
    pub node_count: usize,
    pub root_template: Option<TemplateNodeId>,
    pub control_template: Option<TemplateNodeId>,
    pub item_template: Option<TemplateNodeId>,
    pub item_component: Option<ComponentTypeId>,
    pub root_width: Option<PropertyValue>,
    pub control_width: Option<PropertyValue>,
    pub control_color: Option<PropertyValue>,
    pub first_height: Option<PropertyValue>,
    pub first_color: Option<PropertyValue>,
    pub second_height: Option<PropertyValue>,
    pub control_parent: Option<NodeId>,
    pub root_children: Option<Vec<NodeId>>,
    pub control_children: Option<Vec<NodeId>>,
    pub members: Option<Vec<(u64, NodeId)>>,
    pub first_lookup: Option<NodeId>,
    pub missing_lookup: Option<NodeId>,
    pub empty_fragment: Option<FragmentId>,
    pub foreign_node_absent: bool,
    pub foreign_fragment_absent: bool,
}

impl BuildFacts {
    pub fn capture(
        runtime: RuntimeSpatialBuildViewV2<'_>,
        viewport: SpatialViewportV2,
        foreign: Option<ForeignIds>,
    ) -> Self {
        let root = runtime.root();
        let root_children = runtime.children(root).map(<[NodeId]>::to_vec);
        let container = root_children.as_ref().expect("root should be live")[0];
        let container_children = runtime
            .children(container)
            .expect("container should be live");
        let control = container_children[0];
        let items = runtime
            .fragment(container, ITEMS)
            .expect("item fragment should be live");
        let members = runtime
            .keyed_members(items)
            .map(Iterator::collect::<Vec<_>>);
        let members_ref = members.as_ref().expect("item members should be live");
        let first_item = members_ref[0].1;
        let second_item = members_ref[1].1;
        let nodes = LogicalNodes {
            root,
            container,
            control,
            first_item,
            second_item,
            items,
        };
        let (foreign_node_absent, foreign_fragment_absent) = foreign.map_or((true, true), |ids| {
            (
                runtime.template(ids.node).is_none()
                    && runtime.component(ids.node).is_none()
                    && runtime.property(ids.node, WIDTH).is_none()
                    && runtime.parent(ids.node).is_none()
                    && runtime.children(ids.node).is_none()
                    && runtime.fragment(ids.node, ITEMS).is_none(),
                runtime.keyed_members(ids.fragment).is_none()
                    && runtime.keyed_member(ids.fragment, FIRST_KEY).is_none(),
            )
        });

        Self {
            viewport,
            nodes,
            node_count: runtime.node_count(),
            root_template: runtime.template(root),
            control_template: runtime.template(control),
            item_template: runtime.template(first_item),
            item_component: runtime.component(first_item),
            root_width: runtime.property(root, WIDTH).cloned(),
            control_width: runtime.property(control, WIDTH).cloned(),
            control_color: runtime.property(control, COLOR).cloned(),
            first_height: runtime.property(first_item, HEIGHT).cloned(),
            first_color: runtime.property(first_item, COLOR).cloned(),
            second_height: runtime.property(second_item, HEIGHT).cloned(),
            control_parent: runtime.parent(control),
            root_children,
            control_children: runtime.children(control).map(<[NodeId]>::to_vec),
            members,
            first_lookup: runtime.keyed_member(items, FIRST_KEY),
            missing_lookup: runtime.keyed_member(items, u64::MAX),
            empty_fragment: runtime.fragment(root, ITEMS),
            foreign_node_absent,
            foreign_fragment_absent,
        }
    }

    pub fn assert_complete_styled_view(&self) {
        assert_eq!(self.node_count, 5);
        assert_eq!(self.root_template, Some(crate::support::headless::ROOT));
        assert_eq!(self.control_template, Some(CONTROL));
        assert_eq!(self.item_template, Some(ITEM));
        assert_eq!(self.item_component, Some(COMPONENT));
        assert_eq!(self.root_width, Some(PropertyValue::ScalarI32(100)));
        assert_eq!(self.control_width, Some(PropertyValue::ScalarI32(37)));
        assert_eq!(
            self.control_color,
            Some(PropertyValue::Rgba8(CONTROL_STYLE_COLOR))
        );
        assert_eq!(self.first_height, Some(PropertyValue::ScalarI32(17)));
        assert_eq!(
            self.first_color,
            Some(PropertyValue::Rgba8(ITEM_STYLE_COLOR))
        );
        assert_eq!(self.second_height, Some(PropertyValue::ScalarI32(17)));
        assert_eq!(self.control_parent, Some(self.nodes.container));
        assert_eq!(self.root_children, Some(vec![self.nodes.container]));
        assert_eq!(self.control_children, Some(Vec::new()));
        assert_eq!(
            self.members,
            Some(vec![
                (FIRST_KEY, self.nodes.first_item),
                (SECOND_KEY, self.nodes.second_item),
            ])
        );
        assert_eq!(self.first_lookup, Some(self.nodes.first_item));
        assert_eq!(self.missing_lookup, None);
        assert_eq!(self.empty_fragment, None);
        assert!(self.foreign_node_absent);
        assert!(self.foreign_fragment_absent);
        assert_eq!(self.viewport, super::VIEWPORT);
    }
}
