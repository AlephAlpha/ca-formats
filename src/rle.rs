//! A parser for Golly's [Extended RLE format](http://golly.sourceforge.net/Help/formats.html#rle).
//!
//! It is basically the same as the original [RLE](https://www.conwaylife.com/wiki/Run_Length_Encoded)
//! format, except that it supports up to 256 states, and a `#CXRLE` line.

use crate::{CellData, Coordinates};
use lazy_static::lazy_static;
use regex::Regex;
use std::str::{Bytes, Lines};
use thiserror::Error;

/// Errors that can be returned when parsing a RLE file.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum Error {
    #[error("Invalid state: {0}.")]
    InvalidState(String),
    #[error("Invalid \"#CXRLE\" line: {0}.")]
    InvalidCXRLELine(String),
    #[error("Invalid header line: {0}.")]
    InvalidHeaderLine(String),
}

/// Data from the `#CXRLE` line, e.g., `#CXRLE Pos=0,-1377 Gen=3480106827776`.
#[derive(Clone, Debug, Eq, PartialEq, Default, Hash)]
pub struct CxrleData {
    /// Coordinates of the upper left corner of the pattern.
    pub pos: Option<Coordinates>,
    /// Current generation.
    pub gen: Option<u64>,
}

/// Parse the `#CXRLE` line.
fn parse_cxrle(line: &str) -> Option<CxrleData> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"(?:Pos\s*=\s*(?P<x>-?\d+),\s*(?P<y>-?\d+))|(?:Gen\s*=\s*(?P<gen>\d+))")
                .unwrap();
    }
    let mut data = CxrleData::default();
    for cap in RE.captures_iter(line) {
        if let Some(gen) = cap.name("gen") {
            data.gen = Some(gen.as_str().parse().ok())?;
        } else {
            let x = cap["x"].parse().ok()?;
            let y = cap["y"].parse().ok()?;
            data.pos = Some((x, y));
        }
    }
    Some(data)
}

/// Data from the header line, e.g., `x = 3, y = 3, rule = B3/S23`.
#[derive(Clone, Debug, Eq, PartialEq, Default, Hash)]
pub struct HeaderData {
    /// Width of the pattern.
    pub x: u64,
    /// Height of the pattern.
    pub y: u64,
    /// Rulestring.
    pub rule: Option<String>,
}

/// Parse the header line.
fn parse_header(line: &str) -> Option<HeaderData> {
    let re =
        Regex::new(r"^x\s*=\s*(?P<x>\d+),\s*y\s*=\s*(?P<y>\d+)(?:,\s*rule\s*=\s*(?P<rule>\S+))?")
            .unwrap();
    let mut data = HeaderData::default();
    let cap = re.captures(line)?;
    data.x = cap["x"].parse().ok()?;
    data.y = cap["y"].parse().ok()?;
    if let Some(rule) = cap.name("rule") {
        data.rule = Some(rule.as_str().to_owned());
    }
    Some(data)
}

/// A parser for Golly's [Extended RLE format](http://golly.sourceforge.net/Help/formats.html#rle).
///
/// The format is basically the same as the original [RLE](https://www.conwaylife.com/wiki/Run_Length_Encoded)
/// format, except that it supports up to 256 states, and a `#CXRLE` line.
///
/// As an iterator, it iterates over the living cells.
///
/// # Example
///
/// ```rust
/// use ca_formats::rle::Rle;
///
/// const GLIDER: &str = r"#N Glider
/// #O Richard K. Guy
/// #C The smallest, most common, and first discovered spaceship. Diagonal, has period 4 and speed c/4.
/// #C www.conwaylife.com/wiki/index.php?title=Glider
/// x = 3, y = 3, rule = B3/S23
/// bob$2bo$3o!";
///
/// let glider = Rle::new(GLIDER).unwrap();
/// assert_eq!(glider.header_data().unwrap().x, 3);
/// assert_eq!(glider.header_data().unwrap().y, 3);
/// assert_eq!(glider.header_data().unwrap().rule, Some(String::from("B3/S23")));
///
/// let cells = glider.map(|cell| cell.unwrap().position).collect::<Vec<_>>();
/// assert_eq!(cells, vec![(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)]);
/// ```
#[derive(Clone, Debug)]
pub struct Rle<'a> {
    /// Data from the `#CXRLE` line.
    cxrle_data: Option<CxrleData>,

    /// Data from the header line.
    header_data: Option<HeaderData>,

    /// An iterator over lines of the RLE string.
    lines: Lines<'a>,

    /// An iterator over bytes of the current line.
    current_line: Bytes<'a>,

    /// Coordinates of the current cell.
    position: Coordinates,

    /// X coordinates of the upper left corner of the pattern.
    x_start: i64,

    /// Run count in the RLE string, i.e., the numbers before `b`, `o`, `$` and other tags.
    run_count: i64,

    /// Remaining run count for the current cell when iterating over cells.
    alive_count: i64,

    /// State of the current cell.
    state: u8,

    /// Prefix in a multi-char state, i.e., `p` in `pA`.
    state_prefix: Option<u8>,
}

