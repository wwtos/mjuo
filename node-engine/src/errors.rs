#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum ErrorType {
    AlreadyConnected,
    NotConnected,
    NodeDoesNotExist,
    IndexOutOfBounds,
    SocketDoesNotExist,
    IncompatibleSocketTypes,
    ParserError
}
