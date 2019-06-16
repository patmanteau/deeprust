use common::*;
use square::{Square, SquarePrimitives};

pub type Bitboard = u64;

pub trait BitboardPrimitives {
    fn count(self) -> u32;
    fn scan(self) -> Square;
    
    fn to_debug_string(self) -> String;
}

impl BitboardPrimitives for Bitboard {
    #[inline]
    fn count(self) -> u32 {
        self.count_ones()
    }

    #[inline]
    fn scan(self) -> Square {
        self.trailing_zeros() as Square
    }

    fn to_debug_string(self) -> String {
        let mut out = String::new();

        out.push_str(&format!("DEBUG(bitboard): 0x{:016X}\n", self));

        out.push_str("+--------+\n");
        for i in (0..8).rev() {
            out.push_str("|");
            out.push_str(&self.to_debug_string_rank(i));
            out.push_str(&format!("|{}\n", i + 1));
        }
        out.push_str("+--------+\n abcdefgh\n");
        out
    }
}

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
