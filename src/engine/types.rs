pub enum Piece {
    White,
    Black,
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

pub enum Color {
    White,
    Black,
}

#[derive(Debug, Copy, Clone)]
pub struct Move {
    m: u16
}

impl Move {
    #[inline]
    pub fn new(from: u16, to: u16) -> Move {
        Move {
            m: ((from & 0x3f) << 10) | (to & 0x3f),
        }
    }

    #[inline]
    pub fn set_from(&mut self, from: u16) {
        self.m &= !0xfc00;
        self.m |= (from & 0x3f) << 10;
    }

    #[inline]
    pub fn set_to(&mut self, to: u16) {
        self.m &= !0x3f;
        self.m |= to & 0x3f;
    }

    #[inline]
    pub fn get_from(&self) -> u16 {
        (self.m >> 10) & 0x3f
    }

    #[inline]
    pub fn get_to(&self) -> u16 {
        (self.m & 0x3f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_moves() {
        for from in 0..64 {
            for to in 0..64 {
                let mut mov = Move::new(from, to);
                assert_eq!(from, mov.get_from());
                assert_eq!(to, mov.get_to());

                mov.set_from(63-from);
                mov.set_to(63-to);
                assert_eq!(63-from, mov.get_from());
                assert_eq!(63-to, mov.get_to());
            }
        }
    }
}