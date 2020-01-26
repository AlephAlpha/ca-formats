# CA formats parsers

[![Travis (.org)](https://img.shields.io/travis/AlephAlpha/ca-parsers)](https://travis-ci.org/AlephAlpha/ca-parsers) [![Crates.io](https://img.shields.io/crates/v/ca-parsers)](https://crates.io/crates/ca-parsers) [![Docs.rs](https://docs.rs/ca-parsers/badge.svg)](https://docs.rs/ca-rules/) [![中文](https://img.shields.io/badge/readme-%E4%B8%AD%E6%96%87-brightgreen)](README.md)

Parsing pattern files for Conway's Game of Life. The parsers read a string and return an iterator of coordinates of living cells.

Parsing is lazy. If there is something wrong in the file, it will not be detected
immediately.

Rules with more than 2 states are not supported.

## Supported formats

- [RLE](https://www.conwaylife.com/wiki/Run_Length_Encoded)
- [PlainText](https://www.conwaylife.com/wiki/Plaintext)
- [apgcode](https://www.conwaylife.com/wiki/Apgcode)

## Example

```rust
use ca_formats::rle::RLE;

const GLIDER: &str = r"#N Glider
 O Richard K. Guy
 C The smallest, most common, and first discovered spaceship. Diagonal, has period 4 and speed c/ C www.conwaylife.com/wiki/index.php?title=Glider
x = 3, y = 3, rule = B3/S23
bob$2bo$3o!";
let mut glider = RLE::new(GLIDER).collect::<Result<Vec<_>, _>>().unwrap();
glider.sort();
assert_eq!(glider, vec![(0, 1), (1, 2), (2, 0), (2, 1), (2, 2)]);
```

## See also

- [ca-rules](https://crates.io/crates/ca-rules) - A parser for rule strings.
- [game-of-life-parsers](https://crates.io/crates/game-of-life-parsers)
    by René Perschon - Parsers for [Life 1.05](https://www.conwaylife.com/wiki/Life_1.05)
    and [Life 1.06](https://www.conwaylife.com/wiki/Life_1.06) format.
