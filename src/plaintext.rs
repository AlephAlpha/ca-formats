//! A parser for [Plaintext](https://www.conwaylife.com/wiki/Plaintext) format.

use crate::Coordinates;
use std::str::{Bytes, Lines};
use thiserror::Error;

/// Errors that can be returned when parsing a Plaintext file.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum Error {
    #[error("Unexpected character: {0}.")]
    UnexpectedChar(char),
}

/// A parser for [Plaintext](https://www.conwaylife.com/wiki/Plaintext) format.
///
/// # Example
///
/// ```rust
/// # use ca_formats::plaintext::Plaintext;
///
/// const GLIDER: &str = r"! Glider
/// !
/// .O.
/// ..O
/// OOO";
///
/// let mut glider = Plaintext::new(GLIDER);
///
/// let cells = glider
///     .cells()
///     .map(|cell| cell.unwrap())
///     .collect::<Vec<_>>();
/// assert_eq!(cells, vec![(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)]);
/// ```
#[derive(Clone, Debug)]
pub struct Plaintext<'a> {
    /// An iterator over lines of the Plaintext string.
    lines: Lines<'a>,

    /// An iterator over bytes of the current line.
    current_line: Bytes<'a>,

    /// Coordinates of the current cell.
    position: Coordinates,
}

impl<'a> Plaintext<'a> {
    /// Creates a new parser instance from a string.
    pub fn new(string: &'a str) -> Self {
        let mut lines = string.lines();
        let mut current_line = "".bytes();
        while let Some(line) = lines.next() {
            if !line.starts_with('!') {
                current_line = line.bytes();
                break;
            }
        }
        Plaintext {
            lines,
            current_line,
            position: (0, 0),
        }
    }

    /// An iterator over living cells in a Plaintext file.
    pub fn cells<'b>(&'b mut self) -> Cells<'a, 'b> {
        Cells { parser: self }
    }
}

/// An iterator over living cells in a Plaintext file.
///
/// The actual implementation of the iterator is inside the `Plaintext` struct.
/// If you want to clone the iterator, please clone the `Plaintext` instead.
#[derive(Debug)]
pub struct Cells<'a, 'b> {
    parser: &'b mut Plaintext<'a>,
}

impl<'a, 'b> Iterator for Cells<'a, 'b> {
    type Item = Result<Coordinates, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parser.next_cell()
    }
}

impl<'a> Plaintext<'a> {
    /// The implementation of the `Cells` iterator.
    fn next_cell<'b>(&'b mut self) -> Option<Result<Coordinates, Error>> {
        loop {
            if let Some(c) = self.current_line.next() {
                match c {
                    b'O' | b'*' => {
                        let cell = self.position;
                        self.position.0 += 1;
                        return Some(Ok(cell));
                    }
                    b'.' => self.position.0 += 1,
                    _ if c.is_ascii_whitespace() => continue,
                    _ => return Some(Err(Error::UnexpectedChar(char::from(c)))),
                }
            } else if let Some(l) = self.lines.next() {
                if l.starts_with('!') {
                    continue;
                } else {
                    self.position.0 = 0;
                    self.position.1 += 1;
                    self.current_line = l.bytes();
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
    fn plaintext_glider() -> Result<(), Error> {
        const GLIDER: &str = r"!Name: Glider
!
.O.
..O
OOO";

        let mut glider = Plaintext::new(GLIDER);

        let cells = glider.cells().collect::<Result<Vec<_>, _>>()?;
        assert_eq!(cells, vec![(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)]);
        Ok(())
    }
}