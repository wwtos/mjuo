use common::alloc::{BuddyAlloc, SliceAlloc};
use generational_arena::Arena;
use self_cell::self_cell;

use super::OscIndex;

type OscStorage<'a> = Arena<SliceAlloc<'a, u8>>;

self_cell!(
    struct OscStoreInternal {
        owner: BuddyAlloc,

        #[covariant]
        dependent: OscStorage,
    }
);

pub struct OscStore {
    store: OscStoreInternal,
}

impl std::fmt::Debug for OscStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OscStore {{ ")?;
        self.store.borrow_dependent().fmt(f)?;
        write!(f, " }}")
    }
}

impl OscStore {
    pub fn new(alloc_bytes: usize, storage_size: usize) -> OscStore {
        let store = OscStoreInternal::new(BuddyAlloc::new(alloc_bytes), |_| Arena::with_capacity(storage_size));

        OscStore { store }
    }

    pub fn add_osc<F>(&mut self, len: usize, init: F) -> Option<OscIndex>
    where
        F: FnMut(&mut [u8]),
    {
        self.store.with_dependent_mut(|alloc, arena| {
            let osc = alloc.alloc_slice_u8(len, init).ok()?;
            let index = arena.insert(osc);

            Some(OscIndex(index))
        })
    }

    pub fn copy_from(&mut self, slice: &[u8]) -> Option<OscIndex> {
        self.store.with_dependent_mut(|alloc, arena| {
            let osc = alloc.alloc_slice_copy(slice).ok()?;
            let index = arena.insert(osc);

            Some(OscIndex(index))
        })
    }

    pub fn clone_osc(&mut self, index: OscIndex) -> Option<OscIndex> {
        self.store.with_dependent_mut(|alloc, arena| {
            let current_message = arena.get(index.0)?;
            let cloned_message = alloc
                .alloc_slice_fill_iter(current_message.value.iter().cloned())
                .ok()?;

            let index = arena.insert(cloned_message);

            Some(OscIndex(index))
        })
    }

    pub fn borrow_osc(&self, index: &OscIndex) -> Option<&[u8]> {
        self.store.borrow_dependent().get(index.0).map(|x| &*x.value)
    }

    pub(super) fn remove_osc(&mut self, index: OscIndex) -> bool {
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
