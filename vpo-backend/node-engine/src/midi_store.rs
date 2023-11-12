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

    pub fn add_midi<I>(&mut self, midis: I) -> Option<Index>
    where
        I: IntoIterator<Item = MidiMessage>,
        I::IntoIter: ExactSizeIterator,
    {
        self.store.with_dependent_mut(|alloc, arena| {
            let midi = alloc.alloc_slice_fill_iter(midis).ok()?;
            let index = arena.insert(midi);

            Some(index)
        })
    }

    pub fn add_midi_with<F>(&mut self, count: usize, midi: F) -> Option<Index>
    where
        F: FnMut(usize) -> MidiMessage,
    {
        self.store.with_dependent_mut(|alloc, arena| {
            let midi = alloc.alloc_slice_fill_with(count, midi).ok()?;
            let index = arena.insert(midi);

            Some(index)
        })
    }

    pub fn map_midis<F>(&mut self, index: Index, new_count: usize, mut map: F) -> Option<Index>
    where
        F: FnMut(&[MidiMessage], usize) -> MidiMessage,
    {
        self.store.with_dependent_mut(|alloc, arena| {
            let midis_alloc = arena.get(index);

            let new_alloc = if let Some(midis) = midis_alloc {
                alloc.alloc_slice_fill_with(new_count, |idx| map(midis.value, idx)).ok()
            } else {
                None
            };

            new_alloc.map(|x| arena.insert(x))
        })
    }

    pub fn borrow_midi(&self, index: Index) -> Option<&[MidiMessage]> {
        self.store.borrow_dependent().get(index).map(|x| &*x.value)
    }

    pub fn remove_midi(&mut self, index: Index) -> bool {
        self.store.with_dependent_mut(|_, arena| arena.remove(index).is_some())
    }
}
