use crate::common::*;
use crate::primitives::Square;

use std::iter::Iterator;

pub type Bitboard = u64;

pub struct BitboardIter(Bitboard);

impl Iterator for BitboardIter {
    type Item = Square;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if 0 < self.0 {
            let p = self.0.scan();
            self.0.clear(p);
            Some(p)
        } else {
            None
        }
    }
}

pub trait BitboardPrimitives<T> {
    fn count(self) -> u32;
    fn scan(self) -> Square;

    fn test(self, pos: Square) -> bool;
    fn set(&mut self, pos: Square);
    fn clear(&mut self, pos: Square);

    fn iter(&mut self) -> BitboardIter;

    fn rank_to_debug_string(self, rank: u32) -> String;
    fn to_debug_string(self) -> String;
}

impl BitboardPrimitives<u64> for Bitboard {
    #[inline]
    fn count(self) -> u32 {
        self.count_ones()
    }

    // TODO: scan and reset LSB
    #[inline]
    fn scan(self) -> Square {
        self.trailing_zeros() as Square
    }

    #[inline]
    fn test(self, pos: Square) -> bool {
        //self.test_bit(pos as u32)
        self.test_bit(pos)
    }

    #[inline]
    fn set(&mut self, pos: Square) {
        *self |= 1_u64 << pos;
        // *self |= BB_SQUARES[pos as usize]
        //self.set_bit(pos as u32)
    }

    #[inline]
    fn clear(&mut self, pos: Square) {
        //self.clear_bit(pos as u32)
        // *self ^= BB_SQUARES[pos as usize]
        *self ^= 1_u64 << pos;
    }

    #[inline]
    fn iter(&mut self) -> BitboardIter {
        BitboardIter(*self)
    }

    fn rank_to_debug_string(self, rank: u32) -> String {
        debug_assert!(rank < 8);
        //format!("{:08b}", (self.extract_bits(rank * 8, 8) as u8).reverse_bits())

        let mut b = self.extract_bits(rank * 8, 8) as u8;
        b = (b & 0xF0) >> 4 | (b & 0x0F) << 4;
        b = (b & 0xCC) >> 2 | (b & 0x33) << 2;
        b = (b & 0xAA) >> 1 | (b & 0x55) << 1;
        format!("{:08b}", b)
    }

    fn to_debug_string(self) -> String {
        let mut out = String::new();

        out.push_str(&format!("DEBUG(bitboard): 0x{:016X}\n", self));

        out.push_str("+--------+\n");
        for i in (0..8).rev() {
            out.push_str("|");
            out.push_str(&self.rank_to_debug_string(i));
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
        pub const BB_FILES: [Bitboard; 8] = [
            $(0x0101_0101_0101_0101u64 << $file),*
        ];

        $(pub const $bb_id: Bitboard = 0x0101_0101_0101_0101u64 << $file;)*
    };
}

pub mod bitboards {
    use super::*;
    #[rustfmt::skip]
    mbb_squares!(BB_A1, 0, BB_B1, 1, BB_C1, 2, BB_D1, 3, BB_E1, 4, BB_F1, 5, BB_G1, 6, BB_H1, 7,
                BB_A2, 8, BB_B2, 9, BB_C2,10, BB_D2,11, BB_E2,12, BB_F2,13, BB_G2,14, BB_H2,15,
                BB_A3,16, BB_B3,17, BB_C3,18, BB_D3,19, BB_E3,20, BB_F3,21, BB_G3,22, BB_H3,23,
                BB_A4,24, BB_B4,25, BB_C4,26, BB_D4,27, BB_E4,28, BB_F4,29, BB_G4,30, BB_H4,31,
                BB_A5,32, BB_B5,33, BB_C5,34, BB_D5,35, BB_E5,36, BB_F5,37, BB_G5,38, BB_H5,39,
                BB_A6,40, BB_B6,41, BB_C6,42, BB_D6,43, BB_E6,44, BB_F6,45, BB_G6,46, BB_H6,47,
                BB_A7,48, BB_B7,49, BB_C7,50, BB_D7,51, BB_E7,52, BB_F7,53, BB_G7,54, BB_H7,55,
                BB_A8,56, BB_B8,57, BB_C8,58, BB_D8,59, BB_E8,60, BB_F8,61, BB_G8,62, BB_H8,63);

