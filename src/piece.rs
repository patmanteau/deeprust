use crate::color;
use crate::common::*;

pub type Piece = u8;

pub const EMPTY: Piece = 0;
pub const PAWN: Piece = 2;
pub const KNIGHT: Piece = 3;
pub const BISHOP: Piece = 4;
pub const ROOK: Piece = 5;
pub const QUEEN: Piece = 6;
pub const KING: Piece = 7;

pub trait PiecePrimitives {
    fn new(piece: u8, color: u8) -> Self;
    fn empty() -> Self;
    fn from_char(c: char) -> Self;
    fn to_san_string(self) -> &'static str;
    fn code(self) -> Piece;
    fn color(self) -> color::Color;
}

impl PiecePrimitives for Piece {
    fn new(piece: u8, color: u8) -> Self {
        ((color & 0x1) << 3) | (piece & 0x7)
    }

    fn empty() -> Self {
        Self::new(EMPTY, color::WHITE)
    }

    fn from_char(c: char) -> Self {
        let (piece_code, color) = match c {
            'P' => (PAWN, color::WHITE),
            'N' => (KNIGHT, color::WHITE),
            'B' => (BISHOP, color::WHITE),
            'R' => (ROOK, color::WHITE),
            'Q' => (QUEEN, color::WHITE),
            'K' => (KING, color::WHITE),
            'p' => (PAWN, color::BLACK),
            'n' => (KNIGHT, color::BLACK),
            'b' => (BISHOP, color::BLACK),
            'r' => (ROOK, color::BLACK),
            'q' => (QUEEN, color::BLACK),
            'k' => (KING, color::BLACK),
            _ => unreachable!("Internal error: unknown piece code {}", c),
        };
        Self::new(piece_code, color)
    }

    fn to_san_string(self) -> &'static str {
        match self {
            2 => "P",
            3 => "N",
            4 => "B",
            5 => "R",
            6 => "Q",
            7 => "K",
            10 => "p",
            11 => "n",
            12 => "b",
            13 => "r",
            14 => "q",
            15 => "k",
            _ => " ",
        }
    }

    #[inline]
    fn code(self) -> Piece {
        self.extract_bits(0, 3)
    }

    #[inline]
    fn color(self) -> color::Color {
        self.extract_bits(3, 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_has_correct_piece_enum_values() {
        assert_eq!(2, PAWN);
        assert_eq!(3, KNIGHT);
        assert_eq!(4, BISHOP);
        assert_eq!(5, ROOK);
        assert_eq!(6, QUEEN);
        assert_eq!(7, KING);
    }
}
