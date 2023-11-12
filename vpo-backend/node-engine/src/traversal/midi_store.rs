use common::alloc::{BuddyArena, SliceAlloc};
use generational_arena::{Arena, Index};
use self_cell::self_cell;
use sound_engine::midi::messages::MidiMessage;

type MidiStorage<'a> = Arena<SliceAlloc<'a, MidiMessage>>;

self_cell!(
    struct MidiStoreInternal {
        owner: BuddyArena,

        #[covariant]
        dependent: MidiStorage,
    }
);

pub struct MidiStore {
    store: MidiStoreInternal,
}

impl MidiStore {
    pub fn new(alloc_bytes: usize, storage_size: usize) -> MidiStore {
        let store = MidiStoreInternal::new(BuddyArena::new(alloc_bytes), |_| Arena::with_capacity(storage_size));

        MidiStore { store }
    }

    pub fn add_midi(&mut self, midi: MidiMessage) -> Option<Index> {
        self.store.with_dependent_mut(|alloc, arena| {
            let midi = alloc.alloc_slice_fill_with(1, |_| midi.clone()).ok()?;
            let index = arena.insert(midi);

            Some(index)
        })
    }

    pub fn remove_midi(&mut self, index: Index) -> bool {
        self.store.with_dependent_mut(|_, arena| arena.remove(index).is_some())
    }
}
