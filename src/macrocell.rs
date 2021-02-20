//! A parser for [Macrocell](http://golly.sourceforge.net/Help/formats.html#mc) format.

use crate::Input;
use lazy_static::lazy_static;
use regex::Regex;
use std::io::{BufReader, Error as IoError, Read};
use thiserror::Error;

/// Errors that can be returned when parsing a Macrocell file.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid header line: {0}.")]
    InvalidHeaderLine(String),
    #[error("Invalid node line: {0}.")]
    InvalidNodeLine(String),
    #[error("Error when reading from input: {0}.")]
    IoError(IoError),
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Error::IoError(error)
    }
}

/// A node in [HashLife](https://conwaylife.com/wiki/HashLife)'s quadtree.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Node {
    pub id: usize,
    pub data: NodeData,
}

/// Data in a `Node`.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum NodeData {
    /// A level 1 leaf, representing a 2x2 square, in rules with more than 2 states.
    ///
    /// The data contains the states of four cells in the square.
    Level1 { nw: u8, ne: u8, sw: u8, se: u8 },
    /// A level 3 leaf, representing a 8x8 square, in rules with 2 states.
    ///
    /// The data is represented by a 64-bit integer.
    Level3(u64),
    /// A non-leaf node.
    ///
    /// The data contains the level of the node,
    /// and the ids of four children.
    Node {
        level: u8,
        nw: usize,
        ne: usize,
        sw: usize,
        se: usize,
    },
}

impl NodeData {
    pub fn level(&self) -> u8 {
        match self {
            NodeData::Level1 { .. } => 1,
            NodeData::Level3(_) => 3,
            NodeData::Node { level, .. } => *level,
        }
    }
}

/// Parse a level 3 leaf.
fn parse_level3(line: &str) -> Option<NodeData> {
    let mut node = 0;
    let (mut x, mut y) = (0_u8, 0_u8);
    for char in line.bytes() {
        match char {
            b'.' => x += 1,
            b'*' => {
                if x >= 8 || y >= 8 {
                    return None;
                }
                node |= 1 << ((7 - y) * 8 + (7 - x));
                x += 1;
            }
            b'$' => {
                x = 0;
                y += 1;
            }
            c if c.is_ascii_whitespace() => (),
            _ => return None,
        }
    }
    Some(NodeData::Level3(node))
}

/// Parse a level 1 leaf.
fn parse_level1(line: &str) -> Option<NodeData> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^1\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)").unwrap();
    }
    let cap = RE.captures(line)?;
    let nw = cap[1].parse().ok()?;
    let ne = cap[2].parse().ok()?;
    let sw = cap[3].parse().ok()?;
    let se = cap[4].parse().ok()?;
    Some(NodeData::Level1 { nw, ne, sw, se })
}

/// Parse a non-leaf node.
fn parse_node(line: &str) -> Option<NodeData> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)").unwrap();
    }
    let cap = RE.captures(line)?;
    let level = cap[1].parse().ok()?;
    let nw = cap[2].parse().ok()?;
    let ne = cap[3].parse().ok()?;
    let sw = cap[4].parse().ok()?;
    let se = cap[5].parse().ok()?;
    Some(NodeData::Node {
        level,
        nw,
        ne,
        sw,
        se,
    })
}

/// Parse the rulestring.
fn parse_rule(line: &str) -> Option<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^#R\s*(?P<rule>.*\S)\s*$").unwrap();
    }
    let cap = RE.captures(line)?;
    let rule = cap["rule"].to_string();
    Some(rule)
}

/// Parse the current generation.
fn parse_gen(line: &str) -> Option<u64> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^#G\s*(?P<gen>\d+)\s*$").unwrap();
    }
    let cap = RE.captures(line)?;
    let gen = cap["gen"].parse().ok()?;
    Some(gen)
}

/// A parser for [Macrocell](http://golly.sourceforge.net/Help/formats.html#mc) format.
///
/// This format is specifically designed for the [HashLife](https://conwaylife.com/wiki/HashLife)
/// algorithm. So as an iterator, it iterates over the nodes in the quadtree,
/// instead of the living cells.
///
/// # Examples
///
/// ## Reading from a string:
///
/// ```rust
/// use ca_formats::macrocell::{Macrocell, NodeData};
///
/// const GLIDER: &str = r"[M2] (golly 3.4)
/// #R B3/S23
/// $$$$$$*$.*$
/// .......*$
/// **$
/// 4 0 1 2 3";
///
/// let glider = Macrocell::new(GLIDER).unwrap();
/// assert_eq!(glider.rule().unwrap(), "B3/S23");
///
/// let last_node = glider.last().unwrap().unwrap();
/// assert_eq!(last_node.id, 4);
/// assert_eq!(
///     last_node.data,
///     NodeData::Node {
///         level: 4,
///         nw: 0,
///         ne: 1,
///         sw: 2,
///         se: 3,
///     }
/// );
/// ```
///
/// ## Reading from a file:
///
/// ``` rust
/// use std::fs::File;
/// use ca_formats::macrocell::Macrocell;
///
/// let file = File::open("tests/sirrobin.mc").unwrap();
/// let sirrobin = Macrocell::new_from_file(file).unwrap();
///
/// assert_eq!(sirrobin.count(), 42); // The number of nodes.
#[derive(Debug)]
pub struct Macrocell<I: Input> {
    /// Rulestring.
    rule: Option<String>,
    /// Current generation.
    gen: Option<u64>,
    /// An iterator over lines of the Macrocell string.
    lines: I::Lines,
    /// The current line.
    current_line: Option<I::Line>,
    /// The current node id.
    id: usize,
}