impl<'a> Rle<'a> {
    /// Create a new parser instance from a string, and try to read the header and the `#CXRLE` line.
    ///
    /// If there are multiple header lines / `CXRLE` lines, only the last one will be taken.
    pub fn new(string: &'a str) -> Result<Self, Error> {
        let mut lines = string.lines();
        let mut cxrle_data = None;
        let mut header_data = None;
        let mut current_line = "".bytes();
        let mut position = (0, 0);
        let mut x_start = 0;
        while let Some(line) = lines.next() {
            if line.starts_with("#CXRLE") {
                cxrle_data
                    .replace(parse_cxrle(line).ok_or(Error::InvalidCXRLELine(line.to_string()))?);
            } else if line.starts_with("x ") || line.starts_with("x=") {
                header_data
                    .replace(parse_header(line).ok_or(Error::InvalidHeaderLine(line.to_string()))?);
            } else if !line.starts_with('#') {
                current_line = line.bytes();
                break;
            }
        }
        if let Some(CxrleData { pos: Some(pos), .. }) = cxrle_data {
            position = pos;
            x_start = pos.0;
        }
        Ok(Rle {
            cxrle_data,
            header_data,
            lines,
            current_line,
            position,
            x_start,
            run_count: 0,
            alive_count: 0,
            state: 1,
            state_prefix: None,
        })
    }

    /// Data from the `#CXRLE` line.
    pub fn cxrle_data(&self) -> Option<&CxrleData> {
        self.cxrle_data.as_ref()
    }

    /// Data from the header line.
    pub fn header_data(&self) -> Option<&HeaderData> {
        self.header_data.as_ref()
    }
}

