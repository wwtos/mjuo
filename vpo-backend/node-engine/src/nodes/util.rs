#[derive(Debug, Clone)]
pub enum ProcessState<T> {
    Unprocessed(T),
    Processed,
    None,
}

impl<T> ProcessState<T> {
    pub fn as_unprocessed(self) -> Option<T> {
        match self {
            ProcessState::Unprocessed(value) => Some(value),
            _ => None,
        }
    }
}
