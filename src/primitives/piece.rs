use crate::common::*;
use crate::primitives::*;

pub type Piece = u8;
pub type PieceType = u8;

pub mod piece_types {
    use super::*;

    pub const EMPTY: PieceType = 0;
    pub const PAWN: PieceType = 2;
    pub const KNIGHT: PieceType = 3;
    pub const BISHOP: PieceType = 4;
    pub const ROOK: PieceType = 5;
    pub const QUEEN: PieceType = 6;
    pub const KING: PieceType = 7;
}

pub trait PiecePrimitives {
    fn new(piece: PieceType, color: Color) -> Self;
    fn empty() -> Self;
    fn from_char(c: char) -> Self;
    fn to_san_string(self) -> &'static str;
    fn code(self) -> PieceType;
    fn color(self) -> Color;
}

impl PiecePrimitives for Piece {
    fn new(piece: u8, color: u8) -> Self {
        ((color & 0x1) << 3) | (piece & 0x7)
    }

    fn empty() -> Self {
        Self::new(piece_types::EMPTY, colors::WHITE)
    }

    fn from_char(c: char) -> Self {
        let (piece_code, color) = match c {
            'P' => (piece_types::PAWN, colors::WHITE),
            'N' => (piece_types::KNIGHT, colors::WHITE),
            'B' => (piece_types::BISHOP, colors::WHITE),
            'R' => (piece_types::ROOK, colors::WHITE),
            'Q' => (piece_types::QUEEN, colors::WHITE),
            'K' => (piece_types::KING, colors::WHITE),
            'p' => (piece_types::PAWN, colors::BLACK),
            'n' => (piece_types::KNIGHT, colors::BLACK),
            'b' => (piece_types::BISHOP, colors::BLACK),
            'r' => (piece_types::ROOK, colors::BLACK),
            'q' => (piece_types::QUEEN, colors::BLACK),
            'k' => (piece_types::KING, colors::BLACK),
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
        assert_eq!(2, piece_types::PAWN);
        assert_eq!(3, piece_types::KNIGHT);
        assert_eq!(4, piece_types::BISHOP);
        assert_eq!(5, piece_types::ROOK);
        assert_eq!(6, piece_types::QUEEN);
        assert_eq!(7, piece_types::KING);
    }
}
