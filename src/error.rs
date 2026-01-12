use std::fmt;

/// Common error type for the engine.
#[derive(Debug)]
pub enum DbError {
    Io(std::io::Error),
    /// Buffer access went out of bounds.
    OutOfBounds {
        off: usize,
        size: usize,
        len: usize,
    },
    /// Data corruption or invariant violation.
    Corruption(&'static str),
    NoSpace(&'static str),
    InvalidArgument(&'static str),
}

impl From<std::io::Error> for DbError {
    fn from(err: std::io::Error) -> Self {
        DbError::Io(err)
    }
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbError::Io(e) => write!(f, "io error: {}", e),
            DbError::OutOfBounds { off, size, len } => {
                write!(f, "out of bounds: off={} size={} len={}", off, size, len)
            }
            DbError::Corruption(msg) => write!(f, "corruption: {}", msg),
            DbError::NoSpace(msg) => write!(f, "no space: {}", msg),
            DbError::InvalidArgument(msg) => write!(f, "invalid args: {}", msg),
        }
    }
}

impl std::error::Error for DbError {}

pub type DbResult<T> = Result<T, DbError>;
