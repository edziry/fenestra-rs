use fenestra_ui_ir::prototype::{
    ComponentTypeId, InputPolicy, PropertyId, PropertyValue, StructuralRegionId, TemplateNodeId,
};

use crate::error::{HarnessError, HarnessErrorKind};

use super::{
    FragmentPathV1, NodePathV1, NormalizedChildGroupV1, NormalizedFragmentV1, NormalizedMemberV1,
    NormalizedNodeV1, NormalizedPropertyV1, NormalizedStateV1, PathSegmentV1,
};

/// Registered typed defects for normalized logical-state evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NormalizedStateFaultV1 {
    /// Perturbs one node path.
    NodePath,
    /// Perturbs one node parent.
    NodeParent,
    /// Perturbs one node template symbol.
    NodeTemplate,
    /// Perturbs one node component symbol.
    NodeComponent,
    /// Perturbs authored node order.
    NodeOrder,
    /// Perturbs effective-property order.
    PropertyOrder,
    /// Perturbs one property symbol.
    PropertyId,
    /// Perturbs one property value without changing its type.
    PropertyValue,
    /// Perturbs authored child-group order.
    ChildOrder,
    /// Changes one child-group kind.
    ChildKind,
    /// Perturbs one child-group target without changing its kind.
    ChildTarget,
    /// Perturbs one fragment path.
    FragmentPath,
    /// Perturbs one fragment descriptor symbol.
    FragmentDescriptor,
    /// Perturbs committed member order.
    MemberOrder,
    /// Perturbs one member key.
    MemberKey,
    /// Perturbs one member node path.
    MemberPath,
}

impl NormalizedStateFaultV1 {
    /// All registered state defects in canonical evidence order.
    pub const ALL: [Self; 16] = [
        Self::NodePath,
        Self::NodeParent,
        Self::NodeTemplate,
        Self::NodeComponent,
        Self::NodeOrder,
        Self::PropertyOrder,
        Self::PropertyId,
        Self::PropertyValue,
        Self::ChildOrder,
        Self::ChildKind,
        Self::ChildTarget,
        Self::FragmentPath,
        Self::FragmentDescriptor,
        Self::MemberOrder,
        Self::MemberKey,
        Self::MemberPath,
    ];
}

/// Applies one registered defect to a clone of a normalized logical state.
pub fn inject_normalized_state_fault_v1(
    state: &NormalizedStateV1,
    fault: NormalizedStateFaultV1,
) -> Result<NormalizedStateV1, HarnessError> {
    let mut faulted = state.clone();
    match fault {
        NormalizedStateFaultV1::NodePath => {
            let node = faulted.nodes.first_mut().ok_or_else(invalid)?;
            node.path = perturb_node_path(&node.path);
        }
        NormalizedStateFaultV1::NodeParent => {
            let node = faulted
                .nodes
                .iter_mut()
                .find(|node| node.parent.is_some())
                .ok_or_else(invalid)?;
            node.parent = node.parent.as_ref().map(perturb_node_path);
        }
        NormalizedStateFaultV1::NodeTemplate => {
            let node = faulted.nodes.first_mut().ok_or_else(invalid)?;
            node.template = TemplateNodeId::new(perturb_u32(node.template.get()));
        }
        NormalizedStateFaultV1::NodeComponent => {
            let node = faulted.nodes.first_mut().ok_or_else(invalid)?;
            node.component = ComponentTypeId::new(perturb_u32(node.component.get()));
        }
        NormalizedStateFaultV1::NodeOrder => swap_pair(&mut faulted.nodes)?,
        NormalizedStateFaultV1::PropertyOrder => {
            let properties = property_pair(&mut faulted.nodes)?;
            properties.swap(0, 1);
        }
        NormalizedStateFaultV1::PropertyId => {
            let property = first_property(&mut faulted.nodes)?;
            property.property = PropertyId::new(perturb_u32(property.property.get()));
        }
        NormalizedStateFaultV1::PropertyValue => {
            let property = first_property(&mut faulted.nodes)?;
            property.value = perturb_value(&property.value);
        }
        NormalizedStateFaultV1::ChildOrder => {
            let children = child_pair(&mut faulted.nodes)?;
            children.swap(0, 1);
        }
        NormalizedStateFaultV1::ChildKind => {
            let child = first_child(&mut faulted.nodes)?;
            *child = change_child_kind(child)?;
        }
        NormalizedStateFaultV1::ChildTarget => {
            let child = first_child(&mut faulted.nodes)?;
            *child = perturb_child_target(child);
        }
        NormalizedStateFaultV1::FragmentPath => {
            let fragment = faulted.fragments.first_mut().ok_or_else(invalid)?;
            fragment.path = perturb_fragment_path(&fragment.path);
        }
        NormalizedStateFaultV1::FragmentDescriptor => {
            let fragment = faulted.fragments.first_mut().ok_or_else(invalid)?;
            fragment.descriptor = StructuralRegionId::new(perturb_u32(fragment.descriptor.get()));
        }
        NormalizedStateFaultV1::MemberOrder => {
            let members = member_pair(&mut faulted.fragments)?;
            members.swap(0, 1);
        }
        NormalizedStateFaultV1::MemberKey => {
            let member = first_member(&mut faulted.fragments)?;
            member.key = perturb_u64(member.key);
        }
        NormalizedStateFaultV1::MemberPath => {
            let member = first_member(&mut faulted.fragments)?;
            member.node = perturb_node_path(&member.node);
        }
    }
    Ok(faulted)
}

