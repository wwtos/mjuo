use core::slice;
use std::{
    alloc::Layout,
    cell::UnsafeCell,
    fmt::Debug,
    mem::size_of,
    ptr::{self, NonNull},
};

use allocator_api2::alloc::AllocError;
use buddy_system_allocator::Heap;

pub struct Alloc<'a, T> {
    pub value: &'a mut T,
    buddy_ref: &'a BuddyAlloc,
    ptr: NonNull<u8>,
    layout: Layout,
}

impl<'a, T: Debug> Debug for Alloc<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Allocation {{ ")?;
        self.value.fmt(f)?;
        write!(f, " }}")
    }
}

impl<'a, T> Drop for Alloc<'a, T> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(self.value as *mut T);
        }

        unsafe { &mut *self.buddy_ref.heap.get() }.dealloc(self.ptr, self.layout);
    }
}

pub struct SliceAlloc<'a, T> {
    pub value: &'a mut [T],
    buddy_ref: &'a BuddyAlloc,
    ptr: NonNull<u8>,
    layout: Layout,
}

impl<'a, T: Debug> Debug for SliceAlloc<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SliceAllocation {{ ")?;
        self.value.fmt(f)?;
        write!(f, " }}")
    }
}

impl<'a, T> Drop for SliceAlloc<'a, T> {
    fn drop(&mut self) {
        for x in self.value.iter_mut() {
            unsafe {
                ptr::drop_in_place(x as *mut T);
            }
        }

        unsafe { &mut *self.buddy_ref.heap.get() }.dealloc(self.ptr, self.layout);
    }
}

pub struct BuddyAlloc {
    _space: Vec<usize>,
    heap: UnsafeCell<Heap<32>>,
}

impl BuddyAlloc {
    pub fn new(bytes: usize) -> BuddyAlloc {
        let size = bytes / size_of::<usize>();

        let mut heap = Heap::<32>::new();
        let space = vec![0; size];

        unsafe {
            heap.init(space.as_slice().as_ptr() as usize, size * size_of::<usize>());
        }

        BuddyAlloc {
            _space: space,
            heap: UnsafeCell::new(heap),
        }
    }

    // most methods from here on out are adapted from the fantastic `bumpalo` crate

    pub fn alloc_with<'a, T, F>(&'a self, f: F) -> Result<Alloc<'a, T>, AllocError>
    where
        F: FnOnce() -> T,
    {
        let layout = Layout::new::<T>();

        let alloc = unsafe { self.heap_ref() }.alloc(layout).map_err(|_| AllocError)?;
        let p = alloc.as_ptr() as *mut T;

        let ref_to_x = unsafe {
            ptr::write(p, f());

            &mut *p
        };

        Ok(Alloc {
            value: ref_to_x,
            buddy_ref: &self,
            ptr: alloc,
            layout,
        })
    }

    pub fn alloc_slice_copy<T>(&self, src: &[T]) -> Result<SliceAlloc<'_, T>, AllocError>
    where
        T: Copy,
    {
        let layout = Layout::for_value(src);
        let ptr = unsafe { self.heap_ref() }.alloc(layout).map_err(|_| AllocError)?;
        let dst = ptr.cast::<T>();

        unsafe {
            ptr::copy_nonoverlapping(src.as_ptr(), dst.as_ptr(), src.len());
            Ok(SliceAlloc {
                value: slice::from_raw_parts_mut(dst.as_ptr(), src.len()),
                buddy_ref: &self,
                ptr: ptr,
                layout: layout,
            })
        }
    }

    pub fn alloc_slice_fill_with<T, F>(&self, len: usize, mut f: F) -> Result<SliceAlloc<'_, T>, AllocError>
    where
        F: FnMut(usize) -> T,
    {
        let layout = Layout::array::<T>(len).map_err(|_| AllocError)?;

        let ptr = unsafe { self.heap_ref() }.alloc(layout).map_err(|_| AllocError)?;
        let dst = ptr.cast::<T>();

        unsafe {
            for i in 0..len {
                ptr::write(dst.as_ptr().add(i), f(i));
            }

            let result = slice::from_raw_parts_mut(dst.as_ptr(), len);
            debug_assert_eq!(Layout::for_value(result), layout);

            Ok(SliceAlloc {
                value: result,
                buddy_ref: &self,
                ptr,
                layout,
            })
        }
    }

    pub fn alloc_slice_fill_iter<T, I>(&self, iter: I) -> Result<SliceAlloc<'_, T>, AllocError>
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        let mut iter = iter.into_iter();
        self.alloc_slice_fill_with(iter.len(), |_| iter.next().expect("Iterator supplied too few elements"))
    }

    unsafe fn heap_ref(&self) -> &mut Heap<32> {
        &mut *self.heap.get()
    }
}

#[test]
fn test_alloc() {
    let arena = BuddyAlloc::new(1_000_000);

    let hello = arena.alloc_with(|| "hello").unwrap();
    let world = arena.alloc_with(|| "world").unwrap();
    let arr = arena.alloc_with(|| [hello, world]).unwrap();

    println!("{:?}", arr);
    println!("{:?}", unsafe { arena.heap_ref() });

    println!("{:?}", unsafe { arena.heap_ref() });

    drop(arr);

    println!("done");
    println!("{:?}", unsafe { arena.heap_ref() });
}
