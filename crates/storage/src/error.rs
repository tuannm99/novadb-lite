use std::error::Error as StdError;
use std::fmt;
use std::io;

/// Categorizes storage-layer failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    /// Wrapped I/O failure from the host filesystem.
    Io,
    /// Buffer access exceeded the valid range.
    OutOfBounds,
    /// On-disk or in-memory page state violated invariants.
    Corruption,
    /// A page operation could not fit additional data.
    NoSpace,
    /// Caller supplied an invalid argument.
    InvalidArgument,
}

/// Rich error type used by the storage crate.
#[derive(Debug)]
pub struct DbError {
    /// High-level error category.
    pub kind: ErrorKind,
    /// Byte offset associated with an out-of-bounds failure.
    pub off: usize,
    /// Requested size associated with an out-of-bounds failure.
    pub size: usize,
    /// Available length associated with an out-of-bounds failure.
    pub len: usize,
    /// Human-readable error message.
    pub msg: String,
    /// Underlying I/O source when `kind` is `Io`.
    pub source: Option<io::Error>,
}

impl DbError {
    pub(crate) fn out_of_bounds(off: usize, size: usize, len: usize) -> Self {
        Self {
            kind: ErrorKind::OutOfBounds,
            off,
            size,
            len,
            msg: String::new(),
            source: None,
        }
    }

    pub(crate) fn corruption(msg: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Corruption,
            off: 0,
            size: 0,
            len: 0,
            msg: msg.into(),
            source: None,
        }
    }

    pub(crate) fn no_space(msg: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::NoSpace,
            off: 0,
            size: 0,
            len: 0,
            msg: msg.into(),
            source: None,
        }
    }

    pub(crate) fn invalid_argument(msg: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::InvalidArgument,
            off: 0,
            size: 0,
            len: 0,
            msg: msg.into(),
            source: None,
        }
    }

    pub(crate) fn wrap_io(err: io::Error) -> Self {
        Self {
            kind: ErrorKind::Io,
            off: 0,
            size: 0,
            len: 0,
            msg: String::new(),
            source: Some(err),
        }
    }
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ErrorKind::Io => match &self.source {
                Some(err) => write!(f, "io error: {err}"),
                None => write!(f, "io error"),
            },
            ErrorKind::OutOfBounds => {
                write!(
                    f,
                    "out of bounds: off={} size={} len={}",
                    self.off, self.size, self.len
                )
            }
            ErrorKind::Corruption => write!(f, "corruption: {}", self.msg),
            ErrorKind::NoSpace => write!(f, "no space: {}", self.msg),
            ErrorKind::InvalidArgument => write!(f, "invalid args: {}", self.msg),
        }
    }
}

impl StdError for DbError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_ref()
            .map(|err| err as &(dyn StdError + 'static))
    }
}

/// Standard result type for storage operations.
pub type Result<T> = std::result::Result<T, DbError>;