fn first_property(
    nodes: &mut [NormalizedNodeV1],
) -> Result<&mut NormalizedPropertyV1, HarnessError> {
    nodes
        .iter_mut()
        .find_map(|node| node.properties.first_mut())
        .ok_or_else(invalid)
}

fn property_pair(
    nodes: &mut [NormalizedNodeV1],
) -> Result<&mut Vec<NormalizedPropertyV1>, HarnessError> {
    nodes
        .iter_mut()
        .find(|node| node.properties.len() >= 2)
        .map(|node| &mut node.properties)
        .ok_or_else(invalid)
}

fn first_child(
    nodes: &mut [NormalizedNodeV1],
) -> Result<&mut NormalizedChildGroupV1, HarnessError> {
    nodes
        .iter_mut()
        .find_map(|node| node.child_groups.first_mut())
        .ok_or_else(invalid)
}

fn child_pair(
    nodes: &mut [NormalizedNodeV1],
) -> Result<&mut Vec<NormalizedChildGroupV1>, HarnessError> {
    nodes
        .iter_mut()
        .find(|node| node.child_groups.len() >= 2)
        .map(|node| &mut node.child_groups)
        .ok_or_else(invalid)
}

fn first_member(
    fragments: &mut [NormalizedFragmentV1],
) -> Result<&mut NormalizedMemberV1, HarnessError> {
    fragments
        .iter_mut()
        .find_map(|fragment| fragment.members.first_mut())
        .ok_or_else(invalid)
}

fn member_pair(
    fragments: &mut [NormalizedFragmentV1],
) -> Result<&mut Vec<NormalizedMemberV1>, HarnessError> {
    fragments
        .iter_mut()
        .find(|fragment| fragment.members.len() >= 2)
        .map(|fragment| &mut fragment.members)
        .ok_or_else(invalid)
}

fn change_child_kind(
    child: &NormalizedChildGroupV1,
) -> Result<NormalizedChildGroupV1, HarnessError> {
    Ok(match child {
        NormalizedChildGroupV1::Static(path) => {
            let (owner, authored_slot) = static_owner_and_slot(path)?;
            NormalizedChildGroupV1::Region(FragmentPathV1::new(owner, authored_slot))
        }
        NormalizedChildGroupV1::Region(path) => {
            NormalizedChildGroupV1::Static(path.owner().clone().static_child(path.region_slot()))
        }
    })
}

fn static_owner_and_slot(path: &NodePathV1) -> Result<(NodePathV1, u16), HarnessError> {
    let (last, owner_segments) = path.segments().split_last().ok_or_else(invalid)?;
    let PathSegmentV1::Static { authored_slot } = last else {
        return Err(invalid());
    };
    let owner = owner_segments
        .iter()
        .fold(NodePathV1::root(), |path, segment| match segment {
            PathSegmentV1::Static { authored_slot } => path.static_child(*authored_slot),
            PathSegmentV1::Member { region_slot, key } => path.member(*region_slot, *key),
        });
    Ok((owner, *authored_slot))
}

fn perturb_child_target(child: &NormalizedChildGroupV1) -> NormalizedChildGroupV1 {
    match child {
        NormalizedChildGroupV1::Static(path) => {
            NormalizedChildGroupV1::Static(perturb_node_path(path))
        }
        NormalizedChildGroupV1::Region(path) => {
            NormalizedChildGroupV1::Region(perturb_fragment_path(path))
        }
    }
}

fn perturb_node_path(path: &NodePathV1) -> NodePathV1 {
    path.clone().static_child(u16::MAX)
}

fn perturb_fragment_path(path: &FragmentPathV1) -> FragmentPathV1 {
    FragmentPathV1::new(path.owner().clone(), perturb_u16(path.region_slot()))
}

fn perturb_value(value: &PropertyValue) -> PropertyValue {
    match value {
        PropertyValue::Bool(value) => PropertyValue::Bool(!value),
        PropertyValue::ScalarI32(value) => PropertyValue::ScalarI32(perturb_i32(*value)),
        PropertyValue::Rgba8(value) => {
            let mut value = *value;
            value[0] ^= 1;
            PropertyValue::Rgba8(value)
        }
        PropertyValue::InputPolicy(InputPolicy::Accept) => {
            PropertyValue::InputPolicy(InputPolicy::Ignore)
        }
        PropertyValue::InputPolicy(InputPolicy::Ignore) => {
            PropertyValue::InputPolicy(InputPolicy::Accept)
        }
    }
}

fn swap_pair<T>(values: &mut [T]) -> Result<(), HarnessError> {
    if values.len() < 2 {
        return Err(invalid());
    }
    values.swap(0, 1);
    Ok(())
}

const fn perturb_u16(value: u16) -> u16 {
    if value == u16::MAX {
        value - 1
    } else {
        value + 1
    }
}

const fn perturb_u32(value: u32) -> u32 {
    if value == u32::MAX {
        value - 1
    } else {
        value + 1
    }
}

const fn perturb_u64(value: u64) -> u64 {
    if value == u64::MAX {
        value - 1
    } else {
        value + 1
    }
}

const fn perturb_i32(value: i32) -> i32 {
    if value == i32::MAX {
        value - 1
    } else {
        value + 1
    }
}

fn invalid() -> HarnessError {
    HarnessError::new(HarnessErrorKind::InvalidOperation)
}
