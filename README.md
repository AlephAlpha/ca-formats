# CA formats parsers

[![Travis (.com)](https://img.shields.io/travis/com/AlephAlpha/ca-formats)](https://travis-ci.com/AlephAlpha/ca-formats) [![Crates.io](https://img.shields.io/crates/v/ca-formats)](https://crates.io/crates/ca-formats) [![Docs.rs](https://docs.rs/ca-formats/badge.svg)](https://docs.rs/ca-formats/) [![English](https://img.shields.io/badge/readme-English-brightgreen)](README_en.md)

读取生命游戏的图样文件，返回一个活细胞坐标的 Iterator。

## 支持的格式

- [RLE](https://www.conwaylife.com/wiki/Run_Length_Encoded)
- [Plaintext](https://www.conwaylife.com/wiki/Plaintext)
- [apgcode](https://www.conwaylife.com/wiki/Apgcode)
- [Macrocell](https://www.conwaylife.com/wiki/Macrocell)

## 范例

### 从字符串中读取:

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

### 从文件中读取:

```rust
use std::fs::File;
use ca_formats::rle::Rle;

let file = File::open("tests/sirrobin.rle").unwrap();
let sirrobin = Rle::new_from_file(file).unwrap();

assert_eq!(sirrobin.count(), 282);
```

## 未知的细胞

当启用 `unknown` feature 时，`Rle` 类型会提供一个名为 `with_unknown` 的方法，用来切换到 RLE 的一个特别的变种：这种 RLE 多了一个符号 `?`，用来表示未知的细胞。此时图样的背景是未知的细胞，每行末尾的死细胞不可省略，生成的 Iterator 也会输出每一个死细胞。

只有 RLE 格式支持此功能。

## 另见

- [ca-rules](https://github.com/AlephAlpha/ca-rules) - 读取元胞自动机的规则。
- [game-of-life-parsers](https://crates.io/crates/game-of-life-parsers) by René Perschon - 读取 [Life 1.05](https://www.conwaylife.com/wiki/Life_1.05) 和 [Life 1.06](https://www.conwaylife.com/wiki/Life_1.06) 规则。