/// An iterator over living cells in an RLE file.
impl<'a> Iterator for Rle<'a> {
    type Item = Result<CellData, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.alive_count > 0 {
            self.alive_count -= 1;
            let cell = CellData {
                position: self.position,
                state: self.state,
            };
            self.position.0 += 1;
            return Some(Ok(cell));
        }
        loop {
            if let Some(c) = self.current_line.next() {
                if c.is_ascii_digit() {
                    self.run_count = 10 * self.run_count + (c - b'0') as i64
                } else if !c.is_ascii_whitespace() {
                    if self.run_count == 0 {
                        self.run_count = 1;
                    }
                    if self.state_prefix.is_some() && (c < b'A' || c > b'X') {
                        let mut state_string = char::from(self.state_prefix.unwrap()).to_string();
                        state_string.push(char::from(c));
                        return Some(Err(Error::InvalidState(state_string)));
                    }
                    match c {
                        b'b' | b'.' => {
                            self.position.0 += self.run_count;
                            self.run_count = 0;
                        }
                        b'o' | b'A'..=b'X' => {
                            if c == b'o' {
                                self.state = 1;
                            } else {
                                self.state = 24 * (self.state_prefix.take().unwrap_or(b'o') - b'o');
                                self.state += c + 1 - b'A';
                            }
                            self.alive_count = self.run_count - 1;
                            self.run_count = 0;
                            let cell = CellData {
                                position: self.position,
                                state: self.state,
                            };
                            self.position.0 += 1;
                            return Some(Ok(cell));
                        }
                        b'p'..=b'y' => {
                            self.state_prefix = Some(c);
                        }
                        b'$' => {
                            self.position.0 = self.x_start;
                            self.position.1 += self.run_count;
                            self.run_count = 0;
                        }
                        b'!' => return None,
                        _ => return Some(Err(Error::InvalidState(char::from(c).to_string()))),
                    }
                }
            } else if let Some(l) = self.lines.next() {
                if l.starts_with('#') | l.starts_with("x ") | l.starts_with("x=") {
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
mod tests {
    use super::*;

    #[test]
    fn rle_parse_cxrle() {
        assert_eq!(
            parse_cxrle("#CXRLE"),
            Some(CxrleData {
                pos: None,
                gen: None
            })
        );
        assert_eq!(
            parse_cxrle("#CXRLE Pos=0,-1377 Gen=3480106827776"),
            Some(CxrleData {
                pos: Some((0, -1377)),
                gen: Some(3480106827776)
            })
        );
        assert_eq!(
            parse_cxrle("#CXRLE Gen = 3480106827776 Pos = 0, -1377"),
            Some(CxrleData {
                pos: Some((0, -1377)),
                gen: Some(3480106827776)
            })
        );
        assert_eq!(
            parse_cxrle("#CXRLE211Pos=0,-9dcdcs2,[a ccGen=348sss1068cscPos= -333,-1a6"),
            Some(CxrleData {
                pos: Some((-333, -1)),
                gen: Some(348)
            })
        );
    }

    #[test]
    fn rle_parse_header() {
        assert_eq!(parse_header("xxx"), None);
        assert_eq!(
            parse_header("x = 3, y = 3, rule = B3/S23"),
            Some(HeaderData {
                x: 3,
                y: 3,
                rule: Some(String::from("B3/S23"))
            })
        );
        assert_eq!(
            parse_header("x = 3, y = 3"),
            Some(HeaderData {
                x: 3,
                y: 3,
                rule: None
            })
        );
        assert_eq!(parse_header("x = 3, y = -3"), None);
        assert_eq!(
            parse_header("x=3,y=3,rule=B3/S23   ignored"),
            Some(HeaderData {
                x: 3,
                y: 3,
                rule: Some(String::from("B3/S23"))
            })
        );
    }

    #[test]
    fn rle_glider() -> Result<(), Error> {
        const GLIDER: &str = r"#N Glider
#O Richard K. Guy
#C The smallest, most common, and first discovered spaceship. Diagonal, has period 4 and speed c/4.
#C www.conwaylife.com/wiki/index.php?title=Glider
x = 3, y = 3, rule = B3/S23
bob$2bo$3o!";

        let glider = Rle::new(GLIDER)?;

        assert_eq!(glider.cxrle_data, None);
        assert_eq!(
            glider.header_data,
            Some(HeaderData {
                x: 3,
                y: 3,
                rule: Some(String::from("B3/S23"))
            })
        );

        let cells = glider
            .map(|res| res.map(|c| c.position))
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(cells, vec![(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)]);
        Ok(())
    }

    #[test]
    fn rle_glider_cxrle() -> Result<(), Error> {
        const GLIDER: &str = r"#CXRLE Pos=-1,-1
x = 3, y = 3, rule = B3/S23
bo$2bo$3o!";

        let glider = Rle::new(GLIDER)?;

        assert_eq!(
            glider.cxrle_data,
            Some(CxrleData {
                pos: Some((-1, -1)),
                gen: None
            })
        );
        assert_eq!(
            glider.header_data,
            Some(HeaderData {
                x: 3,
                y: 3,
                rule: Some(String::from("B3/S23"))
            })
        );

        let cells = glider
            .map(|res| res.map(|c| c.position))
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(cells, vec![(0, -1), (1, 0), (-1, 1), (0, 1), (1, 1)]);
        Ok(())
    }

    #[test]
    fn rle_generations() -> Result<(), Error> {
        const OSCILLATOR: &str = r"x = 3, y = 3, rule = 3457/357/5
3A$B2A$.CD!";

        let oscillator = Rle::new(OSCILLATOR)?;

        assert_eq!(oscillator.cxrle_data, None);
        assert_eq!(
            oscillator.header_data,
            Some(HeaderData {
                x: 3,
                y: 3,
                rule: Some(String::from("3457/357/5"))
            })
        );

        let cells = oscillator.collect::<Result<Vec<_>, _>>()?;
        assert_eq!(
            cells,
            vec![
                CellData {
                    position: (0, 0),
                    state: 1
                },
                CellData {
                    position: (1, 0),
                    state: 1
                },
                CellData {
                    position: (2, 0),
                    state: 1
                },
                CellData {
                    position: (0, 1),
                    state: 2
                },
                CellData {
                    position: (1, 1),
                    state: 1
                },
                CellData {
                    position: (2, 1),
                    state: 1
                },
                CellData {
                    position: (1, 2),
                    state: 3
                },
                CellData {
                    position: (2, 2),
                    state: 4
                },
            ]
        );
        Ok(())
    }

    #[test]
    fn rle_generations_256() -> Result<(), Error> {
        const OSCILLATOR: &str = r"x = 3, y = 3, rule = 23/3/256
.AwH$vIxNrQ$2pU!";

        let oscillator = Rle::new(OSCILLATOR)?;

        assert_eq!(oscillator.cxrle_data, None);
        assert_eq!(
            oscillator.header_data,
            Some(HeaderData {
                x: 3,
                y: 3,
                rule: Some(String::from("23/3/256"))
            })
        );

        let cells = oscillator.collect::<Result<Vec<_>, _>>()?;
        assert_eq!(
            cells,
            vec![
                CellData {
                    position: (1, 0),
                    state: 1
                },
                CellData {
                    position: (2, 0),
                    state: 200
                },
                CellData {
                    position: (0, 1),
                    state: 177
                },
                CellData {
                    position: (1, 1),
                    state: 230
                },
                CellData {
                    position: (2, 1),
                    state: 89
                },
                CellData {
                    position: (0, 2),
                    state: 45
                },
                CellData {
                    position: (1, 2),
                    state: 45
                },
            ]
        );
        Ok(())
    }
}
