//! A parser for [apgcode](https://www.conwaylife.com/wiki/Apgcode) format.
//!
//! A parser for [Extended Wechsler format](https://www.conwaylife.com/wiki/Apgcode#Extended_Wechsler_Format)
//! is also provided.

use crate::Coordinates;
use std::str::Bytes;
use thiserror::Error;

/// Errors that can be returned when parsing a Plaintext file.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum Error {
    #[error("Unexpected character: {0}.")]
    UnexpectedChar(char),
}

/// A parser for [Extended Wechsler format](https://www.conwaylife.com/wiki/Apgcode#Extended_Wechsler_Format).
#[derive(Clone, Debug)]
pub struct Wechsler<'a> {
    /// An iterator over bytes of the string.
    bytes: Bytes<'a>,

    /// Coordinates of the current cell.
    position: Coordinates,

    /// The current strip of 5 cells, represented by 1 character in Extended Wechsler format.
    current_strip: u8,

    /// Index of the current cell in the current strip.
    index: u8,
}

impl<'a> Wechsler<'a> {
    /// Creates a new parser instance from a string.
    pub fn new(string: &'a str) -> Self {
        Wechsler {
            bytes: string.bytes(),
            position: (0, 0),
            current_strip: 0,
            index: 5,
        }
    }

    /// An iterator over living cells in a string in Extended Wechsler format.
    pub fn cells<'b>(&'b mut self) -> Cells<'a, 'b> {
        Cells { parser: self }
    }
}

/// An iterator over living cells in a string in Extended Wechsler format.
///
/// The actual implementation of the iterator is inside the `Wechsler` struct.
/// If you want to clone the iterator, please clone the `Wechsler` instead.
#[derive(Debug)]
pub struct Cells<'a, 'b> {
    parser: &'b mut Wechsler<'a>,
}

impl<'a, 'b> Iterator for Cells<'a, 'b> {
    type Item = Result<Coordinates, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parser.next_cell()
    }
}

impl<'a> Wechsler<'a> {
    /// The implementation of the `Cells` iterator.
    fn next_cell<'b>(&'b mut self) -> Option<Result<Coordinates, Error>> {
        loop {
            if self.index < 5 {
                loop {
                    if self.current_strip & 1 << self.index == 0 {
                        self.index += 1;
                        if self.index == 5 {
                            self.position.0 += 1;
                            break;
                        }
                    } else {
                        let cell = (self.position.0, self.position.1 + self.index as i64);
                        self.index += 1;
                        if self.index == 5 {
                            self.position.0 += 1;
                        }
                        return Some(Ok(cell));
                    }
                }
            } else if let Some(c) = self.bytes.next() {
                match c {
                    b'0' => self.position.0 += 1,
                    b'1'..=b'9' => {
                        self.current_strip = c - b'0';
                        self.index = 0;
                    }
                    b'a'..=b'v' => {
                        self.current_strip = c - b'a' + 10;
                        self.index = 0;
                    }
                    b'w' => self.position.0 += 2,
                    b'x' => self.position.0 += 3,
                    b'y' => {
                        if let Some(c) = self.bytes.next() {
                            let n = match c {
                                b'0'..=b'9' => c - b'0',
                                b'a'..=b'z' => c - b'a' + 10,
                                _ => return Some(Err(Error::UnexpectedChar(char::from(c)))),
                            };
                            self.position.0 += 4 + n as i64
                        } else {
                            return Some(Err(Error::UnexpectedChar('y')));
                        }
                    }
                    b'z' => {
                        self.position.0 = 0;
                        self.position.1 += 5;
                    }
                    _ => return Some(Err(Error::UnexpectedChar(char::from(c)))),
                }
            } else {
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wechsler_glider() -> Result<(), Error> {
        const GLIDER: &str = "153";
        let mut glider = Wechsler::new(GLIDER);

        let cells = glider.cells().collect::<Result<Vec<_>, _>>()?;
        assert_eq!(cells, vec![(0, 0), (1, 0), (1, 2), (2, 0), (2, 1)]);
        Ok(())
    }

    #[test]
    fn wechsler_twin_bees_shuttle() -> Result<(), Error> {
        const TWIN_BEE_SHUTTLE: &str = "033y133zzzckgsxsgkczz0cc";
        let mut twin_bees_shuttle = Wechsler::new(TWIN_BEE_SHUTTLE);

        let cells = twin_bees_shuttle.cells().collect::<Result<Vec<_>, _>>()?;
        assert_eq!(
            cells,
            vec![
                (1, 0),
                (1, 1),
                (2, 0),
                (2, 1),
                (8, 0),
                (8, 1),
                (9, 0),
                (9, 1),
                (0, 17),
                (0, 18),
                (1, 17),
                (1, 19),
                (2, 19),
                (3, 17),
                (3, 18),
                (3, 19),
                (7, 17),
                (7, 18),
                (7, 19),
                (8, 19),
                (9, 17),
                (9, 19),
                (10, 17),
                (10, 18),
                (1, 27),
                (1, 28),
                (2, 27),
                (2, 28)
            ]
        );
        Ok(())
    }
}
