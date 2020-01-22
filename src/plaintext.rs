use crate::Error;
use std::str::{Bytes, Lines};

pub struct Plaintext<'a> {
    lines: Lines<'a>,
    current_line: Bytes<'a>,
    x: i32,
    y: i32,
}

impl Plaintext<'_> {
    pub fn new(rle: &str) -> Plaintext {
        Plaintext {
            lines: rle.lines(),
            current_line: "".bytes(),
            x: -1,
            y: -1,
        }
    }
}

impl<'a> Iterator for Plaintext<'a> {
    type Item = Result<(i32, i32), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(c) = self.current_line.next() {
                match c {
                    b'O' => {
                        self.x += 1;
                        return Some(Ok((self.y, self.x)));
                    }
                    b'.' => self.x += 1,
                    _ => return Some(Err(Error::UnexpectedByte(c))),
                }
            } else if let Some(l) = self.lines.next() {
                if l.starts_with('!') {
                    continue;
                } else {
                    self.x = -1;
                    self.y += 1;
                    self.current_line = l.bytes();
                }
            } else {
                return None;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn glider_plaintext() -> Result<(), Error> {
        const GLIDER: &str = r"!Name: Glider
!Author: Richard K. Guy
!The smallest, most common, and first discovered spaceship.
!www.conwaylife.com/wiki/index.php?title=Glider
.O
..O
OOO";
        let mut glider = Plaintext::new(GLIDER).collect::<Result<Vec<_>, _>>()?;
        glider.sort();
        assert_eq!(glider, vec![(0, 1), (1, 2), (2, 0), (2, 1), (2, 2)]);
        Ok(())
    }

    #[test]
    fn twin_bees_shuttle_plaintext() -> Result<(), Error> {
        const TWIN_BEE_SHUTTLE: &str = r"!Name: Twin bees shuttle
!Author: Bill Gosper
!Twin bees shuttle was found in 1971 and is the basis of all known period 46 oscillators.
!www.conwaylife.com/wiki/index.php?title=Twin_bees_shuttle
.................OO
OO...............O.O.......OO
OO.................O.......OO
.................OOO



.................OOO
OO.................O
OO...............O.O
.................OO";
        let mut twin_bees_shuttle =
            Plaintext::new(TWIN_BEE_SHUTTLE).collect::<Result<Vec<_>, _>>()?;
        twin_bees_shuttle.sort();
        assert_eq!(
            twin_bees_shuttle,
            vec![
                (0, 17),
                (0, 18),
                (1, 0),
                (1, 1),
                (1, 17),
                (1, 19),
                (1, 27),
                (1, 28),
                (2, 0),
                (2, 1),
                (2, 19),
                (2, 27),
                (2, 28),
                (3, 17),
                (3, 18),
                (3, 19),
                (7, 17),
                (7, 18),
                (7, 19),
                (8, 0),
                (8, 1),
                (8, 19),
                (9, 0),
                (9, 1),
                (9, 17),
                (9, 19),
                (10, 17),
                (10, 18)
            ]
        );
        Ok(())
    }
}
