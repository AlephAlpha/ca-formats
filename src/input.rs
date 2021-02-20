use std::{
    io::{BufRead, BufReader, Error, Lines as IoLines, Read},
    str::{Bytes, Lines},
    vec::IntoIter,
};

/// Types that can be passed to parsers as input.
///
/// The trait is implemented for [`&str`](str) and [`BufReader`].
/// When parsing a file, you can take a [`BufReader<File>`] as input.
pub trait Input {
    /// An iterator over lines of the input.
    type Lines: Iterator;
    /// A string or a reference to a string, which represents a line of the input.
    type Line: AsRef<str>;
    /// An iterator over bytes of a line.
    type Bytes: Iterator<Item = u8>;

    /// Creates an iterator over lines from the input.
    fn lines(self) -> Self::Lines;

    /// Converts a item in the lines iterator to a string.
    fn line(item: <Self::Lines as Iterator>::Item) -> Result<Self::Line, Error>;

    /// Creates an iterator over bytes from a line.
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
