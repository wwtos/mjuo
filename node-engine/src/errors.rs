pub struct Error {
    pub message: String,
    pub error_type: ErrorType,
}

impl Error {
    pub fn new(message: String, error_type: ErrorType) -> Error {
        Error {
            message,
            error_type,
        }
    }
}

pub enum ErrorType {
    AlreadyConnected,
    NodeDoesNotExist,
}
