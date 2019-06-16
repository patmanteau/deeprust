// Inspired by https://python-chess.readthedocs.io/en/latest/core.html
#![allow(dead_code)]
use square::{Square, SquarePrimitives};

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

#[inline]
pub fn ep_capture_square(ep_square: Square) -> Square {
    let table = [
         0,  0,  0,  0,  0,  0,  0,  0, 
         0,  0,  0,  0,  0,  0,  0,  0, 
        24, 25, 26, 27, 28, 29, 30, 31,
         0,  0,  0,  0,  0,  0,  0,  0, 
         0,  0,  0,  0,  0,  0,  0,  0, 
        32, 33, 34, 35, 36, 37, 38, 39,
         0,  0,  0,  0,  0,  0,  0,  0, 
         0,  0,  0,  0,  0,  0,  0,  0, 
    ];
    table[ep_square as usize]
}

mod tests {
    use super::*;

    #[test]
    fn it_makes_correct_squares() {

    }
}
