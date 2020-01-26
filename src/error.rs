use std::{
    error,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    UnexpectedByte(u8),
    Unencodable,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnexpectedByte(c) => write!(f, "Unexpected byte: {:#x}", c),
            Error::Unencodable => write!(f, "apgcode not in extended Wechsler format."),
        }
    }
}

impl error::Error for Error {}
