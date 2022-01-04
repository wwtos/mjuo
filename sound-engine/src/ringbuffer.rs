pub struct RingBuffer<T>
where
    T: Clone,
{
    pointer: usize,
    data: Vec<T>,
    size: usize,
}

impl<T> RingBuffer<T>
where
    T: Clone + Copy,
{
    pub fn new(size: usize, initial_value: T) -> RingBuffer<T> {
        RingBuffer {
            pointer: 0,
            data: vec![initial_value; size],
            size,
        }
    }

    pub fn advance(&mut self, amount: usize) {
        self.pointer = (self.pointer + amount) % self.size;
    }

    pub fn push_end(&mut self, value: T) {
        self.advance(1);
        self.data[self.pointer] = value;
    }

    pub fn get(self, relative_index: usize) -> T {
        self.data[(self.pointer + 1 + relative_index) % self.size]
    }
}