    #[rustfmt::skip]
    mbb_ranks!  (BB_RANK_1, 0, BB_RANK_2, 1, BB_RANK_3, 2, BB_RANK_4, 3,
                BB_RANK_5, 4, BB_RANK_6, 5, BB_RANK_7, 6, BB_RANK_8, 7);

    #[rustfmt::skip]
    mbb_files!  (BB_FILE_A, 0, BB_FILE_B, 1, BB_FILE_C, 2, BB_FILE_D, 3,
                BB_FILE_E, 4, BB_FILE_F, 5, BB_FILE_G, 6, BB_FILE_H, 7);

    pub const BB_DIAG_A1H8: Bitboard = 0x8040_2010_0804_0201;
    pub const BB_DIAG_A8H1: Bitboard = 0x0102_0408_1020_4080;

    pub const BB_DARK_SQUARES: Bitboard = 0xaa55_aa55_aa55_aa55u64;
    pub const BB_LIGHT_SQUARES: Bitboard = 0x55aa_55aa_55aa_55aau64;

    pub const BB_BACKRANKS: Bitboard = BB_RANK_1 | BB_RANK_8;
    pub const BB_CORNERS: Bitboard = BB_A1 | BB_H1 | BB_A8 | BB_H8;

    pub const BB_EMPTY: Bitboard = 0u64;
    pub const BB_ALL: Bitboard = 0xffff_ffff_ffff_ffffu64;

    pub const BB_NOT_FILE_A: Bitboard = !BB_FILE_A;
    pub const BB_NOT_FILE_H: Bitboard = !BB_FILE_H;
    pub const BB_NOT_FILE_AB: Bitboard = !(BB_FILE_A | BB_FILE_B);
    pub const BB_NOT_FILE_GH: Bitboard = !(BB_FILE_G | BB_FILE_H);

    pub const BB_NOT_RANK_1: Bitboard = !BB_RANK_1;
    pub const BB_NOT_RANK_2: Bitboard = !BB_RANK_8;
    pub const BB_NOT_RANK_12: Bitboard = !BB_RANK_2 & !BB_RANK_1;
    pub const BB_NOT_RANK_78: Bitboard = !BB_RANK_7 & !BB_RANK_8;

    pub const BB_ROOK_HOMES: [Bitboard; 2] = [BB_A1 | BB_H1, BB_A8 | BB_H8];

    lazy_static! {
        pub static ref BB_WPAWN_ATTACKS: [Bitboard; 64] = {
            let mut arr: [Bitboard; 64] = [0; 64];
            for i in 0..56 {
                let orig_bb = BB_SQUARES[i];
                arr[i] = north_west_one(orig_bb) | north_east_one(orig_bb);
            }
            arr
        };

        pub static ref BB_BPAWN_ATTACKS: [Bitboard; 64] = {
            let mut arr: [Bitboard; 64] = [0; 64];
            for i in 8..64 {
                let orig_bb = BB_SQUARES[i];
                arr[i] = south_west_one(orig_bb) | south_east_one(orig_bb);
            }
            arr
        };

        pub static ref BB_PAWN_ATTACKS_2: [[Bitboard; 64]; 2] = {
            let mut arr: [[Bitboard; 64]; 2] = [[0; 64]; 2];
            arr[0] = *BB_WPAWN_ATTACKS;
            arr[1] = *BB_BPAWN_ATTACKS;
            arr
        };

        pub static ref BB_WPAWN_PUSHES: [Bitboard; 64] = {
            let mut arr: [Bitboard; 64] = [0; 64];
            for i in 8..16 {
                let orig_bb = BB_SQUARES[i];
                arr[i] = north_one(orig_bb);
                arr[i] |= north_one(arr[i]);
            }
            for i in 16..56 {
                let orig_bb = BB_SQUARES[i];
                arr[i] = north_one(orig_bb);
            }
            arr
        };

        pub static ref BB_BPAWN_PUSHES: [Bitboard; 64] = {
            let mut arr: [Bitboard; 64] = [0; 64];
            for i in 8..48 {
                let orig_bb = BB_SQUARES[i];
                arr[i] = south_one(orig_bb);
            }
            for i in 48..56 {
                let orig_bb = BB_SQUARES[i];
                arr[i] = south_one(orig_bb);
                arr[i] |= south_one(arr[i]);
            }
            arr
        };

        pub static ref BB_PAWN_PUSHES: [[Bitboard; 64]; 2] = {
            let mut arr: [[Bitboard; 64]; 2] = [[0; 64]; 2];
            arr[0] = *BB_WPAWN_PUSHES;
            arr[1] = *BB_BPAWN_PUSHES;
            arr
        };

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
                let diag = 8 * (i & 7) - (i & 56);
                let north = -diag & (diag >> 31);
                let south = diag & (-diag >> 31);

                arr[i as usize] = (BB_DIAG_A1H8 >> south) << north;
            }
            arr
        };

