//! Parsing pattern files for Conway's Game of Life.
//!
//! The parsers reads a string and returns an iterator of coordinates of living cells.
//!
//! # Example:
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
//! glider.sort();
//! assert_eq!(glider, vec![(0, 1), (1, 2), (2, 0), (2, 1), (2, 2)]);
//! ```

mod error;
mod formats;

pub use error::Error;
pub use formats::*;
