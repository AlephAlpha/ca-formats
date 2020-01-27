//! A parser for [RLE](https://www.conwaylife.com/wiki/Run_Length_Encoded) format.
//!
//! Rules with more than 2 states are not supported.
//!
//! # Example
//!
//! ```rust
//! use ca_formats::rle::RLE;
//!
//! const GLIDER: &str = r"#N Glider
//! #O Richard K. Guy
//! #C The smallest, most common, and first discovered spaceship. Diagonal, has period 4 and speed c/4.
//! #C www.conwaylife.com/wiki/index.php?title=Glider
//! x = 3, y = 3, rule = B3/S23
//! bob$2bo$3o!";
//! let mut glider = RLE::new(GLIDER).collect::<Result<Vec<_>, _>>().unwrap();
//! assert_eq!(glider, vec![(0, 1), (1, 2), (2, 0), (2, 1), (2, 2)]);
//! ```
//!

use crate::Error;
use std::{
    mem,
    str::{Bytes, Lines},
};

/// An iterator of coordinates of living cells. Returns by parsing an
/// [RLE](https://www.conwaylife.com/wiki/Run_Length_Encoded) file.
pub struct RLE<'a> {
    lines: Lines<'a>,
    current_line: Bytes<'a>,
    x: i32,
    y: i32,
    count: i32,
    alive_count: i32,
}

impl RLE<'_> {
    /// Creates a new iterator from a string.
    pub fn new(text: &str) -> RLE {
        RLE {
            lines: text.lines(),
            current_line: "".bytes(),
            x: 0,
            y: 0,
            count: 0,
            alive_count: 0,
        }
    }
}

impl<'a> Iterator for RLE<'a> {
    /// Coordinates of a living cell, or an error in the pattern string.
    type Item = Result<(i32, i32), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.alive_count > 0 {
            self.x += 1;
            self.alive_count -= 1;
            if self.alive_count > 0 {
                return Some(Ok((self.y, self.x)));
            }
        }
        loop {
            if let Some(c) = self.current_line.next() {
                if c.is_ascii_digit() {
                    self.count = 10 * self.count + (c - b'0') as i32
                } else {
                    if self.count == 0 {
                        self.count = 1;
                    }
                    match c {
                        b'o' | b'A' => {
                            self.alive_count = mem::take(&mut self.count);
                            return Some(Ok((self.y, self.x)));
                        }
                        b'b' | b'.' => self.x += self.count,
                        b'$' => {
                            self.x = 0;
                            self.y += self.count
                        }
                        b'!' => return None,
                        _ => return Some(Err(Error::UnexpectedByte(c))),
                    }
                    self.count = 0;
                }
            } else if let Some(l) = self.lines.next() {
                if l.starts_with('#') | l.starts_with('x') {
                    continue;
                } else {
                    self.current_line = l.bytes();
                }
            } else {
                return None;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn glider_rle() -> Result<(), Error> {
        const GLIDER: &str = r"#N Glider
#O Richard K. Guy
#C The smallest, most common, and first discovered spaceship. Diagonal, has period 4 and speed c/4.
#C www.conwaylife.com/wiki/index.php?title=Glider
x = 3, y = 3, rule = B3/S23
bob$2bo$3o!";
        let mut glider = RLE::new(GLIDER).collect::<Result<Vec<_>, _>>()?;
        glider.sort();
        assert_eq!(glider, vec![(0, 1), (1, 2), (2, 0), (2, 1), (2, 2)]);
        Ok(())
    }

    #[test]
    fn twin_bees_shuttle_rle() -> Result<(), Error> {
        const TWIN_BEE_SHUTTLE: &str = r"#N Twin bees shuttle
#O Bill Gosper
#C Twin bees shuttle was found in 1971 and is the basis of all known period 46 oscillators.
#C www.conwaylife.com/wiki/index.php?title=Twin_bees_shuttle
x = 29, y = 11, rule = b3/s23
17b2o10b$2o15bobo7b2o$2o17bo7b2o$17b3o9b4$17b3o9b$2o17bo9b$2o15bobo9b$
17b2o!";
        let mut twin_bees_shuttle = RLE::new(TWIN_BEE_SHUTTLE).collect::<Result<Vec<_>, _>>()?;
        twin_bees_shuttle.sort();
        assert_eq!(
            twin_bees_shuttle,
            vec![
                (0, 17),
                (0, 18),
                (1, 0),
                (1, 1),
                (1, 17),
                (1, 19),
                (1, 27),
                (1, 28),
                (2, 0),
                (2, 1),
                (2, 19),
                (2, 27),
                (2, 28),
                (3, 17),
                (3, 18),
                (3, 19),
                (7, 17),
                (7, 18),
                (7, 19),
                (8, 0),
                (8, 1),
                (8, 19),
                (9, 0),
                (9, 1),
                (9, 17),
                (9, 19),
                (10, 17),
                (10, 18)
            ]
        );
        Ok(())
    }
}
