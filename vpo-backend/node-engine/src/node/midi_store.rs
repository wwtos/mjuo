use clocked::midi::MidiMessage;
use common::alloc::{BuddyAlloc, SliceAlloc};
use generational_arena::Arena;
use self_cell::self_cell;

use super::MidisIndex;

type MidiStorage<'a> = Arena<SliceAlloc<'a, MidiMessage>>;

self_cell!(
    struct MidiStoreInternal {
        owner: BuddyAlloc,

        #[covariant]
        dependent: MidiStorage,
    }
);

pub struct MidiStore {
    store: MidiStoreInternal,
}

impl std::fmt::Debug for MidiStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MidiStore {{ ")?;
        self.store.borrow_dependent().fmt(f)?;
        write!(f, " }}")
    }
}

impl MidiStore {
    pub fn new(alloc_bytes: usize, storage_size: usize) -> MidiStore {
        let store = MidiStoreInternal::new(BuddyAlloc::new(alloc_bytes), |_| Arena::with_capacity(storage_size));

        MidiStore { store }
    }

    pub fn add_midi<I>(&mut self, midis: I) -> Option<MidisIndex>
    where
        I: IntoIterator<Item = MidiMessage>,
        I::IntoIter: ExactSizeIterator,
    {
        self.store.with_dependent_mut(|alloc, arena| {
            let midi = alloc.alloc_slice_fill_iter(midis).ok()?;
            let index = arena.insert(midi);

            Some(MidisIndex(index))
        })
    }

    pub fn clone_midi(&mut self, index: MidisIndex) -> Option<MidisIndex> {
        self.store.with_dependent_mut(|alloc, arena| {
            let current_message = arena.get(index.0)?;
            let cloned_message = alloc
                .alloc_slice_fill_iter(current_message.value.iter().cloned())
                .ok()?;

            let index = arena.insert(cloned_message);

            Some(MidisIndex(index))
        })
    }

    pub fn add_midi_with<F>(&mut self, count: usize, midi: F) -> Option<MidisIndex>
    where
        F: FnMut(usize) -> MidiMessage,
    {
        self.store.with_dependent_mut(|alloc, arena| {
            let midi = alloc.alloc_slice_fill_with(count, midi).ok()?;
            let index = arena.insert(midi);

            Some(MidisIndex(index))
        })
    }

    pub fn map_midis<F>(&mut self, index: &MidisIndex, new_count: usize, mut map: F) -> Option<MidisIndex>
    where
        F: FnMut(&[MidiMessage], usize) -> MidiMessage,
    {
        self.store.with_dependent_mut(|alloc, arena| {
            let midis_alloc = arena.get(index.0);

            let new_alloc = if let Some(midis) = midis_alloc {
                alloc.alloc_slice_fill_with(new_count, |idx| map(midis.value, idx)).ok()
            } else {
                None
            };

            new_alloc.map(|x| MidisIndex(arena.insert(x)))
        })
    }

    pub fn borrow_midi(&self, index: &MidisIndex) -> Option<&[MidiMessage]> {
        self.store.borrow_dependent().get(index.0).map(|x| &*x.value)
    }

    pub(super) fn remove_midi(&mut self, index: MidisIndex) -> bool {
        self.store
            .with_dependent_mut(|_, arena| arena.remove(index.0).is_some())
    }

    /// Consumes `self` to block clearing without first destroying
    /// all references
    pub fn clear(mut self) -> Self {
        self.store.with_dependent_mut(|_, arena| arena.clear());

        self
    }
}
