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

pub mod squares {
    #![allow(dead_code)]
    use square::{Square, SquarePrimitives};
    
    macro_rules! msq {
        ($($id:ident,$val:expr),*) => {
            $(pub const $id: Square = $val;)*
        };
    }

    msq!(A1, 0, B1, 1, C1, 2, D1, 3, E1, 4, F1, 5, G1, 6, H1, 7,
         A2, 8, B2, 9, C2,10, D2,11, E2,12, F2,13, G2,14, H2,15,
         A3,16, B3,17, C3,18, D3,19, E3,20, F3,21, G3,22, H3,23,
         A4,24, B4,25, C4,26, D4,27, E4,28, F4,29, G4,30, H4,31,
         A5,32, B5,33, C5,34, D5,35, E5,36, F5,37, G5,38, H5,39,
         A6,40, B6,41, C6,42, D6,43, E6,44, F6,45, G6,46, H6,47,
         A7,48, B7,49, C7,50, D7,51, E7,52, F7,53, G7,54, H7,55,
         A8,56, B8,57, C8,58, D8,59, E8,60, F8,61, G8,62, H8,63);

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

    pub const EP_CAPTURE_SQUARES: [usize; 64] = [
         0,  0,  0,  0,  0,  0,  0,  0, 
         0,  0,  0,  0,  0,  0,  0,  0, 
        24, 25, 26, 27, 28, 29, 30, 31,
         0,  0,  0,  0,  0,  0,  0,  0, 
         0,  0,  0,  0,  0,  0,  0,  0, 
        32, 33, 34, 35, 36, 37, 38, 39,
         0,  0,  0,  0,  0,  0,  0,  0, 
         0,  0,  0,  0,  0,  0,  0,  0, 
    ];

    // #[inline]
    // pub fn flip_square(square: Square) -> Square {
    //     square ^ 56
    // }
}

pub mod bb {
    #![allow(dead_code)]
    
    use std::fmt;
    use lazy_static;
    use common::*;
    use bitboard::*;
    use util::squares;
    use square::{Square, SquarePrimitives};

    macro_rules! mbb_squares {
        ($($bb_id:ident,$square:expr),*) => {
            pub const BB_SQUARES: [Bitboard; 64] = [
                $(1u64 << $square),*
            ];

            $(pub const $bb_id: Bitboard = 1u64 << $square;)*
        };
    }

    macro_rules! mbb_ranks {
        ($($bb_id:ident,$rank:expr),*) => {
            pub const BB_RANKS: [Bitboard; 8] = [
                $(0xffu64 << ($rank * 8)),*
            ];

            $(pub const $bb_id: Bitboard = 0xffu64 << ($rank * 8);)*
        };
    }

    macro_rules! mbb_files {
        ($($bb_id:ident,$file:expr),*) => {
            pub const BB_FILES: [u64; 8] = [
                $(0x0101010101010101u64 << $file),*
            ];

            $(pub const $bb_id: Bitboard = 0x0101010101010101u64 << $file;)*
        };
    }

    mbb_squares!(BB_A1, 0, BB_B1, 1, BB_C1, 2, BB_D1, 3, BB_E1, 4, BB_F1, 5, BB_G1, 6, BB_H1, 7,
                 BB_A2, 8, BB_B2, 9, BB_C2,10, BB_D2,11, BB_E2,12, BB_F2,13, BB_G2,14, BB_H2,15,
                 BB_A3,16, BB_B3,17, BB_C3,18, BB_D3,19, BB_E3,20, BB_F3,21, BB_G3,22, BB_H3,23,
                 BB_A4,24, BB_B4,25, BB_C4,26, BB_D4,27, BB_E4,28, BB_F4,29, BB_G4,30, BB_H4,31,
                 BB_A5,32, BB_B5,33, BB_C5,34, BB_D5,35, BB_E5,36, BB_F5,37, BB_G5,38, BB_H5,39,
                 BB_A6,40, BB_B6,41, BB_C6,42, BB_D6,43, BB_E6,44, BB_F6,45, BB_G6,46, BB_H6,47,
                 BB_A7,48, BB_B7,49, BB_C7,50, BB_D7,51, BB_E7,52, BB_F7,53, BB_G7,54, BB_H7,55,
                 BB_A8,56, BB_B8,57, BB_C8,58, BB_D8,59, BB_E8,60, BB_F8,61, BB_G8,62, BB_H8,63);

