use std::{
    ops::DerefMut,
    sync::{Mutex, TryLockError},
};

#[cfg(test)]
use std::{sync::Arc, thread, time::Duration};

pub struct DoubleBuffer<T> {
    pub active: Mutex<T>,
    pub background: Mutex<T>,
}

#[derive(Debug)]
pub struct WouldBlock;

impl<T> DoubleBuffer<T> {
    pub fn new(buffer_left: T, buffer_right: T) -> DoubleBuffer<T> {
        DoubleBuffer {
            active: Mutex::new(buffer_left),
            background: Mutex::new(buffer_right),
        }
    }
}

impl<T> DoubleBuffer<T> {
    pub fn swap(&self) {
        // ensure we have access to these
        let mut lock_active = self.active.lock().unwrap();
        let mut lock_background = self.background.lock().unwrap();

        std::mem::swap(lock_active.deref_mut(), lock_background.deref_mut());
    }

    pub fn try_swap(&self) -> Result<(), WouldBlock> {
        // ensure we have access to these
        let mut lock_active = self.active.try_lock().map_err(|err| match err {
            TryLockError::Poisoned(_) => panic!("poisoned lock"),
            TryLockError::WouldBlock => WouldBlock,
        })?;
        let mut lock_background = self.background.try_lock().map_err(|err| match err {
            TryLockError::Poisoned(_) => panic!("poisoned lock"),
            TryLockError::WouldBlock => WouldBlock,
        })?;

        std::mem::swap(lock_active.deref_mut(), lock_background.deref_mut());

        Ok(())
    }
}

pub fn double_buffer<T>(buffer_a: T, buffer_b: T) -> DoubleBuffer<T>
where
    T: Send + Sync,
{
    DoubleBuffer::new(buffer_a, buffer_b)
}

#[test]
fn test_swap_block() {
    let double_buffer = Arc::new(double_buffer(vec![1, 2, 3], vec![4, 5, 6]));
    let double_buffer_clone = double_buffer.clone();

    let active_lock = (*double_buffer_clone).active.lock().unwrap();
    let active_lock_ref = active_lock.as_ref();

    thread::scope(|s| {
        s.spawn(|| {
            // make sure we can still access the background buffer
            assert!((*double_buffer).background.try_lock().is_ok());

            // now, swapping it should fail
            // (*double_buffer).try_swap();
            assert!((*double_buffer).try_swap().is_err());
        });
    });

    // keep left lock in scope
    assert_eq!(active_lock_ref, vec![1, 2, 3]);
}

#[test]
fn test_swap_lock() {
    let double_buffer = Arc::new(double_buffer(vec![1, 2, 3], vec![4, 5, 6]));
    let double_buffer_clone = double_buffer.clone();

    thread::spawn(move || {
        // lock the right and do something intensive
        let mut background_lock = (*double_buffer_clone).background.lock().unwrap();
        thread::sleep(Duration::from_millis(100));

        background_lock[0] = 7;
        background_lock[1] = 8;
        background_lock[2] = 9;
    });

    // give the other thread a second to lock
    thread::sleep(Duration::from_millis(10));

    // swapping should block
    (*double_buffer).swap();

    assert_eq!((*double_buffer).active.lock().unwrap().as_ref(), vec![7, 8, 9]);
}
