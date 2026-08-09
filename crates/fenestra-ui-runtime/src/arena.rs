#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ArenaId {
    slot: usize,
    generation: u64,
}

#[derive(Clone, Debug)]
struct Slot<T> {
    generation: u64,
    value: Option<T>,
    next_free: Option<usize>,
}

#[derive(Debug)]
pub(crate) struct Arena<T> {
    slots: Vec<Slot<T>>,
    free_head: Option<usize>,
    len: usize,
}

impl<T> Arena<T> {
    pub(crate) const fn new() -> Self {
        Self {
            slots: Vec::new(),
            free_head: None,
            len: 0,
        }
    }

    pub(crate) fn insert(&mut self, value: T) -> ArenaId {
        let id = match self.free_head {
            Some(slot_index) => {
                let slot = &mut self.slots[slot_index];
                self.free_head = slot.next_free.take();
                slot.value = Some(value);
                ArenaId {
                    slot: slot_index,
                    generation: slot.generation,
                }
            }
            None => {
                let slot_index = self.slots.len();
                self.slots.push(Slot {
                    generation: 0,
                    value: Some(value),
                    next_free: None,
                });
                ArenaId {
                    slot: slot_index,
                    generation: 0,
                }
            }
        };

        self.len += 1;
        id
    }

    pub(crate) fn get(&self, id: ArenaId) -> Option<&T> {
        let slot = self.slots.get(id.slot)?;
        (slot.generation == id.generation)
            .then_some(slot.value.as_ref())
            .flatten()
    }

    pub(crate) fn get_mut(&mut self, id: ArenaId) -> Option<&mut T> {
        let slot = self.slots.get_mut(id.slot)?;
        (slot.generation == id.generation)
            .then_some(slot.value.as_mut())
            .flatten()
    }

    pub(crate) fn remove(&mut self, id: ArenaId) -> Option<T> {
        let slot = self.slots.get_mut(id.slot)?;
        if slot.generation != id.generation {
            return None;
        }

        let value = slot.value.take()?;
        self.len -= 1;

        if slot.generation != u64::MAX {
            slot.generation += 1;
            slot.next_free = self.free_head;
            self.free_head = Some(id.slot);
        }

        Some(value)
    }

    pub(crate) const fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (ArenaId, &T)> {
        self.slots.iter().enumerate().filter_map(|(index, slot)| {
            slot.value.as_ref().map(|value| {
                (
                    ArenaId {
                        slot: index,
                        generation: slot.generation,
                    },
                    value,
                )
            })
        })
    }

    pub(crate) fn fork_for_transaction(&self) -> Self
    where
        T: Clone,
    {
        Self {
            slots: self.slots.clone(),
            free_head: self.free_head,
            len: self.len,
        }
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{Arena, ArenaId, Slot};

    #[test]
    fn removed_slot_is_reused_with_the_next_generation() {
        let mut arena = Arena::new();
        let retired = arena.insert("retired");

        assert_eq!(arena.remove(retired), Some("retired"));
        let replacement = arena.insert("replacement");

        assert_eq!(retired.slot, replacement.slot);
        assert_eq!(retired.generation + 1, replacement.generation);
        assert_eq!(arena.get(retired), None);
        assert_eq!(arena.get(replacement), Some(&"replacement"));
    }

    #[test]
    fn maximum_generation_retires_the_slot_instead_of_wrapping() {
        let mut arena = Arena {
            slots: vec![Slot {
                generation: u64::MAX,
                value: Some("retired"),
                next_free: None,
            }],
            free_head: None,
            len: 1,
        };
        let maximum_id = ArenaId {
            slot: 0,
            generation: u64::MAX,
        };

        assert_eq!(arena.remove(maximum_id), Some("retired"));
        let replacement = arena.insert("replacement");

        assert_eq!(replacement.slot, 1);
        assert_eq!(replacement.generation, 0);
        assert_eq!(arena.get(maximum_id), None);
        assert_eq!(arena.get(replacement), Some(&"replacement"));
    }

    #[test]
    fn transaction_fork_preserves_live_and_free_slot_generations() {
        let mut arena = Arena::new();
        let retained = arena.insert("retained");
        let removed = arena.insert("removed");
        assert_eq!(arena.remove(removed), Some("removed"));
        let mut fork = arena.fork_for_transaction();

        let original_replacement = arena.insert("original");
        let fork_replacement = fork.insert("fork");

        assert_eq!(original_replacement, fork_replacement);
        assert_eq!(arena.get(retained), Some(&"retained"));
        assert_eq!(fork.get(retained), Some(&"retained"));
        assert_eq!(arena.get(original_replacement), Some(&"original"));
        assert_eq!(fork.get(fork_replacement), Some(&"fork"));
        assert_eq!(arena.iter().count(), 2);
        assert_eq!(fork.iter().count(), 2);
    }
}
