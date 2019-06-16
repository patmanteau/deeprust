pub type Piece = u8;

pub const PAWN: Piece = 2;
pub const KNIGHT: Piece = 3;
pub const BISHOP: Piece = 4;
pub const ROOK: Piece = 5;
pub const QUEEN: Piece = 6;
pub const KING: Piece = 7;

pub trait PiecePrimitives {
    fn to_san_string(self) -> &'static str;
}

impl PiecePrimitives for Piece {
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
}