    mbb_ranks!  (BB_RANK_1, 0, BB_RANK_2, 1, BB_RANK_3, 2, BB_RANK_4, 3, 
                 BB_RANK_5, 4, BB_RANK_6, 5, BB_RANK_7, 6, BB_RANK_8, 7);

    mbb_files!  (BB_FILE_A, 0, BB_FILE_B, 1, BB_FILE_C, 2, BB_FILE_D, 3, 
                 BB_FILE_E, 4, BB_FILE_F, 5, BB_FILE_G, 6, BB_FILE_H, 7);

    pub const BB_DARK_SQUARES: Bitboard = 0xaa55aa55aa55aa55u64;
    pub const BB_LIGHT_SQUARES: Bitboard = 0x55aa55aa55aa55aau64;
    
    pub const BB_BACKRANKS: Bitboard = BB_RANK_1 | BB_RANK_8;
    pub const BB_CORNERS: Bitboard = BB_A1 | BB_H1 | BB_A8 | BB_H8;

    pub const BB_EMPTY: Bitboard = 0u64;
    pub const BB_ALL: Bitboard = 0xffffffffffffffffu64;

    pub const BB_NOT_FILE_A: Bitboard = !BB_FILE_A;
    pub const BB_NOT_FILE_H: Bitboard = !BB_FILE_H;
    pub const BB_NOT_FILE_AB: Bitboard = !(BB_FILE_A | BB_FILE_B);
    pub const BB_NOT_FILE_GH: Bitboard = !(BB_FILE_G | BB_FILE_H);
    
    pub fn north_one(bb: Bitboard) -> Bitboard        { bb << 8 }
    pub fn north_east_one(bb: Bitboard) -> Bitboard   { (bb & BB_NOT_FILE_H) << 9 }
    pub fn east_one(bb: Bitboard) -> Bitboard         { (bb & BB_NOT_FILE_H) << 1 }
    pub fn south_east_one(bb: Bitboard) -> Bitboard   { (bb & BB_NOT_FILE_H) >> 7 }
    pub fn south_one(bb: Bitboard) -> Bitboard        { bb >> 8 }
    pub fn south_west_one(bb: Bitboard) -> Bitboard   { (bb & BB_NOT_FILE_A) >> 9 }
    pub fn west_one(bb: Bitboard) -> Bitboard         { (bb & BB_NOT_FILE_A) >> 1 }
    pub fn north_west_one(bb: Bitboard) -> Bitboard   { (bb & BB_NOT_FILE_A) << 7 }

    /// see https://chessprogramming.wikispaces.com/Flipping+Mirroring+and+Rotating
    pub fn flip_diag_a1h8(mut bb: Bitboard) -> Bitboard {
        let k1 = 0x5500550055005500;
        let k2 = 0x3333000033330000;
        let k4 = 0x0f0f0f0f00000000;
       
        let mut t = k4 & (bb ^ (bb << 28));
        bb = bb ^ (t ^ (t >> 28));
        t = k2 & (bb ^ (bb << 14));
        bb = bb ^ (t ^ (t >> 14));
        t = k1 & (bb ^ (bb <<  7));
        bb = bb ^ (t ^ (t >>  7));
        bb
    }