        pub static ref BB_DIAG_MASK_EX: [Bitboard; 64] = {
            let mut arr: [Bitboard; 64] = [0; 64];
            for i in 0..64 {
                arr[i] = BB_DIAG[i] ^ BB_SQUARES[i];
            }
            arr
        };

        pub static ref BB_ANTI_DIAG: [Bitboard; 64] = {
            let mut arr: [Bitboard; 64] = [0; 64];
            for i in 0i64..64i64 {
                let diag = 56 - 8 * (i & 7) - (i & 56);
                let north = -diag & (diag >> 31);
                let south = diag & (-diag >> 31);

                arr[i as usize] = (BB_DIAG_A8H1 >> south) << north;
            }
            arr
        };

        pub static ref BB_ANTI_DIAG_MASK_EX: [Bitboard; 64] = {
            let mut arr: [Bitboard; 64] = [0; 64];
            for i in 0..64 {
                arr[i] = BB_ANTI_DIAG[i] ^ BB_SQUARES[i];
            }
            arr
        };

        pub static ref BB_RANK_MASK_EX: [Bitboard; 64] = {
            let mut arr: [Bitboard; 64] = [0; 64];
            for i in 0..64 {
                arr[i] = BB_RANKS[(i >> 0x3) as usize] ^ BB_SQUARES[i as usize];
            }
            arr
        };

