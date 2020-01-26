# CA formats parsers

[![Travis (.org)](https://img.shields.io/travis/AlephAlpha/ca-formats)](https://travis-ci.org/AlephAlpha/ca-formats) [![Crates.io](https://img.shields.io/crates/v/ca-formats)](https://crates.io/crates/ca-formats) [![Docs.rs](https://docs.rs/ca-formats/badge.svg)](https://docs.rs/ca-formats/) [![中文](https://img.shields.io/badge/readme-%E4%B8%AD%E6%96%87-brightgreen)](README.md)

读取生命游戏的图样文件。返回一个活细胞坐标的 Iterator。

Parsing 是 Lazy 的。如果文件有错，它不会马上发现。

只适用于两种状态的规则。

## 支持的格式

- [RLE](https://www.conwaylife.com/wiki/Run_Length_Encoded)
- [PlainText](https://www.conwaylife.com/wiki/Plaintext)
- [apgcode](https://www.conwaylife.com/wiki/Apgcode)

## 用法

```rust
use ca_formats::rle::RLE;

const GLIDER: &str = r"#N Glider
#O Richard K. Guy
#C The smallest, most common, and first discovered spaceship. Diagonal, has period 4 and speed c/4.
#C www.conwaylife.com/wiki/index.php?title=Glider
x = 3, y = 3, rule = B3/S23
bob$2bo$3o!";
let mut glider = RLE::new(GLIDER).collect::<Result<Vec<_>, _>>().unwrap();
glider.sort();
assert_eq!(glider, vec![(0, 1), (1, 2), (2, 0), (2, 1), (2, 2)]);
```

## 另见

- [ca-rules](https://github.com/AlephAlpha/ca-rules) - 读取元胞自动机的规则。
- [game-of-life-parsers](https://crates.io/crates/game-of-life-parsers) by René Perschon - 读取 [Life 1.05](https://www.conwaylife.com/wiki/Life_1.05) 和 [Life 1.06](https://www.conwaylife.com/wiki/Life_1.06) 规则。
