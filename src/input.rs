use std::{
    io::{BufRead, BufReader, Error, Lines as IoLines, Read},
    str::{Bytes, Lines},
    vec::IntoIter,
};

/// Types that can be passed to parsers as input.
///
/// The trait is implemented for `&str` and `BufReader`.
/// When parsing a file, you can take a `BufReader<File>` as input.
pub trait Input {
    type Lines: Iterator;
    type Line: AsRef<str>;
    type Bytes: Iterator<Item = u8>;

    fn lines(self) -> Self::Lines;

    fn line(item: <Self::Lines as Iterator>::Item) -> Result<Self::Line, Error>;

    fn bytes(line: Self::Line) -> Self::Bytes;
}

impl<'a> Input for &'a str {
    type Lines = Lines<'a>;
    type Line = &'a str;
    type Bytes = Bytes<'a>;

    fn lines(self) -> Self::Lines {
        self.lines()
    }

    fn line(item: <Self::Lines as Iterator>::Item) -> Result<Self::Line, Error> {
        Ok(item)
    }

    fn bytes(line: Self::Line) -> Self::Bytes {
        line.bytes()
    }
}

impl<R: Read> Input for BufReader<R> {
    type Lines = IoLines<BufReader<R>>;
    type Line = String;
    type Bytes = IntoIter<u8>;

    fn lines(self) -> Self::Lines {
        BufRead::lines(self)
    }

    fn line(item: <Self::Lines as Iterator>::Item) -> Result<Self::Line, Error> {
        item
    }

    fn bytes(line: Self::Line) -> Self::Bytes {
        line.into_bytes().into_iter()
    }
}
