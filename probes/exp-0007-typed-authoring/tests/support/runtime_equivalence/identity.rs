use fenestra_ui_ir::prototype::{ChildFactory, ValidatedConstruction};
use fenestra_ui_runtime::prototype::{CommittedRuntimeSnapshot, FragmentId, NodeId, UiTransaction};
use fenestra_ui_testkit::prototype::{
    FragmentPathV1, NodePathV1, PathSegmentV1, SemanticOperationV1,
};

pub(super) struct BoundIdentities {
    nodes: Vec<(NodeId, NodePathV1)>,
    fragments: Vec<(FragmentId, FragmentPathV1)>,
}

impl BoundIdentities {
    pub(super) fn bind(
        construction: &ValidatedConstruction,
        snapshot: &CommittedRuntimeSnapshot,
    ) -> Result<Self, &'static str> {
        let mut identities = Self {
            nodes: Vec::new(),
            fragments: Vec::new(),
        };
        let mut work = vec![(
            snapshot.root(),
            NodePathV1::root(),
            construction.root_factory().id(),
        )];

        while let Some((node, path, template)) = work.pop() {
            if snapshot.template(node) != Some(template)
                || identities.nodes.iter().any(|(id, _)| *id == node)
            {
                return Err("runtime-node-identity");
            }
            identities.nodes.push((node, path.clone()));
            let factory = construction.template(template).ok_or("runtime-template")?;
            let children = snapshot.children(node).ok_or("runtime-children")?;
            let mut offset = 0_usize;
            let mut child_work = Vec::new();
            for (slot, child) in factory.children().enumerate() {
                let slot = u16::try_from(slot).map_err(|_| "runtime-slot")?;
                match child {
                    ChildFactory::Static { template, .. } => {
                        let child_node = *children.get(offset).ok_or("runtime-static-child")?;
                        offset = offset.checked_add(1).ok_or("runtime-offset")?;
                        child_work.push((
                            child_node,
                            path.clone().static_child(slot),
                            template.id(),
                        ));
                    }
                    ChildFactory::Region { region, .. } => {
                        let fragment = snapshot
                            .fragment(node, region.id())
                            .ok_or("runtime-fragment")?;
                        if identities.fragments.iter().any(|(id, _)| *id == fragment) {
                            return Err("runtime-fragment-identity");
                        }
                        identities
                            .fragments
                            .push((fragment, FragmentPathV1::new(path.clone(), slot)));
                        let members = snapshot.keyed_members(fragment).ok_or("runtime-members")?;
                        for (key, member) in members {
                            if children.get(offset) != Some(&member) {
                                return Err("runtime-member-order");
                            }
                            offset = offset.checked_add(1).ok_or("runtime-offset")?;
                            child_work.push((
                                member,
                                path.clone().member(slot, key),
                                region.repeat_body().id(),
                            ));
                        }
                    }
                }
            }
            if offset != children.len() {
                return Err("runtime-child-count");
            }
            work.extend(child_work.into_iter().rev());
        }

        if identities.nodes.len() != snapshot.node_count()
            || identities.fragments.len() != snapshot.fragment_count()
        {
            return Err("runtime-identity-count");
        }
        Ok(identities)
    }

    pub(super) fn node_path(&self, node: NodeId) -> Option<&NodePathV1> {
        self.nodes
            .iter()
            .find_map(|(candidate, path)| (*candidate == node).then_some(path))
    }

    pub(super) fn fragment_path(&self, fragment: FragmentId) -> Option<&FragmentPathV1> {
        self.fragments
            .iter()
            .find_map(|(candidate, path)| (*candidate == fragment).then_some(path))
    }
}

