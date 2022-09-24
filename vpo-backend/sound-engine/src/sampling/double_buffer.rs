use std::{
    cell::UnsafeCell,
    sync::{Arc, Mutex, MutexGuard, PoisonError, TryLockError},
    time::Duration,
};

#[cfg(test)]
use std::thread;

pub struct DoubleBuffer<T> {
    buffer_left: Mutex<BufferWrapper<T>>,
    buffer_right: Mutex<BufferWrapper<T>>,
}

impl<T> DoubleBuffer<T> {
    pub fn new(buffer_left: T, buffer_right: T) -> DoubleBuffer<T> {
        DoubleBuffer {
            buffer_left: Mutex::new(BufferWrapper(buffer_left)),
            buffer_right: Mutex::new(BufferWrapper(buffer_right)),
        }
    }
}

pub trait BufferHalf<T>
where
    T: Send + Sync,
{
    fn lock(&self) -> Result<MutexGuard<'_, BufferWrapper<T>>, PoisonError<MutexGuard<'_, BufferWrapper<T>>>>;

    fn try_lock(&self) -> Result<MutexGuard<'_, BufferWrapper<T>>, TryLockError<MutexGuard<'_, BufferWrapper<T>>>>;

    fn swap(&mut self) -> Result<(), PoisonError<MutexGuard<'_, BufferWrapper<T>>>>;

    fn try_swap(&mut self) -> Result<(), TryLockError<MutexGuard<'_, BufferWrapper<T>>>>;
}

pub struct BufferWrapper<T>(T);

pub struct BufferLeft<T>
where
    T: Send + Sync,
{
    packet: UnsafeCell<Arc<DoubleBuffer<T>>>,
}

pub struct BufferRight<T>
where
    T: Send + Sync,
{
    packet: UnsafeCell<Arc<DoubleBuffer<T>>>,
}

impl<T> BufferLeft<T>
where
    T: Send + Sync,
{
    pub fn new(packet: Arc<DoubleBuffer<T>>) -> BufferLeft<T> {
        BufferLeft {
            packet: UnsafeCell::new(packet),
        }
    }
}

impl<T> BufferRight<T>
where
    T: Send + Sync,
{
    pub fn new(packet: Arc<DoubleBuffer<T>>) -> BufferRight<T> {
        BufferRight {
            packet: UnsafeCell::new(packet),
        }
    }
}

impl<T> BufferHalf<T> for BufferLeft<T>
where
    T: Send + Sync,
{
    fn lock(&self) -> Result<MutexGuard<'_, BufferWrapper<T>>, PoisonError<MutexGuard<'_, BufferWrapper<T>>>> {
        unsafe { (*self.packet.get()).buffer_left.lock() }
    }

    fn try_lock(&self) -> Result<MutexGuard<'_, BufferWrapper<T>>, TryLockError<MutexGuard<'_, BufferWrapper<T>>>> {
        unsafe { (*self.packet.get()).buffer_left.try_lock() }
    }

    fn swap(&mut self) -> Result<(), PoisonError<MutexGuard<'_, BufferWrapper<T>>>> {
        unsafe {
            // ensure we have access to these
            let mut lock_left = (*self.packet.get()).buffer_left.lock()?;
            let mut lock_right = (*self.packet.get()).buffer_right.lock()?;

            std::mem::swap(&mut lock_left.0, &mut lock_right.0);
        }

        Ok(())
    }

    fn try_swap(&mut self) -> Result<(), TryLockError<MutexGuard<'_, BufferWrapper<T>>>> {
        unsafe {
            // ensure we have access to these
            let mut lock_left = (*self.packet.get()).buffer_left.try_lock()?;
            let mut lock_right = (*self.packet.get()).buffer_right.try_lock()?;

            std::mem::swap(&mut lock_left.0, &mut lock_right.0);
        }

        Ok(())
    }
}

impl<T> BufferHalf<T> for BufferRight<T>
where
    T: Send + Sync,
{
    fn lock(&self) -> Result<MutexGuard<'_, BufferWrapper<T>>, PoisonError<MutexGuard<'_, BufferWrapper<T>>>> {
        unsafe { (*self.packet.get()).buffer_right.lock() }
    }

    fn try_lock(&self) -> Result<MutexGuard<'_, BufferWrapper<T>>, TryLockError<MutexGuard<'_, BufferWrapper<T>>>> {
        unsafe { (*self.packet.get()).buffer_right.try_lock() }
    }

    fn swap(&mut self) -> Result<(), PoisonError<MutexGuard<'_, BufferWrapper<T>>>> {
        unsafe {
            // ensure we have access to these
            let mut lock_left = (*self.packet.get()).buffer_left.lock()?;
            let mut lock_right = (*self.packet.get()).buffer_right.lock()?;

            std::mem::swap(&mut lock_left.0, &mut lock_right.0);
        }

        Ok(())
    }

    fn try_swap(&mut self) -> Result<(), TryLockError<MutexGuard<'_, BufferWrapper<T>>>> {
        unsafe {
            // ensure we have access to these
            let mut lock_left = (*self.packet.get()).buffer_left.try_lock()?;
            let mut lock_right = (*self.packet.get()).buffer_right.try_lock()?;

            std::mem::swap(&mut lock_left.0, &mut lock_right.0);
        }

        Ok(())
    }
}

unsafe impl<T> Send for BufferLeft<T> where T: Send + Sync {}
unsafe impl<T> Sync for BufferLeft<T> where T: Send + Sync {}

unsafe impl<T> Send for BufferRight<T> where T: Send + Sync {}
unsafe impl<T> Sync for BufferRight<T> where T: Send + Sync {}

pub fn double_buffer<T>(buffer_a: T, buffer_b: T) -> (BufferLeft<T>, BufferRight<T>)
where
    T: Send + Sync,
{
    let new_packet = Arc::new(DoubleBuffer::new(buffer_a, buffer_b));

    (BufferLeft::new(new_packet.clone()), BufferRight::new(new_packet))
}

#[test]
fn test_swap_block() {
    let (left, mut right) = double_buffer(vec![1, 2, 3], vec![4, 5, 6]);

    let left_lock = left.lock().unwrap();
    assert_eq!(left_lock.0, vec![1, 2, 3]);

    thread::scope(|s| {
        s.spawn(|| {
            // make sure we can still access the right buffer
            assert!(right.try_lock().is_ok());

            // now, swapping it should fail
            assert!(right.try_swap().is_err());
        });
    });

    // keep left lock in scope
    assert_eq!(left_lock.0, vec![1, 2, 3]);
}

#[test]
fn test_swap_lock() {
    let (mut left, right) = double_buffer(vec![1, 2, 3], vec![4, 5, 6]);

    thread::spawn(move || {
        // lock the right and do something intensive
        let mut right_lock = right.lock().unwrap();
        thread::sleep(Duration::from_millis(100));

        right_lock.0[0] = 7;
        right_lock.0[1] = 8;
        right_lock.0[2] = 9;
    });

    // give the other thread a second to lock
    thread::sleep(Duration::from_millis(10));

    // swapping should block
    left.swap().unwrap();

    assert_eq!(left.lock().unwrap().0, vec![7, 8, 9]);
}
