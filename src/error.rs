// Copyright 2019-2020 koushiro. Licensed under MIT.

/// Type alias to use this library's [`Error`] type in a `Result`.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors generated from this library.
#[derive(Debug)]
pub enum Error {
    /// Io error.
    Io(std::io::Error),
    /// Parse error.
    Parse,
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(err) => write!(f, "{}", err),
            Error::Parse => write!(f, "Parse error"),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}
