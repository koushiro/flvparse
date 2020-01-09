// Copyright 2019-2020 koushiro. Licensed under MIT.

/// Type alias to use this library's [`Error`] type in a `Result`.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors generated from this library.
#[derive(Debug)]
pub enum Error {
    /// Parse error.
    Parse,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse => write!(f, "Parse error"),
        }
    }
}
