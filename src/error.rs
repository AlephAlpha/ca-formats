//! Errors that can be returned when parsing a pattern file.

use thiserror::Error;

/// Errors that can be returned when parsing a pattern file.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum Error {
    #[error("Unexpected byte: {0:#x}.")]
    UnexpectedByte(u8),
    #[error("apgcode not encoded in extended Wechsler format")]
    Unencodable,
}
