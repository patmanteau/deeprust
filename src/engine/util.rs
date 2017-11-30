// Inspired by https://python-chess.readthedocs.io/en/latest/core.html

use ::bits;

pub mod piece {
    pub const WHITE: u32 = 0;
    pub const BLACK: u32 = 1;

    pub const PAWN: u32 = 2;
    pub const KNIGHT: u32 = 3;
    pub const BISHOP: u32 = 4;
    pub const ROOK: u32 = 5;
    pub const QUEEN: u32 = 6;
    pub const KING: u32 = 7;
}

pub mod squares {
    macro_rules! msq {
        ($i:ident,$val:expr) => {
            pub const $i: u32 = $val;
        };
    }
    msq!(A1, 0); msq!(B1, 1); msq!(C1, 2); msq!(D1, 3); msq!(E1, 4); msq!(F1, 5); msq!(G1, 6); msq!(H1, 7);
    msq!(A2, 8); msq!(B2, 9); msq!(C2,10); msq!(D2,11); msq!(E2,12); msq!(F2,13); msq!(G2,14); msq!(H2,15);
    msq!(A3,16); msq!(B3,17); msq!(C3,18); msq!(D3,19); msq!(E3,20); msq!(F3,21); msq!(G3,22); msq!(H3,23);
    msq!(A4,24); msq!(B4,25); msq!(C4,26); msq!(D4,27); msq!(E4,28); msq!(F4,29); msq!(G4,30); msq!(H4,31);
    msq!(A5,32); msq!(B5,33); msq!(C5,34); msq!(D5,35); msq!(E5,36); msq!(F5,37); msq!(G5,38); msq!(H5,39);
    msq!(A6,40); msq!(B6,41); msq!(C6,42); msq!(D6,43); msq!(E6,44); msq!(F6,45); msq!(G6,46); msq!(H6,47);
    msq!(A7,48); msq!(B7,49); msq!(C7,50); msq!(D7,51); msq!(E7,52); msq!(F7,53); msq!(G7,54); msq!(H7,55);
    msq!(A8,56); msq!(B8,57); msq!(C8,58); msq!(D8,59); msq!(E8,60); msq!(F8,61); msq!(G8,62); msq!(H8,63);
}

pub const SQUARE_NAMES: [&'static str; 64] = [
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1",
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
    "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
];

pub const FILE_NAMES: [&'static str; 8] = [
    "a", "b", "c", "d", "e", "f", "g", "h", 
];

pub const RANK_NAMES: [&'static str; 8] = [
    "1", "2", "3", "4", "5", "6", "7", "8", 
];

/// Calculates square index from file and rank index
#[inline]
pub fn square(x: u32, y: u32) -> u32 {
    (y << 3) + x
}


mod tests {
    use super::*;

    #[test]
    fn it_makes_correct_squares() {

    }
}
