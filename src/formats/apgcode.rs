use crate::Error;
use std::str::Bytes;

pub struct ApgCode<'a> {
    /// Extended Wechsler format
    ewf: Bytes<'a>,
    current_strip: u8,
    position: i8,
    x: i32,
    y: i32,
}

impl ApgCode<'_> {
    pub fn new(text: &str) -> Result<ApgCode, Error> {
        let mut split = text.split('_');
        let prefix = split.next().ok_or(Error::Unencodable)?;
        match &prefix[..2] {
            "xs" | "xp" | "xq" => {
                for d in prefix[2..].bytes() {
                    if !d.is_ascii_digit() {
                        return Err(Error::UnexpectedByte(d));
                    }
                }
            }
            _ => return Err(Error::Unencodable),
        }
        Ok(ApgCode {
            ewf: split.next().ok_or(Error::Unencodable)?.bytes(),
            current_strip: 0,
            position: -1,
            x: -1,
            y: 0,
        })
    }
}

impl<'a> Iterator for ApgCode<'a> {
    type Item = Result<(i32, i32), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.position >= 0 {
                loop {
                    if (self.current_strip >> self.position) & 1 == 0 {
                        self.position += 1;
                        if self.position == 5 {
                            self.position = -1;
                            break;
                        }
                    } else {
                        let position = self.position;
                        self.position += 1;
                        if self.position == 5 {
                            self.position = -1;
                        }
                        return Some(Ok((self.x, self.y + position as i32)));
                    }
                }
            } else if let Some(c) = self.ewf.next() {
                self.current_strip = match c {
                    b'0'..=b'9' => c - b'0',
                    b'a'..=b'v' => c - b'a' + 10,
                    b'w' => {
                        self.x += 2;
                        continue;
                    }
                    b'x' => {
                        self.x += 3;
                        continue;
                    }
                    b'y' => {
                        if let Some(c) = self.ewf.next() {
                            let n = match c {
                                b'0'..=b'9' => c - b'0',
                                b'a'..=b'z' => c - b'a' + 10,
                                _ => return Some(Err(Error::UnexpectedByte(c))),
                            };
                            self.x += 4 + n as i32;
                            continue;
                        } else {
                            return Some(Err(Error::UnexpectedByte(b'y')));
                        }
                    }
                    b'z' => {
                        self.x = -1;
                        self.y += 5;
                        continue;
                    }
                    _ => return Some(Err(Error::UnexpectedByte(c))),
                };
                self.x += 1;
                self.position = 0;
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
    fn glider_apgcode() -> Result<(), Error> {
        const GLIDER: &str = "xq4_153";
        let mut glider = ApgCode::new(GLIDER)?.collect::<Result<Vec<_>, _>>()?;
        glider.sort();
        assert_eq!(glider, vec![(0, 0), (1, 0), (1, 2), (2, 0), (2, 1)]);
        Ok(())
    }

    #[test]
    fn twin_bees_shuttle_apgcode() -> Result<(), Error> {
        const TWIN_BEE_SHUTTLE: &str = "xp46_033y133zzzckgsxsgkczz0cc";
        let mut twin_bees_shuttle =
            ApgCode::new(TWIN_BEE_SHUTTLE)?.collect::<Result<Vec<_>, _>>()?;
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
