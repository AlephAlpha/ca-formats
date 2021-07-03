//! A parser for [Plaintext](https://www.conwaylife.com/wiki/Plaintext) format.

use crate::{Coordinates, Input};
use displaydoc::Display;
use std::io::{BufReader, Error as IoError, Read};
use thiserror::Error;

/// Errors that can be returned when parsing a Plaintext file.
#[derive(Debug, Error, Display)]
pub enum Error {
    /// Unexpected character: {0}.
    UnexpectedChar(char),
    /// Error when reading from input: {0}.
    IoError(#[from] IoError),
}

/// A parser for [Plaintext](https://www.conwaylife.com/wiki/Plaintext) format.
///
/// As an iterator, it iterates over the living cells.
///
/// # Examples
///
/// ## Reading from a string:
///
/// ```rust
/// use ca_formats::plaintext::Plaintext;
///
/// const GLIDER: &str = r"! Glider
/// !
/// .O.
/// ..O
/// OOO";
///
/// let glider = Plaintext::new(GLIDER).unwrap();
///
/// let cells = glider.map(|cell| cell.unwrap()).collect::<Vec<_>>();
/// assert_eq!(cells, vec![(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)]);
/// ```
///
/// ## Reading from a file:
///
/// ``` rust
/// use std::fs::File;
/// use ca_formats::plaintext::Plaintext;
///
/// let file = File::open("tests/sirrobin.cells").unwrap();
/// let sirrobin = Plaintext::new_from_file(file).unwrap();
///
/// assert_eq!(sirrobin.count(), 282);
/// ```
#[must_use]
#[derive(Debug)]
pub struct Plaintext<I: Input> {
    /// An iterator over lines of a Plaintext file.
    lines: I::Lines,

    /// An iterator over bytes of the current line.
    current_line: Option<I::Bytes>,

    /// Coordinates of the current cell.
    position: Coordinates,
}

impl<I: Input> Plaintext<I> {
    /// Creates a new parser instance from input.
    pub fn new(input: I) -> Result<Self, Error> {
        let mut lines = input.lines();
        let mut current_line = None;
        while let Some(item) = lines.next() {
            let line = I::line(item)?;
            if !line.as_ref().starts_with('!') {
                current_line = Some(I::bytes(line));
                break;
            }
        }
        Ok(Self {
            lines,
            current_line,
            position: (0, 0),
        })
    }
}

impl<I, L> Plaintext<I>
where
    I: Input<Lines = L>,
    L: Input,
{
    /// Parse the remaining unparsed lines as a new Plaintext.
    pub fn remains(self) -> Result<Plaintext<L>, Error> {
        Plaintext::new(self.lines)
    }
}

impl<R: Read> Plaintext<BufReader<R>> {
    /// Creates a new parser instance from something that implements [`Read`] trait, e.g., a [`File`](std::fs::File).
    pub fn new_from_file(file: R) -> Result<Self, Error> {
        Self::new(BufReader::new(file))
    }
}

impl<I: Input> Clone for Plaintext<I>
where
    I::Lines: Clone,
    I::Bytes: Clone,
{
    fn clone(&self) -> Self {
        Self {
            lines: self.lines.clone(),
            current_line: self.current_line.clone(),
            position: self.position,
        }
    }
}

/// An iterator over living cells in a Plaintext file.
impl<I: Input> Iterator for Plaintext<I> {
    type Item = Result<Coordinates, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(c) = self.current_line.as_mut().and_then(Iterator::next) {
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
            } else if let Some(item) = self.lines.next() {
                match I::line(item) {
                    Ok(line) => {
                        if line.as_ref().starts_with('!') {
                            continue;
                        } else {
                            self.position.0 = 0;
                            self.position.1 += 1;
                            self.current_line = Some(I::bytes(line));
                        }
                    }
                    Err(e) => {
                        return Some(Err(Error::IoError(e)));
                    }
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

        let glider = Plaintext::new(GLIDER)?;

        let _ = glider.clone();

        let cells = glider.collect::<Result<Vec<_>, _>>()?;
        assert_eq!(cells, vec![(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)]);
        Ok(())
    }
}
