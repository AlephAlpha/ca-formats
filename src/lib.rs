/*!
Parsing pattern files for Conway's Game of Life.

The parsers read a string and return an iterator of coordinates of living cells.

## Supported formats

- [RLE](https://www.conwaylife.com/wiki/Run_Length_Encoded)
- [Plaintext](https://www.conwaylife.com/wiki/Plaintext)
- [apgcode](https://www.conwaylife.com/wiki/Apgcode)
- [Macrocell](https://www.conwaylife.com/wiki/Macrocell)

## Example

### Reading from a string:

```rust
use ca_formats::rle::Rle;

const GLIDER: &str = r"#N Glider
#O Richard K. Guy
#C The smallest, most common, and first discovered spaceship. Diagonal, has period 4 and speed c/4.
#C www.conwaylife.com/wiki/index.php?title=Glider
x = 3, y = 3, rule = B3/S23
bob$2bo$3o!";

let glider = Rle::new(GLIDER).unwrap();
assert_eq!(glider.header_data().unwrap().x, 3);
assert_eq!(glider.header_data().unwrap().y, 3);
assert_eq!(glider.header_data().unwrap().rule, Some(String::from("B3/S23")));

let cells = glider.map(|cell| cell.unwrap().position).collect::<Vec<_>>();
assert_eq!(cells, vec![(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)]);
```

### Reading from a file:

```rust
use std::fs::File;
use ca_formats::rle::Rle;

let file = File::open("tests/sirrobin.rle").unwrap();
let sirrobin = Rle::new_from_file(file).unwrap();

assert_eq!(sirrobin.count(), 282);
```

## See also

- [ca-rules](https://crates.io/crates/ca-rules) - A parser for rule strings.
- [game-of-life-parsers](https://crates.io/crates/game-of-life-parsers)
    by René Perschon - Parsers for [Life 1.05](https://www.conwaylife.com/wiki/Life_1.05)
    and [Life 1.06](https://www.conwaylife.com/wiki/Life_1.06) format.

*/

#![cfg_attr(docs_rs, feature(doc_cfg))]

pub mod apgcode;
mod input;
pub mod macrocell;
pub mod plaintext;
pub mod rle;

pub use input::Input;

pub type Coordinates = (i64, i64);

/// Position and state of a cell.
///
/// Rules with more than 256 states are not supported.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Default, Hash)]
pub struct CellData {
    /// Coordinates of the cell.
    pub position: Coordinates,
    /// State of the cell.
    ///
    /// For rules with only 2 states, `0` means dead and `1` means alive.
    pub state: u8,
}

/// Convert the coordinates into a [`CellData`] with state `1`.
impl From<Coordinates> for CellData {
    fn from(position: Coordinates) -> Self {
        Self { position, state: 1 }
    }
}
