use fenestra_ui_ir::prototype::{ChildFactory, ValidatedConstruction};
use fenestra_ui_runtime::prototype::CommittedRuntimeSnapshot;

use super::path::BoundIdentities;
use super::types::{NormalizedChild, NormalizedFragment, NormalizedNode, NormalizedState};

pub(super) fn normalize_state(
    construction: &ValidatedConstruction,
    snapshot: &CommittedRuntimeSnapshot,
) -> NormalizedState {
    let identities = BoundIdentities::bind(construction, snapshot);
    let nodes = identities
        .nodes()
        .map(|(node, path)| {
            let template = snapshot
                .template(node)
                .expect("a bound node should expose its template");
            let factory = construction
                .template(template)
                .expect("a live template should resolve");
            let component = factory.component();
            assert_eq!(snapshot.component(node), Some(component.id()));
            let properties = component
                .properties()
                .map(|property| {
                    let value = snapshot
                        .property(node, property.id())
                        .expect("every schema property should have one live slot")
                        .clone();
                    (property.id().get(), value)
                })
                .collect();
            let children = factory
                .children()
                .enumerate()
                .map(|(slot, child)| {
                    let slot = u16::try_from(slot).expect("fixture slot should fit");
                    match child {
                        ChildFactory::Static { .. } => {
                            let child = child_at_slot(construction, snapshot, node, slot);
                            NormalizedChild::Static(identities.node_path(child).clone())
                        }
                        ChildFactory::Region { region, .. } => {
                            let fragment = snapshot
                                .fragment(node, region.id())
                                .expect("a region slot should expose one fragment");
                            NormalizedChild::Region(identities.fragment_path(fragment).clone())
                        }
                    }
                })
                .collect();
            NormalizedNode {
                path: path.clone(),
                parent: snapshot
                    .parent(node)
                    .map(|parent| identities.node_path(parent).clone()),
                template: template.get(),
                component: component.id().get(),
                properties,
                children,
            }
        })
        .collect();
    let fragments = identities
        .fragments()
        .map(|(fragment, path)| {
            let owner = identities.node_id(path.owner());
            let template = snapshot
                .template(owner)
                .expect("a fragment owner should expose its template");
            let factory = construction
                .template(template)
                .expect("a fragment owner template should resolve");
            let child = factory
                .children()
                .nth(usize::from(path.slot()))
                .expect("a fragment path should address an authored slot");
            let ChildFactory::Region { region, .. } = child else {
                panic!("a fragment path should address a region slot");
            };
            let members = snapshot
                .keyed_members(fragment)
                .expect("a bound fragment should expose members")
                .map(|(key, member)| (key, identities.node_path(member).clone()))
                .collect();
            NormalizedFragment {
                path: path.clone(),
                descriptor: region.id().get(),
                members,
            }
        })
        .collect();
    NormalizedState { nodes, fragments }
}

fn child_at_slot(
    construction: &ValidatedConstruction,
    snapshot: &CommittedRuntimeSnapshot,
    owner: fenestra_ui_runtime::prototype::NodeId,
    requested: u16,
) -> fenestra_ui_runtime::prototype::NodeId {
    let template = snapshot
        .template(owner)
        .expect("a live owner should expose its template");
    let factory = construction
        .template(template)
        .expect("a live owner template should resolve");
    let children = snapshot
        .children(owner)
        .expect("a live owner should expose children");
    let mut offset = 0_usize;
    for (slot, child) in factory.children().enumerate() {
        match child {
            ChildFactory::Static { .. } => {
                if slot == usize::from(requested) {
                    return children[offset];
                }
                offset += 1;
            }
            ChildFactory::Region { region, .. } => {
                let fragment = snapshot
                    .fragment(owner, region.id())
                    .expect("a region slot should expose one fragment");
                offset += snapshot
                    .keyed_members(fragment)
                    .expect("a live fragment should expose members")
                    .len();
            }
        }
    }
    panic!("a static slot should resolve one child")
}