        pub static ref BB_FILE_MASK_EX: [Bitboard; 64] = {
            let mut arr: [Bitboard; 64] = [0; 64];
            for i in 0..64 {
                arr[i] = BB_FILES[(i & 7) as usize] ^ BB_SQUARES[i as usize];
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
            let mut arr: [[Bitboard; 64]; 8] = [[0; 64]; 8];
            for sq in 0..8 {
                for occ in 0..64 {
                    arr[sq as usize][occ as usize] = flip_diag_a1h8(BB_FIRST_RANK_ATTACKS[sq as usize][occ as usize]);
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

    // SSSE3 specifics
    // #[cfg(target_arch = "x86_64")]
    // use std::arch::x86_64::*;
    // #[cfg(
    //     all(
    //         any(target_arch = "x86", target_arch = "x86_64"),
    //         target_feature = "avx2"
    //     )
    // )]
    use std::arch::x86_64::*;
    lazy_static! {
        pub static ref BB_SSSE3_DIAG_MASK_EX: [__m128i; 64] = {
            unsafe {
                let zero = _mm_setzero_si128();//_mm_set_epi64x(0, 0);
                let mut arr: [__m128i; 64] = [zero; 64];

                for i in 0..64 {
                    arr[i] = _mm_set_epi64x(
                        std::mem::transmute::<u64, i64>(BB_ANTI_DIAG_MASK_EX[i]),
                        std::mem::transmute::<u64, i64>(BB_DIAG_MASK_EX[i])
                    );
                }
                arr
            }
        };

        pub static ref BB_SSSE3_SQUARE: [__m128i; 64] = {
            unsafe {
                let zero = _mm_setzero_si128();//_mm_set_epi64x(0, 0);
                let mut arr: [__m128i; 64] = [zero; 64];

                for (i, sq) in arr.iter_mut().enumerate() {
                    *sq = _mm_set_epi64x(
                        1 << i,
                        1 << i,
                    );
                }
                arr
            }
        };

        pub static ref BB_SSSE3_SWAP_MASK: __m128i = {
            unsafe {
                _mm_set_epi8(
                    8,  9, 10, 11,
                    12, 13, 14, 15,
                    0,  1,  2,  3,
                    4,  5,  6,  7
                )
            }
        };
    }

    // TODO: Pawn attack and push tables
    #[rustfmt::skip] #[inline] pub const fn north_one(bb: Bitboard) -> Bitboard      { bb << 8 }
    #[rustfmt::skip] #[inline] pub const fn north_east_one(bb: Bitboard) -> Bitboard { (bb & BB_NOT_FILE_H) << 9 }
    #[rustfmt::skip] #[inline] pub const fn east_one(bb: Bitboard) -> Bitboard       { (bb & BB_NOT_FILE_H) << 1 }
    #[rustfmt::skip] #[inline] pub const fn south_east_one(bb: Bitboard) -> Bitboard { (bb & BB_NOT_FILE_H) >> 7 }
    #[rustfmt::skip] #[inline] pub const fn south_one(bb: Bitboard) -> Bitboard      { bb >> 8 }
    #[rustfmt::skip] #[inline] pub const fn south_west_one(bb: Bitboard) -> Bitboard { (bb & BB_NOT_FILE_A) >> 9 }
    #[rustfmt::skip] #[inline] pub const fn west_one(bb: Bitboard) -> Bitboard       { (bb & BB_NOT_FILE_A) >> 1 }
    #[rustfmt::skip] #[inline] pub const fn north_west_one(bb: Bitboard) -> Bitboard { (bb & BB_NOT_FILE_A) << 7 }

    /// see https://chessprogramming.org/Flipping_Mirroring_and_Rotating
    pub const fn flip_diag_a1h8(mut bb: Bitboard) -> Bitboard {
        let k1 = 0x5500_5500_5500_5500;
        let k2 = 0x3333_0000_3333_0000;
        let k4 = 0x0f0f_0f0f_0000_0000;

        let mut t = k4 & (bb ^ (bb << 28));
        bb ^= t ^ (t >> 28);
        t = k2 & (bb ^ (bb << 14));
        bb ^= t ^ (t >> 14);
        t = k1 & (bb ^ (bb << 7));
        bb ^= t ^ (t >> 7);
        bb
    }

    pub const BB_PAWN_ATTACKS: [[Bitboard; 64]; 2] = [
        [
            0x0000000000000200,
            0x0000000000000500,
            0x0000000000000A00,
            0x0000000000001400,
            0x0000000000002800,
            0x0000000000005000,
            0x000000000000A000,
            0x0000000000004000,
            0x0000000000020000,
            0x0000000000050000,
            0x00000000000A0000,
            0x0000000000140000,
            0x0000000000280000,
            0x0000000000500000,
            0x0000000000A00000,
            0x0000000000400000,
            0x0000000002000000,
            0x0000000005000000,
            0x000000000A000000,
            0x0000000014000000,
            0x0000000028000000,
            0x0000000050000000,
            0x00000000A0000000,
            0x0000000040000000,
            0x0000000200000000,
            0x0000000500000000,
            0x0000000A00000000,
            0x0000001400000000,
            0x0000002800000000,
            0x0000005000000000,
            0x000000A000000000,
            0x0000004000000000,
            0x0000020000000000,
            0x0000050000000000,
            0x00000A0000000000,
            0x0000140000000000,
            0x0000280000000000,
            0x0000500000000000,
            0x0000A00000000000,
            0x0000400000000000,
            0x0002000000000000,
            0x0005000000000000,
            0x000A000000000000,
            0x0014000000000000,
            0x0028000000000000,
            0x0050000000000000,
            0x00A0000000000000,
            0x0040000000000000,
            0x0200000000000000,
            0x0500000000000000,
            0x0A00000000000000,
            0x1400000000000000,
            0x2800000000000000,
            0x5000000000000000,
            0xA000000000000000,
            0x4000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
        ],
        [
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000002,
            0x0000000000000005,
            0x000000000000000A,
            0x0000000000000014,
            0x0000000000000028,
            0x0000000000000050,
            0x00000000000000A0,
            0x0000000000000040,
            0x0000000000000200,
            0x0000000000000500,
            0x0000000000000A00,
            0x0000000000001400,
            0x0000000000002800,
            0x0000000000005000,
            0x000000000000A000,
            0x0000000000004000,
            0x0000000000020000,
            0x0000000000050000,
            0x00000000000A0000,
            0x0000000000140000,
            0x0000000000280000,
            0x0000000000500000,
            0x0000000000A00000,
            0x0000000000400000,
            0x0000000002000000,
            0x0000000005000000,
            0x000000000A000000,
            0x0000000014000000,
            0x0000000028000000,
            0x0000000050000000,
            0x00000000A0000000,
            0x0000000040000000,
            0x0000000200000000,
            0x0000000500000000,
            0x0000000A00000000,
            0x0000001400000000,
            0x0000002800000000,
            0x0000005000000000,
            0x000000A000000000,
            0x0000004000000000,
            0x0000020000000000,
            0x0000050000000000,
            0x00000A0000000000,
            0x0000140000000000,
            0x0000280000000000,
            0x0000500000000000,
            0x0000A00000000000,
            0x0000400000000000,
            0x0002000000000000,
            0x0005000000000000,
            0x000A000000000000,
            0x0014000000000000,
            0x0028000000000000,
            0x0050000000000000,
            0x00A0000000000000,
            0x0040000000000000,
        ],
    ];

    /// See https://www.chessprogramming.org/Kindergarten_Bitboards
    // pub fn diagonal_attacks(square: Square, mut occupied: Bitboard) -> Bitboard {
    //     let diag_mask_ex = BB_DIAG_MASK_EX[square as usize];
    //     let north_fill = (diag_mask_ex & occupied).overflowing_mul(BB_FILE_B);
    //     occupied = north_fill.0 >> 58;
    //     diag_mask_ex & BB_KG_FILL_UP_ATTACKS[(square & 0x7) as usize][occupied as usize]
    // }

    // pub fn anti_diagonal_attacks(square: Square, mut occupied: Bitboard) -> Bitboard {
    //     let anti_diag_mask_ex = BB_ANTI_DIAG_MASK_EX[square as usize];
    //     let north_fill = (anti_diag_mask_ex & occupied).overflowing_mul(BB_FILE_B);
    //     occupied = north_fill.0 >> 58;
    //     anti_diag_mask_ex & BB_KG_FILL_UP_ATTACKS[(square & 0x7) as usize][occupied as usize]
    // }

    #[inline]
    pub fn rank_attacks(square: Square, mut occupied: Bitboard) -> Bitboard {
        //let rank_mask_ex = BB_RANKS[(square >> 0x3) as usize] ^ BB_SQUARES[square as usize];
        //let north_fill = (rank_mask_ex & occupied).overflowing_mul(BB_FILE_B);
        let north_fill = (BB_RANK_MASK_EX[square as usize] & occupied).overflowing_mul(BB_FILE_B);
        occupied = north_fill.0 >> 58;
        // rank_mask_ex & BB_KG_FILL_UP_ATTACKS[(square & 0x7) as usize][occupied as usize]
        BB_RANK_MASK_EX[square as usize]
            & BB_KG_FILL_UP_ATTACKS[(square & 0x7) as usize][occupied as usize]
    }

    // pub fn file_attacks(square: Square, mut occupied: Bitboard) -> Bitboard {
    //     let diag_c7h2: Bitboard = 0x0004_0810_2040_8000;
    //     occupied = BB_FILE_A & (occupied >> (square & 0x7));
    //     occupied = (diag_c7h2.overflowing_mul(occupied).0) >> 58;
    //     BB_A_FILE_ATTACKS[(square >> 0x3) as usize][occupied as usize] << (square & 0x7)
    // }

    // See https://www.chessprogramming.org/Hyperbola_Quintessence
    // #[inline]
    // pub fn diagonal_attacks(square: Square, occupied: Bitboard) -> Bitboard {
    //     let mut forward = occupied & BB_DIAG_MASK_EX[square as usize];
    //     let mut reverse = forward.swap_bytes();
    //     forward = forward.wrapping_sub(BB_SQUARES[square as usize]);
    //     reverse = reverse.wrapping_sub(BB_SQUARES[square as usize].swap_bytes());
    //     forward ^= reverse.swap_bytes();
    //     forward & BB_DIAG_MASK_EX[square as usize]
    // }

    // #[inline]
    // pub fn anti_diagonal_attacks(square: Square, occupied: Bitboard) -> Bitboard {
    //     let mut forward = occupied & BB_ANTI_DIAG_MASK_EX[square as usize];
    //     let mut reverse = forward.swap_bytes();
    //     forward = forward.wrapping_sub(BB_SQUARES[square as usize]);
    //     reverse = reverse.wrapping_sub(BB_SQUARES[square as usize].swap_bytes());
    //     forward ^= reverse.swap_bytes();
    //     forward & BB_ANTI_DIAG_MASK_EX[square as usize]
    // }

    #[inline]
    pub fn file_attacks(square: Square, occupied: Bitboard) -> Bitboard {
        let mut forward = occupied & BB_FILE_MASK_EX[square as usize];
        let mut reverse = forward.swap_bytes();
        forward = forward.wrapping_sub(BB_SQUARES[square as usize]);
        reverse = reverse.wrapping_sub(BB_SQUARES[square as usize].swap_bytes());
        forward ^= reverse.swap_bytes();
        forward & BB_FILE_MASK_EX[square as usize]
    }

    // see https://www.chessprogramming.org/SSSE3#SSSE3Version
    // #[cfg(
    //     all(
    //         any(target_arch = "x86", target_arch = "x86_64"),
    //         target_feature = "avx2"
    //     )
    // )]
    #[inline]
    pub fn bishop_attacks(square: Square, occupied: Bitboard) -> Bitboard {
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;
        unsafe {
            let swap_mask = *BB_SSSE3_SWAP_MASK;
            let diag_mask_ex = BB_SSSE3_DIAG_MASK_EX[square as usize];
            let sq_mask = BB_SSSE3_SQUARE[square as usize];
            let mut occ_mask = _mm_cvtsi64x_si128(std::mem::transmute::<u64, i64>(occupied)); // general purpose 64 bit to xmm low qword
            occ_mask = _mm_unpacklo_epi64(occ_mask, occ_mask); // occ : occ
            occ_mask = _mm_and_si128(occ_mask, diag_mask_ex); // occ_mask (antidiag : diagonal)
            let mut rev_occ_mask = _mm_shuffle_epi8(occ_mask, swap_mask); // occ_mask'(antidiag : diagonal)
            occ_mask = _mm_sub_epi64(occ_mask, sq_mask); // occ_mask - bishop
            let sq_mask = _mm_shuffle_epi8(sq_mask, swap_mask); // bishop', one may also use singleBitsXMM [sq^56]
            rev_occ_mask = _mm_sub_epi64(rev_occ_mask, sq_mask); // occ_mask'- bishop'
            rev_occ_mask = _mm_shuffle_epi8(rev_occ_mask, swap_mask); // re-reverse
            occ_mask = _mm_xor_si128(occ_mask, rev_occ_mask); // attacks
            occ_mask = _mm_and_si128(occ_mask, diag_mask_ex); // antidiag : diagonal
            rev_occ_mask = _mm_unpackhi_epi64(occ_mask, occ_mask); // antidiag : antidiag
            occ_mask = _mm_add_epi64(occ_mask, rev_occ_mask); // diagonal + antidiag
            std::mem::transmute::<i64, u64>(_mm_cvtsi128_si64(occ_mask)) // convert xmm to general purpose 64 bit
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{thread_rng, Rng};
    use std::u64;
    use test::{self, Bencher};

    static TESTSIZE: usize = 5_000;

    lazy_static! {
        static ref DATA: Vec<Bitboard> = (0..)
            .take(test::black_box(TESTSIZE))
            .map(|_| thread_rng().gen_range(u64::MIN, u64::MAX) as Bitboard)
            .collect();
    }

    fn scan_count(mut bb: Bitboard) -> u64 {
        let mut count = 0_u64;
        while bb > 0 {
            let sq = bb.scan();
            count += u64::from(sq);
            bb.clear(sq);
        }
        count
    }

    fn iter_count(mut bb: Bitboard) -> u64 {
        let mut count = 0_u64;
        for sq in bb.iter() {
            count += u64::from(sq)
        }
        count
    }

    #[bench]
    fn bench_naked_bitboards(b: &mut Bencher) {
        b.iter(|| {
            let mut sum = 0_u64;
            for bb in DATA.clone().iter_mut() {
                let mut count = 0_u64;
                while *bb > 0 {
                    let sq = bb.scan();
                    count += u64::from(sq);
                    bb.clear(sq);
                }
                sum += count;
            }
            sum
        });
    }

    #[bench]
    fn bench_naked_bitboards_v2(b: &mut Bencher) {
        b.iter(|| {
            //scan_count(u64::MAX)
            (u64::MAX - (TESTSIZE as u64)..u64::MAX).fold(0, |acc, el| acc + scan_count(el))
        });
    }

    #[bench]
    fn bench_wrapped_bitboards(b: &mut Bencher) {
        b.iter(|| {
            let mut sum = 0_u64;
            for bb in DATA.clone().iter_mut() {
                let mut count = 0_u64;
                for sq in bb.iter() {
                    count += u64::from(sq);
                }
                sum += count;
            }
            sum
        });
    }

    #[bench]
    fn bench_wrapped_bitboards_v2(b: &mut Bencher) {
        b.iter(|| {
            //iter_count(u64::MAX)
            (u64::MAX - (TESTSIZE as u64)..u64::MAX).fold(0, |acc, el| acc + iter_count(el))
        });
    }

}
