use std::error::Error;
use std::fmt::{Display, Formatter};

pub trait HTTPError: std::error::Error {
    fn statuscode(&self) -> i32;
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct StatusCodeError(i32);

impl StatusCodeError {
    pub fn new(code: i32) -> Self {
        Self(code)
    }

    pub fn ok() -> Self {
        Self(200)
    }
}

impl Error for StatusCodeError {}

impl Display for StatusCodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "StatusCodeError{{{}}}", self.0)
    }
}

impl HTTPError for StatusCodeError {
    fn statuscode(&self) -> i32 {
        self.0
    }
}