impl<I: Input> Macrocell<I> {
    /// Create a new parser instance from input, and try to read the header lines.
    pub fn new(input: I) -> Result<Self, Error> {
        let mut lines = input.lines();
        let mut rule = None;
        let mut gen = None;
        let mut current_line = None;
        while let Some(item) = lines.next() {
            let line = I::line(item)?;
            if line.as_ref().starts_with("[M2]") {
                continue;
            } else if line.as_ref().starts_with("#R") {
                rule.replace(
                    parse_rule(line.as_ref())
                        .ok_or_else(|| Error::InvalidHeaderLine(line.as_ref().to_string()))?,
                );
            } else if line.as_ref().starts_with("#G") {
                gen.replace(
                    parse_gen(line.as_ref())
                        .ok_or_else(|| Error::InvalidHeaderLine(line.as_ref().to_string()))?,
                );
            } else if !line.as_ref().starts_with('#') {
                current_line = Some(line);
                break;
            }
        }
        Ok(Macrocell {
            rule,
            gen,
            lines,
            current_line,
            id: 1,
        })
    }

    /// The rulestring.
    pub fn rule(&self) -> Option<&str> {
        self.rule.as_deref()
    }

    /// The current generation.
    pub fn gen(&self) -> Option<u64> {
        self.gen
    }
}

impl<R: Read> Macrocell<BufReader<R>> {
    /// Creates a new parser instance from something that implements [`Read`] trait, e.g., a [`File`](std::fs::File).
    pub fn new_from_file(file: R) -> Result<Self, Error> {
        Self::new(BufReader::new(file))
    }
}

impl<I: Input> Clone for Macrocell<I>
where
    I::Lines: Clone,
    I::Line: Clone,
{
    fn clone(&self) -> Self {
        Macrocell {
            rule: self.rule.clone(),
            gen: self.gen,
            lines: self.lines.clone(),
            current_line: self.current_line.clone(),
            id: self.id,
        }
    }
}

/// An iterator over quadtree nodes in an Macrocell file.
impl<I: Input> Iterator for Macrocell<I> {
    type Item = Result<Node, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(line) = self.current_line.take() {
                if line.as_ref().starts_with('#') {
                    continue;
                } else if line.as_ref().starts_with(&['.', '*', '$'][..]) {
                    if let Some(data) = parse_level3(line.as_ref()) {
                        let node = Node { id: self.id, data };
                        self.id += 1;
                        return Some(Ok(node));
                    } else {
                        return Some(Err(Error::InvalidNodeLine(line.as_ref().to_string())));
                    }
                } else if line.as_ref().starts_with("1 ") {
                    if let Some(data) = parse_level1(line.as_ref()) {
                        let node = Node { id: self.id, data };
                        self.id += 1;
                        return Some(Ok(node));
                    } else {
                        return Some(Err(Error::InvalidNodeLine(line.as_ref().to_string())));
                    }
                } else if let Some(data) = parse_node(line.as_ref()) {
                    let node = Node { id: self.id, data };
                    self.id += 1;
                    return Some(Ok(node));
                } else {
                    return Some(Err(Error::InvalidNodeLine(line.as_ref().to_string())));
                }
            } else if let Some(item) = self.lines.next() {
                match I::line(item) {
                    Ok(line) => {
                        self.current_line = Some(line);
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

#[allow(clippy::unusual_byte_groupings)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macrocell_parse_line() {
        assert_eq!(
            parse_level3("$$..*$...*$.***$$$$"),
            Some(NodeData::Level3(
                0b_00000000_00000000_00100000_00010000_01110000_00000000_00000000_00000000
            ))
        );
        assert_eq!(parse_level3("$$..*$...*$.***$$$$*"), None);
        assert_eq!(
            parse_level1("1 2 3 4 255"),
            Some(NodeData::Level1 {
                nw: 2,
                ne: 3,
                sw: 4,
                se: 255,
            })
        );
        assert_eq!(parse_level1("1 2 3 4 256"), None);
        assert_eq!(
            parse_node("10 20 30 40 50"),
            Some(NodeData::Node {
                level: 10,
                nw: 20,
                ne: 30,
                sw: 40,
                se: 50,
            })
        );
        assert_eq!(parse_node("10 20 30 40"), None);
    }

    #[test]
    fn macrocell_glider() -> Result<(), Error> {
        const GLIDER: &str = r"[M2] (golly 3.4)
#R B3/S23
$$$$$$*$.*$
.......*$
**$
4 0 1 2 3";

        let glider = Macrocell::new(GLIDER)?;

        let _ = glider.clone();

        assert_eq!(glider.rule(), Some("B3/S23"));

        let nodes = glider.collect::<Result<Vec<_>, _>>()?;
        assert_eq!(
            nodes,
            vec![
                Node {
                    id: 1,
                    data: NodeData::Level3(
                        0b_00000000_00000000_00000000_00000000_00000000_00000000_10000000_01000000
                    )
                },
                Node {
                    id: 2,
                    data: NodeData::Level3(
                        0b_00000001_00000000_00000000_00000000_00000000_00000000_00000000_00000000
                    )
                },
                Node {
                    id: 3,
                    data: NodeData::Level3(
                        0b_11000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000
                    )
                },
                Node {
                    id: 4,
                    data: NodeData::Node {
                        level: 4,
                        nw: 0,
                        ne: 1,
                        sw: 2,
                        se: 3,
                    }
                }
            ]
        );
        Ok(())
    }
}
