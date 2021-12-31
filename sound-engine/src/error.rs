use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeError {
    err: String,
    err_type: NodeErrorType,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeErrorType {
    UnsupportedInput,
    UnsupportedOutput,
}

impl NodeError {
    #[inline]
    pub fn new<T: Into<String>>(t: T, err_type: NodeErrorType) -> NodeError {
        NodeError {
            err: t.into(),
            err_type,
        }
    }
}

impl fmt::Display for NodeError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.err.fmt(f)
    }
}

impl std::error::Error for NodeError {
    #[inline]
    fn description(&self) -> &str {
        &self.err
    }
}
