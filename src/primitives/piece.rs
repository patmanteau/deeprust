use crate::primitives::*;
use crate::common::*;

pub type Piece = u8;

pub mod pieces {
    use super::*;

    pub const EMPTY: Piece = 0;
    pub const PAWN: Piece = 2;
    pub const KNIGHT: Piece = 3;
    pub const BISHOP: Piece = 4;
    pub const ROOK: Piece = 5;
    pub const QUEEN: Piece = 6;
    pub const KING: Piece = 7;
}

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
        Self::new(pieces::EMPTY, colors::WHITE)
    }

    fn from_char(c: char) -> Self {
        let (piece_code, color) = match c {
            'P' => (pieces::PAWN, colors::WHITE),
            'N' => (pieces::KNIGHT, colors::WHITE),
            'B' => (pieces::BISHOP, colors::WHITE),
            'R' => (pieces::ROOK, colors::WHITE),
            'Q' => (pieces::QUEEN, colors::WHITE),
            'K' => (pieces::KING, colors::WHITE),
            'p' => (pieces::PAWN, colors::BLACK),
            'n' => (pieces::KNIGHT, colors::BLACK),
            'b' => (pieces::BISHOP, colors::BLACK),
            'r' => (pieces::ROOK, colors::BLACK),
            'q' => (pieces::QUEEN, colors::BLACK),
            'k' => (pieces::KING, colors::BLACK),
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
        assert_eq!(2, pieces::PAWN);
        assert_eq!(3, pieces::KNIGHT);
        assert_eq!(4, pieces::BISHOP);
        assert_eq!(5, pieces::ROOK);
        assert_eq!(6, pieces::QUEEN);
        assert_eq!(7, pieces::KING);
    }
}
