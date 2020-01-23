# CA formats parsers

读取生命游戏的图样文件。返回一个活细胞坐标的 Iterator。只适用于两种状态的规则。

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

## 文件格式

已支持的格式：

- [RLE](https://www.conwaylife.com/wiki/Run_Length_Encoded)
- [PlainText](https://www.conwaylife.com/wiki/Plaintext)

待支持的格式：

- [ ] [apgcode](https://www.conwaylife.com/wiki/Apgcode)
- [ ] [Macrocell](https://www.conwaylife.com/wiki/Macrocell)

别的格式用的人不多，懒得管了。

## 另见

- [ca-rules](https://github.com/AlephAlpha/ca-rules) - 读取元胞自动机的规则。
