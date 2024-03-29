//! A parser for Golly's [Extended RLE format](http://golly.sourceforge.net/Help/formats.html#rle).
//!
//! It is basically the same as the original [RLE](https://www.conwaylife.com/wiki/Run_Length_Encoded)
//! format, except that it supports up to 256 states, and a `#CXRLE` line.

use crate::{CellData, Coordinates, Input};
use lazy_regex::regex;
use std::io::{BufReader, Error as IoError, Read};
use thiserror::Error;

/// Errors that can be returned when parsing a RLE file.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid state: {0}.")]
    InvalidState(String),
    #[error("Invalid \"#CXRLE\" line: {0}.")]
    InvalidCxrleLine(String),
    #[error("Invalid header line: {0}.")]
    InvalidHeaderLine(String),
    #[error("Error when reading from input: {0}.")]
    IoError(#[from] IoError),
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
    let re = regex!(r"(?:Pos\s*=\s*(?P<x>-?\d+),\s*(?P<y>-?\d+))|(?:Gen\s*=\s*(?P<gen>\d+))");
    let mut data = CxrleData::default();
    for cap in re.captures_iter(line) {
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
        regex!(r"^x\s*=\s*(?P<x>\d+),\s*y\s*=\s*(?P<y>\d+)(?:,\s*rule\s*=\s*(?P<rule>.*\S)\s*)?$");
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
/// # Examples
///
/// ## Reading from a string:
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
///
/// ## Reading from a file:
///
/// ``` rust
/// use std::fs::File;
/// use ca_formats::rle::Rle;
///
/// let file = File::open("tests/sirrobin.rle").unwrap();
/// let sirrobin = Rle::new_from_file(file).unwrap();
///
/// assert_eq!(sirrobin.count(), 282);
/// ```
#[must_use]
#[derive(Debug)]
pub struct Rle<I: Input> {
    /// Data from the `#CXRLE` line.
    cxrle_data: Option<CxrleData>,

    /// Data from the header line.
    header_data: Option<HeaderData>,

    /// An iterator over lines of the RLE string.
    lines: I::Lines,

    /// An iterator over bytes of the current line.
    current_line: Option<I::Bytes>,

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

    /// Whether this RLE file allows unknown cells.
    #[cfg(feature = "unknown")]
    unknown: bool,
}

impl<I: Input> Rle<I> {
    /// Create a new parser instance from input, and try to read the header and the `#CXRLE` line.
    ///
    /// If there are multiple header lines / `CXRLE` lines, only the last one will be taken.
    pub fn new(input: I) -> Result<Self, Error> {
        let mut lines = input.lines();
        let mut cxrle_data = None;
        let mut header_data = None;
        let mut current_line = None;
        let mut position = (0, 0);
        let mut x_start = 0;
        for item in &mut lines {
            let line = I::line(item)?;
            if line.as_ref().starts_with("#CXRLE") {
                cxrle_data.replace(
                    parse_cxrle(line.as_ref())
                        .ok_or_else(|| Error::InvalidCxrleLine(line.as_ref().to_string()))?,
                );
            } else if line.as_ref().starts_with("x ") || line.as_ref().starts_with("x=") {
                header_data.replace(
                    parse_header(line.as_ref())
                        .ok_or_else(|| Error::InvalidHeaderLine(line.as_ref().to_string()))?,
                );
            } else if !line.as_ref().starts_with('#') {
                current_line = Some(I::bytes(line));
                break;
            }
        }
        if let Some(CxrleData { pos: Some(pos), .. }) = cxrle_data {
            position = pos;
            x_start = pos.0;
        }
        Ok(Self {
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
            #[cfg(feature = "unknown")]
            unknown: false,
        })
    }

    /// Data from the `#CXRLE` line.
    pub const fn cxrle_data(&self) -> Option<&CxrleData> {
        self.cxrle_data.as_ref()
    }

    /// Data from the header line.
    pub const fn header_data(&self) -> Option<&HeaderData> {
        self.header_data.as_ref()
    }

    /// Allow unknown cells.
    ///
    /// In this variant of RLE format, there is another symbol, `?`,
    /// which represents unknown cells. Now unknown cells are the background.
    /// Dead cells at the end of each line must not be omitted.
    /// The iterator will also explicitly output the dead cells.
    #[cfg(feature = "unknown")]
    #[cfg_attr(docs_rs, doc(cfg(feature = "unknown")))]
    pub fn with_unknown(mut self) -> Self {
        self.unknown = true;
        self
    }
}

impl<I, L> Rle<I>
where
    I: Input<Lines = L>,
    L: Input,
{
    /// Parse the remaining unparsed lines as a new RLE.
    pub fn remains(self) -> Result<Rle<L>, Error> {
        Rle::new(self.lines)
    }

    /// Try to parse the remaining unparsed lines as a new RLE.
    ///
    /// Returns `Ok(None)` if the remaining lines is empty or only
    /// contains header lines and comments.
    pub fn try_remains(self) -> Result<Option<Rle<L>>, Error> {
        let rle = Rle::new(self.lines)?;
        Ok(if rle.current_line.is_some() {
            Some(rle)
        } else {
            None
        })
    }
}

impl<R: Read> Rle<BufReader<R>> {
    /// Creates a new parser instance from something that implements [`Read`] trait,
    /// e.g., a [`File`](std::fs::File).
    pub fn new_from_file(file: R) -> Result<Self, Error> {
        Self::new(BufReader::new(file))
    }
}

impl<I: Input> Clone for Rle<I>
where
    I::Lines: Clone,
    I::Bytes: Clone,
{
    fn clone(&self) -> Self {
        Self {
            cxrle_data: self.cxrle_data.clone(),
            header_data: self.header_data.clone(),
            lines: self.lines.clone(),
            current_line: self.current_line.clone(),
            position: self.position,
            x_start: self.x_start,
            run_count: self.run_count,
            alive_count: self.alive_count,
            state: self.state,
            state_prefix: self.state_prefix,
            #[cfg(feature = "unknown")]
            unknown: self.unknown,
        }
    }
}

/// An iterator over living cells in an RLE file.
impl<I: Input> Iterator for Rle<I> {
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
            if let Some(c) = self.current_line.as_mut().and_then(Iterator::next) {
                if c.is_ascii_digit() {
                    self.run_count = 10 * self.run_count + (c - b'0') as i64
                } else if !c.is_ascii_whitespace() {
                    if self.run_count == 0 {
                        self.run_count = 1;
                    }
                    if self.state_prefix.is_some() && !(b'A'..=b'X').contains(&c) {
                        let mut state_string = char::from(self.state_prefix.unwrap()).to_string();
                        state_string.push(char::from(c));
                        return Some(Err(Error::InvalidState(state_string)));
                    }
                    match c {
                        #[cfg(feature = "unknown")]
                        b'?' if self.unknown => {
                            self.position.0 += self.run_count;
                            self.run_count = 0;
                        }
                        #[cfg(feature = "unknown")]
                        b'b' | b'.' | b'o' | b'A'..=b'X' if self.unknown => {
                            match c {
                                b'b' | b'.' => self.state = 0,
                                b'o' => self.state = 1,
                                _ => {
                                    self.state =
                                        24 * (self.state_prefix.take().unwrap_or(b'o') - b'o');
                                    self.state += c + 1 - b'A';
                                }
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
                        b'!' => {
                            self.current_line = None;
                            return None;
                        }
                        _ => return Some(Err(Error::InvalidState(char::from(c).to_string()))),
                    }
                }
            } else if let Some(item) = self.lines.next() {
                match I::line(item) {
                    Ok(line) => {
                        if line.as_ref().starts_with('#')
                            | line.as_ref().starts_with("x ")
                            | line.as_ref().starts_with("x=")
                        {
                            continue;
                        } else {
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
            parse_header("x = 3, y = 3, rule = Conway's Game of Life  "),
            Some(HeaderData {
                x: 3,
                y: 3,
                rule: Some(String::from("Conway's Game of Life"))
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

        let _ = glider.clone();

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
                    state: 1,
                },
                CellData {
                    position: (1, 0),
                    state: 1,
                },
                CellData {
                    position: (2, 0),
                    state: 1,
                },
                CellData {
                    position: (0, 1),
                    state: 2,
                },
                CellData {
                    position: (1, 1),
                    state: 1,
                },
                CellData {
                    position: (2, 1),
                    state: 1,
                },
                CellData {
                    position: (1, 2),
                    state: 3,
                },
                CellData {
                    position: (2, 2),
                    state: 4,
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
                    state: 1,
                },
                CellData {
                    position: (2, 0),
                    state: 200,
                },
                CellData {
                    position: (0, 1),
                    state: 177,
                },
                CellData {
                    position: (1, 1),
                    state: 230,
                },
                CellData {
                    position: (2, 1),
                    state: 89,
                },
                CellData {
                    position: (0, 2),
                    state: 45,
                },
                CellData {
                    position: (1, 2),
                    state: 45,
                },
            ]
        );
        Ok(())
    }

    #[test]
    fn rle_two_rles() -> Result<(), Error> {
        const GLIDER: &str = r"#N Glider
#O Richard K. Guy
#C The smallest, most common, and first discovered spaceship. Diagonal, has period 4 and speed c/4.
#C www.conwaylife.com/wiki/index.php?title=Glider
x = 3, y = 3, rule = B3/S23
bob$2bo$3o!
#N Glider
#O Richard K. Guy
#C The smallest, most common, and first discovered spaceship. Diagonal, has period 4 and speed c/4.
#C www.conwaylife.com/wiki/index.php?title=Glider
x = 3, y = 3, rule = B3/S23
bob$2bo$3o!";

        let mut first_rle = Rle::new(GLIDER)?;

        assert_eq!(first_rle.cxrle_data, None);
        assert_eq!(
            first_rle.header_data,
            Some(HeaderData {
                x: 3,
                y: 3,
                rule: Some(String::from("B3/S23"))
            })
        );

        let mut cells = Vec::new();
        for c in &mut first_rle {
            cells.push(c?.position);
        }
        assert_eq!(cells, vec![(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)]);

        let mut second_rle = first_rle.remains()?;
        cells.clear();

        for c in &mut second_rle {
            cells.push(c?.position);
        }

        let third_rle = second_rle.try_remains()?;
        assert!(third_rle.is_none());
        Ok(())
    }

    #[test]
    #[cfg(feature = "unknown")]
    fn rle_glider_with_unknown() -> Result<(), Error> {
        const GLIDER: &str = r"#CXRLE Pos=-1,-1
x = 3, y = 3, rule = B3/S23
5?$?bob?$?2bo?$?3o?$5?!";

        let glider = Rle::new(GLIDER)?.with_unknown();

        assert_eq!(
            glider.cxrle_data(),
            Some(&CxrleData {
                pos: Some((-1, -1)),
                gen: None
            })
        );
        assert_eq!(
            glider.header_data(),
            Some(&HeaderData {
                x: 3,
                y: 3,
                rule: Some(String::from("B3/S23"))
            })
        );

        let cells = glider.collect::<Result<Vec<_>, _>>()?;
        assert_eq!(
            cells,
            vec![
                CellData {
                    position: (0, 0),
                    state: 0,
                },
                CellData {
                    position: (1, 0),
                    state: 1,
                },
                CellData {
                    position: (2, 0),
                    state: 0,
                },
                CellData {
                    position: (0, 1),
                    state: 0,
                },
                CellData {
                    position: (1, 1),
                    state: 0,
                },
                CellData {
                    position: (2, 1),
                    state: 1,
                },
                CellData {
                    position: (0, 2),
                    state: 1,
                },
                CellData {
                    position: (1, 2),
                    state: 1,
                },
                CellData {
                    position: (2, 2),
                    state: 1,
                },
            ]
        );
        Ok(())
    }
}
