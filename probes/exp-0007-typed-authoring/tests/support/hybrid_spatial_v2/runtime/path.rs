use fenestra_ui_ir::prototype::{ChildFactory, ValidatedConstruction};
use fenestra_ui_runtime::prototype::{CommittedRuntimeSnapshot, FragmentId, NodeId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NodePath {
    segments: Vec<PathSegment>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum PathSegment {
    Static(u16),
    Member { slot: u16, key: u64 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct FragmentPath {
    owner: NodePath,
    slot: u16,
}

impl NodePath {
    pub(super) const fn root() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    fn static_child(&self, slot: u16) -> Self {
        let mut path = self.clone();
        path.segments.push(PathSegment::Static(slot));
        path
    }

    fn member(&self, slot: u16, key: u64) -> Self {
        let mut path = self.clone();
        path.segments.push(PathSegment::Member { slot, key });
        path
    }

    pub(super) fn render(&self) -> String {
        let mut output = String::from("root");
        for segment in &self.segments {
            match segment {
                PathSegment::Static(slot) => output.push_str(&format!("/s:{slot}")),
                PathSegment::Member { slot, key } => {
                    output.push_str(&format!("/m:{slot}:{key}"));
                }
            }
        }
        output
    }
}

impl FragmentPath {
    fn new(owner: NodePath, slot: u16) -> Self {
        Self { owner, slot }
    }

    pub(super) const fn owner(&self) -> &NodePath {
        &self.owner
    }

    pub(super) const fn slot(&self) -> u16 {
        self.slot
    }

    pub(super) fn render(&self) -> String {
        format!("{}/r:{}", self.owner.render(), self.slot)
    }
}

pub(super) struct BoundIdentities {
    nodes: Vec<(NodeId, NodePath)>,
    fragments: Vec<(FragmentId, FragmentPath)>,
}

impl BoundIdentities {
    pub(super) fn bind(
        construction: &ValidatedConstruction,
        snapshot: &CommittedRuntimeSnapshot,
    ) -> Self {
        let mut identities = Self {
            nodes: Vec::new(),
            fragments: Vec::new(),
        };
        let mut work = vec![(
            snapshot.root(),
            NodePath::root(),
            construction.root_factory().id(),
        )];

        while let Some((node, path, template)) = work.pop() {
            assert_eq!(snapshot.template(node), Some(template));
            assert!(!identities.nodes.iter().any(|(id, _)| *id == node));
            identities.nodes.push((node, path.clone()));
            let factory = construction
                .template(template)
                .expect("a live template should resolve");
            let children = snapshot
                .children(node)
                .expect("a live node should expose children");
            let mut offset = 0_usize;
            let mut child_work = Vec::new();
            for (slot, child) in factory.children().enumerate() {
                let slot = u16::try_from(slot).expect("fixture slot should fit");
                match child {
                    ChildFactory::Static { template, .. } => {
                        let child_node = *children
                            .get(offset)
                            .expect("a static child should occupy its authored slot");
                        offset += 1;
                        child_work.push((child_node, path.static_child(slot), template.id()));
                    }
                    ChildFactory::Region { region, .. } => {
                        let fragment = snapshot
                            .fragment(node, region.id())
                            .expect("a region child should retain a live fragment");
                        assert!(!identities.fragments.iter().any(|(id, _)| *id == fragment));
                        identities
                            .fragments
                            .push((fragment, FragmentPath::new(path.clone(), slot)));
                        for (key, member) in snapshot
                            .keyed_members(fragment)
                            .expect("a live fragment should expose members")
                        {
                            assert_eq!(children.get(offset), Some(&member));
                            offset += 1;
                            child_work.push((
                                member,
                                path.member(slot, key),
                                region.repeat_body().id(),
                            ));
                        }
                    }
                }
            }
            assert_eq!(offset, children.len());
            work.extend(child_work.into_iter().rev());
        }

        assert_eq!(identities.nodes.len(), snapshot.node_count());
        assert_eq!(identities.fragments.len(), snapshot.fragment_count());
        identities
    }

    pub(super) fn node_path(&self, node: NodeId) -> &NodePath {
        self.nodes
            .iter()
            .find_map(|(candidate, path)| (*candidate == node).then_some(path))
            .expect("every live node should have one normalized path")
    }

    pub(super) fn fragment_path(&self, fragment: FragmentId) -> &FragmentPath {
        self.fragments
            .iter()
            .find_map(|(candidate, path)| (*candidate == fragment).then_some(path))
            .expect("every live fragment should have one normalized path")
    }

    pub(super) fn node_id(&self, path: &NodePath) -> NodeId {
        self.nodes
            .iter()
            .find_map(|(node, candidate)| (candidate == path).then_some(*node))
            .expect("every normalized path should identify one live node")
    }

    pub(super) fn nodes(&self) -> impl Iterator<Item = (NodeId, &NodePath)> {
        self.nodes.iter().map(|(node, path)| (*node, path))
    }

    pub(super) fn fragments(&self) -> impl Iterator<Item = (FragmentId, &FragmentPath)> {
        self.fragments
            .iter()
            .map(|(fragment, path)| (*fragment, path))
    }
}