    lazy_static! {
        pub static ref BB_KNIGHT_ATTACKS: [Bitboard; 64] = {
            let mut arr: [Bitboard; 64] = [0; 64];
            for i in 0..64 {
                let orig_bb = BB_SQUARES[i];
                arr[i] = 
                    north_one(north_west_one(orig_bb)) |
                    north_one(north_east_one(orig_bb)) |
                    east_one(north_east_one(orig_bb)) |
                    east_one(south_east_one(orig_bb)) |
                    south_one(south_east_one(orig_bb)) |
                    south_one(south_west_one(orig_bb)) |
                    west_one(south_west_one(orig_bb)) |
                    west_one(north_west_one(orig_bb));
            }
            arr
        };

        pub static ref BB_KING_ATTACKS: [Bitboard; 64] = {
            let mut arr: [Bitboard; 64] = [0; 64];
            for i in 0..64 {
                let orig_bb = BB_SQUARES[i];
                arr[i] =
                    north_one(orig_bb) | north_east_one(orig_bb) |
                    east_one(orig_bb) | south_east_one(orig_bb) |
                    south_one(orig_bb) | south_west_one(orig_bb) |
                    west_one(orig_bb) | north_west_one(orig_bb);
            }
            arr
        };

        pub static ref BB_DIAG: [Bitboard; 64] = {
            let mut arr: [Bitboard; 64] = [0; 64];
            for i in 0i64..64i64 {
                let main_diag = 0x8040201008040201u64;
                let diag = 8 * (i & 7) - (i & 56);
                let north = -diag & (diag >> 31);
                let south = diag & (-diag >> 31);

                arr[i as usize] = (main_diag >> south) << north;
            }
            arr
        };

        pub static ref BB_ANTI_DIAG: [Bitboard; 64] = {
            let mut arr: [Bitboard; 64] = [0; 64];
            for i in 0i64..64i64 {
                let main_diag = 0x0102040810204080u64;
                let diag = 56 - 8 * (i & 7) - (i & 56);
                let north = -diag & (diag >> 31);
                let south = diag & (-diag >> 31);

                arr[i as usize] = (main_diag >> south) << north;
            }
            arr
        };

        pub static ref BB_BISHOP_ATTACKS: [Bitboard; 64] = {
            let mut arr: [Bitboard; 64] = [0; 64];
            for i in 0..64 {
                arr[i] = (BB_DIAG[i] | BB_ANTI_DIAG[i]) ^ BB_SQUARES[i];
            }
            arr
        };

        pub static ref BB_ROOK_ATTACKS: [Bitboard; 64] = {
            let mut arr: [Bitboard; 64] = [0; 64];
            for i in 0..64 {
                arr[i] = (BB_RANKS[i >> 3] | BB_FILES[i & 7]) ^ BB_SQUARES[i];
            }
            arr
        };

        pub static ref BB_QUEEN_ATTACKS: [Bitboard; 64] = {
            let mut arr: [Bitboard; 64] = [0; 64];
            for i in 0..64 {
                arr[i] = BB_BISHOP_ATTACKS[i] | BB_ROOK_ATTACKS[i];
            }
            arr
        };

        /// Attack rays for cardinal direction and square
        pub static ref BB_RAYS_WEST: [Bitboard; 64] = {
            let mut arr: [Bitboard; 64] = [0; 64];
            for i in 1..64 {
                arr[i] = ((1u64 << i) - 1) & BB_RANKS[i >> 3];
            }
            arr
        };

        pub static ref BB_RAYS_EAST: [Bitboard; 64] = {
            let mut arr: [Bitboard; 64] = [0; 64];
            for i in 0..63 {
                arr[i] = ((BB_ALL ^ ((1u64 << i) - 1)) & BB_RANKS[i >> 3]) ^ BB_SQUARES[i];
            }
            arr
        };

        pub static ref BB_FIRST_RANK_ATTACKS: [[Bitboard; 64]; 8] = {
            let mut arr: [[Bitboard; 64]; 8] = [[0; 64]; 8];
            for sq in 0..8 {
                for occ in 0..64 {
                    let mut east_attacks = BB_RAYS_EAST[sq];
                    let east_blocker = east_attacks & (occ << 1);
                    if 0 < east_blocker {
                        let blocker_square = east_blocker.trailing_zeros();
                        east_attacks ^= BB_RAYS_EAST[blocker_square as usize];
                    }

                    let mut west_attacks = BB_RAYS_WEST[sq];
                    let west_blocker: u8 = ((west_attacks & (occ << 1)) & 0xff) as u8;
                    if 0 < west_blocker {
                        let blocker_square = 7 - west_blocker.leading_zeros();
                        west_attacks ^= BB_RAYS_WEST[blocker_square as usize];
                    }
                    arr[sq as usize][occ as usize] = east_attacks | west_attacks;
                }
            }
            arr
        };

        pub static ref BB_A_FILE_ATTACKS: [[Bitboard; 64]; 8] = {
            let diag_a1h8: Bitboard = 0x8040201008040201;
            let mut arr: [[Bitboard; 64]; 8] = [[0; 64]; 8];
            for sq in 0..8 {
                for occ in 0..64 {
                    // arr[sq as usize][occ as usize] = bits::swap_bytes(flip_diag_a1h8(BB_FIRST_RANK_ATTACKS[sq as usize][occ as usize]));
                    arr[sq as usize][occ as usize] = flip_diag_a1h8(BB_FIRST_RANK_ATTACKS[sq as usize][occ as usize]);
                    // arr[sq as usize][occ as usize] = (diag_a1h8.overflowing_mul(BB_FIRST_RANK_ATTACKS[sq as usize][occ as usize]).0 >> 0x7) & BB_FILE_A;
                }
            }
            arr
        };

        pub static ref BB_KG_FILL_UP_ATTACKS: [[Bitboard; 64]; 8] = {
            let mut arr: [[Bitboard; 64]; 8] = [[0; 64]; 8];
            for sq in 0..8 {
                for occ in 0..64 {
                    arr[sq as usize][occ as usize] = 
                        BB_FIRST_RANK_ATTACKS[sq as usize][occ as usize] |
                        (BB_FIRST_RANK_ATTACKS[sq as usize][occ as usize] << 8) |
                        (BB_FIRST_RANK_ATTACKS[sq as usize][occ as usize] << 16) |
                        (BB_FIRST_RANK_ATTACKS[sq as usize][occ as usize] << 24) |
                        (BB_FIRST_RANK_ATTACKS[sq as usize][occ as usize] << 32) |
                        (BB_FIRST_RANK_ATTACKS[sq as usize][occ as usize] << 40) |
                        (BB_FIRST_RANK_ATTACKS[sq as usize][occ as usize] << 48) |
                        (BB_FIRST_RANK_ATTACKS[sq as usize][occ as usize] << 56);
                }
            }
            arr
        };
    }

