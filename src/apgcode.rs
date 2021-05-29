//! Parsers for [apgcode](https://www.conwaylife.com/wiki/Apgcode) format
//! and [Extended Wechsler format](https://www.conwaylife.com/wiki/Apgcode#Extended_Wechsler_Format).

use crate::Coordinates;
use std::str::Bytes;
use thiserror::Error;

/// Errors that can be returned when parsing a apgcode string.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum Error {
    #[error("Unexpected character: {0}.")]
    UnexpectedChar(char),
    #[error("Pattern not encoded in extended Wechsler format")]
    Unencodable,
}

/// A parser for [Extended Wechsler format](https://www.conwaylife.com/wiki/Apgcode#Extended_Wechsler_Format).
///
/// Extended Wechsler format is the part of apgcode that encodes the cells in the pattern,
/// e.g. `153` in `xq4_153`.
///
/// As an iterator, it iterates over the living cells.
///
/// Reading from files is not supported, since the format is usually short and not stored in a file.
///
/// # Example
///
/// ```rust
/// use ca_formats::apgcode::Wechsler;
///
/// const GLIDER: &str = "153";
///
/// let glider = Wechsler::new(GLIDER);
///
/// let cells = glider.map(|cell| cell.unwrap()).collect::<Vec<_>>();
/// assert_eq!(cells, vec![(0, 0), (1, 0), (1, 2), (2, 0), (2, 1)]);
/// ```
#[must_use]
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
}

/// An iterator over living cells in a string in Extended Wechsler format.
impl<'a> Iterator for Wechsler<'a> {
    type Item = Result<Coordinates, Error>;

    fn next(&mut self) -> Option<Self::Item> {
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

/// Type of a pattern.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum PatternType {
    /// [Still life](https://conwaylife.com/wiki/Still_life).
    StillLife,
    /// [Oscillator](https://conwaylife.com/wiki/Oscillator).
    Oscillator,
    /// [Spaceship](https://conwaylife.com/wiki/Spaceship).
    Spaceship,
}

/// A parser for [apgcode](https://www.conwaylife.com/wiki/Apgcode) format.
///
/// Only supports patterns that are encoded in Extended Wechsler format,
/// i.e., still lifes, oscillators, spaceships.
/// Rules with more than 2 states are not yet supported.
///
/// As an iterator, it iterates over the living cells.
///
/// Reading from files is not supported, since apgcode is usually short and not stored in a file.
///
/// # Example
///
/// ```rust
/// use ca_formats::apgcode::{ApgCode, PatternType};
///
/// const GLIDER: &str = "xq4_153";
///
/// let glider = ApgCode::new(GLIDER).unwrap();
/// assert_eq!(glider.pattern_type(), PatternType::Spaceship);
/// assert_eq!(glider.period(), 4);
///
/// let cells = glider.map(|cell| cell.unwrap()).collect::<Vec<_>>();
/// assert_eq!(cells, vec![(0, 0), (1, 0), (1, 2), (2, 0), (2, 1)]);
/// ```
#[must_use]
#[derive(Clone, Debug)]
pub struct ApgCode<'a> {
    pattern_type: PatternType,
    period: u64,
    wechsler: Wechsler<'a>,
}

impl<'a> ApgCode<'a> {
    /// Creates a new parser instance from a string.
    pub fn new(string: &'a str) -> Result<Self, Error> {
        let mut split = string.split('_');
        let prefix = split.next().ok_or(Error::Unencodable)?;
        if prefix[2..].bytes().any(|c| !c.is_ascii_digit()) {
            return Err(Error::Unencodable);
        }
        let pattern_type = match &prefix[..2] {
            "xs" => PatternType::StillLife,
            "xp" => PatternType::Oscillator,
            "xq" => PatternType::Spaceship,
            _ => return Err(Error::Unencodable),
        };
        let period = if pattern_type == PatternType::StillLife {
            1
        } else {
            prefix[2..].parse().map_err(|_| Error::Unencodable)?
        };
        let wechsler_string = split.next().ok_or(Error::Unencodable)?;
        let wechsler = Wechsler::new(wechsler_string);
        Ok(ApgCode {
            pattern_type,
            period,
            wechsler,
        })
    }

    /// Period of the pattern.
    pub const fn period(&self) -> u64 {
        self.period
    }

    /// Type of the pattern.
    pub const fn pattern_type(&self) -> PatternType {
        self.pattern_type
    }
}

/// An iterator over living cells in an apgcode string.
impl<'a> Iterator for ApgCode<'a> {
    type Item = Result<Coordinates, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.wechsler.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wechsler_glider() -> Result<(), Error> {
        const GLIDER: &str = "153";
        let glider = Wechsler::new(GLIDER);

        let _ = glider.clone();

        let cells = glider.collect::<Result<Vec<_>, _>>()?;
        assert_eq!(cells, vec![(0, 0), (1, 0), (1, 2), (2, 0), (2, 1)]);
        Ok(())
    }

    #[test]
    fn wechsler_twin_bees_shuttle() -> Result<(), Error> {
        const TWIN_BEE_SHUTTLE: &str = "033y133zzzckgsxsgkczz0cc";
        let twin_bees_shuttle = Wechsler::new(TWIN_BEE_SHUTTLE);

        let cells = twin_bees_shuttle.collect::<Result<Vec<_>, _>>()?;
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

    #[test]
    fn apgcode_glider() -> Result<(), Error> {
        const GLIDER: &str = "xq4_153";
        let glider = ApgCode::new(GLIDER)?;

        let _ = glider.clone();

        assert_eq!(glider.pattern_type(), PatternType::Spaceship);
        assert_eq!(glider.period(), 4);

        let cells = glider.collect::<Result<Vec<_>, _>>()?;
        assert_eq!(cells, vec![(0, 0), (1, 0), (1, 2), (2, 0), (2, 1)]);
        Ok(())
    }
}
