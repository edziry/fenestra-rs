use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use fenestra_ui_ir::prototype::StructuralRegionId;

use crate::arena::{Arena, ArenaId};
use crate::logical_tree::NodeId;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct RuntimeDomain(u64);

impl RuntimeDomain {
    fn next() -> Self {
        static NEXT_RUNTIME_DOMAIN: AtomicU64 = AtomicU64::new(1);

        let id = NEXT_RUNTIME_DOMAIN
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .unwrap_or_else(|_| panic!("runtime identity space exhausted"));
        Self(id)
    }
}

/// Opaque generational identity for one runtime structural fragment.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct FragmentId {
    domain: RuntimeDomain,
    arena: ArenaId,
}

impl fmt::Debug for FragmentId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("FragmentId(..)")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KeyedMember {
    pub(crate) key: u64,
    pub(crate) root: NodeId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Fragment {
    pub(crate) owner: NodeId,
    pub(crate) descriptor: StructuralRegionId,
    pub(crate) members: Vec<KeyedMember>,
}

#[derive(Debug)]
pub(crate) struct FragmentStore {
    domain: RuntimeDomain,
    fragments: Arena<Fragment>,
}

impl FragmentStore {
    pub(crate) fn new() -> Self {
        Self {
            domain: RuntimeDomain::next(),
            fragments: Arena::new(),
        }
    }

    pub(crate) fn insert(&mut self, fragment: Fragment) -> FragmentId {
        FragmentId {
            domain: self.domain,
            arena: self.fragments.insert(fragment),
        }
    }

    pub(crate) fn get(&self, id: FragmentId) -> Option<&Fragment> {
        (id.domain == self.domain)
            .then(|| self.fragments.get(id.arena))
            .flatten()
    }

    pub(crate) fn get_mut(&mut self, id: FragmentId) -> Option<&mut Fragment> {
        (id.domain == self.domain)
            .then(|| self.fragments.get_mut(id.arena))
            .flatten()
    }

    pub(crate) fn remove(&mut self, id: FragmentId) -> Option<Fragment> {
        (id.domain == self.domain)
            .then(|| self.fragments.remove(id.arena))
            .flatten()
    }

    pub(crate) const fn len(&self) -> usize {
        self.fragments.len()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (FragmentId, &Fragment)> {
        self.fragments.iter().map(|(arena, fragment)| {
            (
                FragmentId {
                    domain: self.domain,
                    arena,
                },
                fragment,
            )
        })
    }

    pub(crate) fn find(&self, owner: NodeId, descriptor: StructuralRegionId) -> Option<FragmentId> {
        self.iter().find_map(|(id, fragment)| {
            (fragment.owner == owner && fragment.descriptor == descriptor).then_some(id)
        })
    }

    pub(crate) fn fork_for_transaction(&self) -> Self {
        Self {
            domain: self.domain,
            fragments: self.fragments.fork_for_transaction(),
        }
    }
}