    /// See https://chessprogramming.wikispaces.com/Kindergarten+Bitboards
    pub fn diagonal_attacks(square: Square, mut occupied: Bitboard) -> Bitboard {
        let diag_mask_ex = BB_DIAG[square as usize] ^ BB_SQUARES[square as usize];
        let north_fill = (diag_mask_ex & occupied).overflowing_mul(BB_FILE_B);
        occupied = north_fill.0 >> 58;
        diag_mask_ex & BB_KG_FILL_UP_ATTACKS[(square & 0x7) as usize][occupied as usize]
    }

    pub fn anti_diagonal_attacks(square: Square, mut occupied: Bitboard) -> Bitboard {
        let anti_diag_mask_ex = BB_ANTI_DIAG[square as usize] ^ BB_SQUARES[square as usize];
        let north_fill = (anti_diag_mask_ex & occupied).overflowing_mul(BB_FILE_B);
        occupied = north_fill.0 >> 58;
        anti_diag_mask_ex & BB_KG_FILL_UP_ATTACKS[(square & 0x7) as usize][occupied as usize]
    }

    pub fn rank_attacks(square: Square, mut occupied: Bitboard) -> Bitboard {
        let rank_mask_ex = BB_RANKS[(square >> 0x3) as usize] ^ BB_SQUARES[square as usize];
        let north_fill = (rank_mask_ex & occupied).overflowing_mul(BB_FILE_B);
        occupied = north_fill.0 >> 58;
        rank_mask_ex & BB_KG_FILL_UP_ATTACKS[(square & 0x7) as usize][occupied as usize]
    }

    pub fn file_attacks(square: Square, mut occupied: Bitboard) -> Bitboard {
        let diag_c2h7: Bitboard = 0x0080402010080400;
        let diag_c7h2: Bitboard = diag_c2h7.swap_bytes();
        occupied = BB_FILE_A & (occupied >> (square & 0x7));
        occupied = (diag_c7h2.overflowing_mul(occupied).0) >> 58;
        BB_A_FILE_ATTACKS[(square >> 0x3) as usize][occupied as usize] << (square & 0x7)
    }
}


// /// Calculates square index from file and rank index
// #[inline]
// pub fn square_from_coords(x: u32, y: u32) -> Square {
//     (y << 3) + x
// }

/// Returns the captured piece's square in an en passant capture
/// 
/// # Arguments
/// 
/// * `ep_square` - An en passant capture target square
/// 
/// # Example
/// 
/// ```
/// assert_eq!(squares::D4, lookup_ep_capture(squares::D3));
/// ```
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
