//! Errors that can be returned when parsing a pattern file.

use std::{
    error,
    fmt::{self, Display, Formatter},
};

/// Errors that can be returned when parsing a pattern file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// Unexpected byte.
    UnexpectedByte(u8),
    /// apgcode not encoded in extended Wechsler format.
    Unencodable,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnexpectedByte(c) => write!(f, "Unexpected byte: {:#x}", c),
            Error::Unencodable => write!(f, "apgcode not encoded in extended Wechsler format."),
        }
    }
}

impl error::Error for Error {}