pub(super) fn stage_operation(
    construction: &ValidatedConstruction,
    snapshot: &CommittedRuntimeSnapshot,
    transaction: &mut UiTransaction,
    operation: &SemanticOperationV1,
) -> Result<(), &'static str> {
    match operation {
        SemanticOperationV1::SetProperty {
            node,
            property,
            value,
        } => transaction.set_property(
            resolve_node(construction, snapshot, node)?,
            *property,
            value.clone(),
        ),
        SemanticOperationV1::InsertKeyed {
            fragment,
            key,
            final_index,
        } => transaction.insert_keyed(
            resolve_fragment(construction, snapshot, fragment)?,
            *key,
            usize::try_from(*final_index).map_err(|_| "runtime-index")?,
        ),
        SemanticOperationV1::MoveKeyed {
            fragment,
            key,
            final_index,
        } => transaction.move_keyed(
            resolve_fragment(construction, snapshot, fragment)?,
            *key,
            usize::try_from(*final_index).map_err(|_| "runtime-index")?,
        ),
        SemanticOperationV1::UpdateKeyed {
            fragment,
            key,
            property,
            value,
        } => transaction.update_keyed(
            resolve_fragment(construction, snapshot, fragment)?,
            *key,
            *property,
            value.clone(),
        ),
        SemanticOperationV1::RemoveKeyed { fragment, key } => {
            transaction.remove_keyed(resolve_fragment(construction, snapshot, fragment)?, *key)
        }
    }
    .map_err(|_| "runtime-stage")
}

pub(super) fn resolve_fragment(
    construction: &ValidatedConstruction,
    snapshot: &CommittedRuntimeSnapshot,
    path: &FragmentPathV1,
) -> Result<FragmentId, &'static str> {
    let owner = resolve_node(construction, snapshot, path.owner())?;
    let template = snapshot.template(owner).ok_or("runtime-template")?;
    let factory = construction.template(template).ok_or("runtime-template")?;
    let child = factory
        .children()
        .nth(usize::from(path.region_slot()))
        .ok_or("runtime-region-slot")?;
    let ChildFactory::Region { region, .. } = child else {
        return Err("runtime-region-kind");
    };
    snapshot
        .fragment(owner, region.id())
        .ok_or("runtime-fragment")
}

fn resolve_node(
    construction: &ValidatedConstruction,
    snapshot: &CommittedRuntimeSnapshot,
    path: &NodePathV1,
) -> Result<NodeId, &'static str> {
    let mut node = snapshot.root();
    for segment in path.segments() {
        let template = snapshot.template(node).ok_or("runtime-template")?;
        let factory = construction.template(template).ok_or("runtime-template")?;
        let child = factory
            .children()
            .nth(usize::from(segment.authored_slot()))
            .ok_or("runtime-child-slot")?;
        node = match (segment, child) {
            (PathSegmentV1::Static { authored_slot }, ChildFactory::Static { .. }) => {
                let offset = flattened_offset(
                    snapshot,
                    node,
                    factory.children().take(usize::from(*authored_slot)),
                )?;
                *snapshot
                    .children(node)
                    .and_then(|children| children.get(offset))
                    .ok_or("runtime-static-child")?
            }
            (PathSegmentV1::Member { key, .. }, ChildFactory::Region { region, .. }) => {
                let fragment = snapshot
                    .fragment(node, region.id())
                    .ok_or("runtime-fragment")?;
                snapshot
                    .keyed_member(fragment, *key)
                    .ok_or("runtime-member")?
            }
            _ => return Err("runtime-path-kind"),
        };
    }
    Ok(node)
}

fn flattened_offset<'a>(
    snapshot: &CommittedRuntimeSnapshot,
    owner: NodeId,
    mut preceding: impl Iterator<Item = ChildFactory<'a>>,
) -> Result<usize, &'static str> {
    preceding.try_fold(0_usize, |offset, child| {
        let width = match child {
            ChildFactory::Static { .. } => 1,
            ChildFactory::Region { region, .. } => snapshot
                .fragment(owner, region.id())
                .and_then(|fragment| snapshot.keyed_members(fragment))
                .map(|members| members.len())
                .ok_or("runtime-fragment")?,
        };
        offset.checked_add(width).ok_or("runtime-offset")
    })
}
