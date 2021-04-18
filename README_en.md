# CA formats parsers

[![Travis (.com)](https://img.shields.io/travis/com/AlephAlpha/ca-formats)](https://travis-ci.com/AlephAlpha/ca-formats) [![Crates.io](https://img.shields.io/crates/v/ca-formats)](https://crates.io/crates/ca-formats) [![Docs.rs](https://docs.rs/ca-formats/badge.svg)](https://docs.rs/ca-formats/) [![中文](https://img.shields.io/badge/readme-%E4%B8%AD%E6%96%87-brightgreen)](README.md)

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

## Unknown cells

When the `unknown` feature is enabled. the `Rle` type will provide an extra method `with_unknown`, which turns the RLE into a special variant of RLE format. In this variant there is another symbol, `?`, which represents unknown cells. Now unknown cells are the background. Dead cells at the end of each line must not be omitted. The iterator will also explicitly output the dead cells.

This is only supported for RLE.

## See also

- [ca-rules](https://crates.io/crates/ca-rules) - A parser for rule strings.
- [game-of-life-parsers](https://crates.io/crates/game-of-life-parsers)
    by René Perschon - Parsers for [Life 1.05](https://www.conwaylife.com/wiki/Life_1.05)
    and [Life 1.06](https://www.conwaylife.com/wiki/Life_1.06) format.
