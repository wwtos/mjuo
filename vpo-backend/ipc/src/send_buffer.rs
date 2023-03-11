use std::sync::Mutex;

pub struct SendBuffer<T> {
    pub buffer: Mutex<Vec<T>>,
}

impl<T> SendBuffer<T> {
    pub async fn send(&self, value: T) {
        self.buffer.lock().unwrap().push(value);
    }
}
