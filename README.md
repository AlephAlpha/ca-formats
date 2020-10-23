# CA formats parsers

[![Travis (.com)](https://img.shields.io/travis/com/AlephAlpha/ca-formats)](https://travis-ci.com/AlephAlpha/ca-formats) [![Crates.io](https://img.shields.io/crates/v/ca-formats)](https://crates.io/crates/ca-formats) [![Docs.rs](https://docs.rs/ca-formats/badge.svg)](https://docs.rs/ca-formats/) [![English](https://img.shields.io/badge/readme-English-brightgreen)](README_en.md)

读取生命游戏的图样文件，返回一个活细胞坐标的 Iterator。

正在重写中。重写之后将不再兼容以前的版本。

## 支持的格式

- [RLE](https://www.conwaylife.com/wiki/Run_Length_Encoded)
- [Plaintext](https://www.conwaylife.com/wiki/Plaintext)
<!-- - [apgcode](https://www.conwaylife.com/wiki/Apgcode) -->

## 用法

```rust
use ca_formats::rle::Rle;

const GLIDER: &str = r"#N Glider
#O Richard K. Guy
#C The smallest, most common, and first discovered spaceship. Diagonal, has period 4 and speed c/4.
#C www.conwaylife.com/wiki/index.php?title=Glider
x = 3, y = 3, rule = B3/S23
bob$2bo$3o!";

let mut glider = Rle::new(GLIDER).unwrap();
assert_eq!(glider.header_data().unwrap().x, 3);
assert_eq!(glider.header_data().unwrap().y, 3);
assert_eq!(glider.header_data().unwrap().rule, Some(String::from("B3/S23")));

let cells = glider
    .cells()
    .map(|cell| cell.unwrap().position)
    .collect::<Vec<_>>();
assert_eq!(cells, vec![(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)]);
```

## 另见

- [ca-rules](https://github.com/AlephAlpha/ca-rules) - 读取元胞自动机的规则。
- [game-of-life-parsers](https://crates.io/crates/game-of-life-parsers) by René Perschon - 读取 [Life 1.05](https://www.conwaylife.com/wiki/Life_1.05) 和 [Life 1.06](https://www.conwaylife.com/wiki/Life_1.06) 规则。
