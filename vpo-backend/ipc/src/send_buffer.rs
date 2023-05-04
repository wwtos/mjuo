use std::sync::Mutex;

pub struct SendBuffer<T> {
    pub buffer: Mutex<Vec<T>>,
}

impl<T> SendBuffer<T> {
    pub fn send(&self, value: T) -> Result<(), ()> {
        self.buffer.lock().unwrap().push(value);

        Ok(())
    }
}
