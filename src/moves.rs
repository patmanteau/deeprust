use common::*;
use std::fmt;
use san::SAN;
use square::{Square, SquarePrimitives};

/// Stores information required for unmaking moves - captured piece,
/// castling rights, en passant square and half move clock.
#[derive(Copy, Clone)]
pub struct UnmakeInfo {
    // bit mask:
    //
    // 0..2:    Captured piece
    // 3:       Captured piece color
    // 4..7:    Castling rights before (KQkq)
    // 8..13:   En passant square
    // 14:      En passant available
    // 15..31:  Half move clock
    m: u32
}

impl fmt::Debug for UnmakeInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UnmakeInfo {{ captured_piece: {}, captured_color: {}, castling: {}{}, ep_square: {}, ep_available: {}, halfmoves: {} }}",
            self.captured_piece(), self.captured_color(), self.castling()[0], self.castling()[1], self.ep_square(), self.ep_available(), self.halfmoves())
    }
}

impl UnmakeInfo {

    /// Constructs a new UnmakeInfo
    #[inline]
    pub fn new(cap_piece: u32, cap_color: u32, castling: [u32; 2],
               ep_square: Square, ep_available: bool, halfmoves: u32) -> UnmakeInfo {
        UnmakeInfo {
            m: ((halfmoves & 0x1ffff) << 15) |
               ((ep_available as u32 & 0x1) << 14) |
               ((ep_square & 0x3f) << 8) |
               ((castling[1] & 0x3) << 6) |
               ((castling[0] & 0x3) << 4) |
               ((cap_color & 0x1) << 3) |
               ((cap_piece) & 0x7)
        }
    }

    #[inline]
    pub fn captured_piece(&self) -> u32 {
        self.m.extract_bits(0, 3)
    }

    #[inline]
    pub fn captured_color(&self) -> u32 {
        self.m.extract_bits(3, 1)
    }

    #[inline]
    pub fn castling(&self) -> [u32; 2] {
        [self.m.extract_bits(4, 2), self.m.extract_bits(6, 2)]
    }

    #[inline]
    pub fn ep_square(&self) -> Square {
        self.m.extract_bits(8, 6) as Square
    }

    #[inline]
    pub fn ep_available(&self) -> bool {
        self.m.test_bit(14)
    }

    #[inline]
    pub fn halfmoves(&self) -> u32 {
        self.m.extract_bits(15, 16)
    }
}

#[derive(Copy, Clone)]
pub struct Move {
    // bit mask:
    // (from https://chessprogramming.wikispaces.com/Encoding+Moves)
    // 
    // 0..5:    Destination square
    // |
    // 6:       Special 0  _
    // 7:       Special 1  |__ Flags
    // 8:       Capture?   |
    // 9:       Promotion? -
    // \
    //  \
    //    Flags:
    //         Prom Capt Spc1 Spc0     Kind of move
    //     0     0    0    0    0      quiet
    //     1     0    0    0    1      double pawn push
    //     2     0    0    1    0      king castle
    //     3     0    0    1    1      queen castle
    //     4     0    1    0    0      capture
    //     5     0    1    0    1      capture en passant
    //     6     0    1    1    0      reserved
    //     7     0    1    1    1      reserved
    //     8     1    0    0    0      promotion to knight
    //     9     1    0    0    1      promotion to bishop
    //    10     1    0    1    0      promotion to rook
    //    11     1    0    1    1      promotion to queen
    //    12     1    1    0    0      capture to promotion to knight
    //    13     1    1    0    1      capture to promotion to bishop
    //    14     1    1    1    0      capture to promotion to rook
    //    15     1    1    1    1      capture to promotion to queen
    //  /
    // /
    // 10..15:  Origin square
    
    m: u32
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Move {{ orig: {}, dest: {}, capture: {}, promotion: {}, spc0: {}, spc1: {} }}",
            SAN::from_square(self.orig()).s,
            SAN::from_square(self.dest()).s,
            self.is_capture(),
            self.is_promotion(),
            self.has_special_0(),
            self.has_special_1())
    }
}

pub const MOV_QUIET: u32 =       0b0000;
pub const MOV_DPP: u32 =         0b0001;

pub const MOV_K_CASTLE: u32 =    0b0010;
pub const MOV_Q_CASTLE: u32 =    0b0011;

pub const MOV_CAPTURE: u32 =     0b0100;
pub const MOV_CAPTURE_EP: u32 =  0b0101;

pub const MOV_PROM_QUEEN: u32 =  0b1011;
pub const MOV_PROM_ROOK: u32 =   0b1010;
pub const MOV_PROM_BISHOP: u32 = 0b1001;
pub const MOV_PROM_KNIGHT: u32 = 0b1000;

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}{}",
            SAN::from_square(self.orig()).s,
            match self.is_capture() {
                true => "x",
                false => "-",
            },
            SAN::from_square(self.dest()).s)
    }
}

// TODO: Make Move a trait
impl Move {
    /// Constructs a new Move
    #[inline]
    // pub fn new(orig: u32, dest: u32, color: u32, piece: u32, flags: u32, extended: u32) -> Move {
    pub fn new(orig: Square, dest: Square, flags: u32) -> Move {
        Move {
            m: ((orig & 0x3f) << 10) | 
               (dest & 0x3f) | 
               ((flags & 0xf) << 6),
        }
    }

    #[inline]
    pub fn make_flags(is_capture: bool, is_promotion: bool,
                      is_special_0: bool, is_special_1: bool) -> u32 {
        ((is_promotion as u32) << 3) | ((is_capture as u32) << 2) | ((is_special_1 as u32) << 1) | (is_special_0 as u32)
    }

    // #[inline]
    // pub fn make_extended(captured_piece: u32, castling_before: u32) -> u32 {
    //     (castling_before << 3) | ((captured_piece) & 0x7)
    // }

    #[inline]
    pub fn set_orig(&mut self, from: Square) {
        self.m &= !0xfc00;
        self.m |= (from & 0x3f) << 10;
    }

    #[inline]
    pub fn set_dest(&mut self, to: Square) {
        self.m &= !0x3f;
        self.m |= to & 0x3f;
    }

    #[inline]
    pub fn toggle_special_0(&mut self) {
        self.m.flip_bit(6);
    }

    #[inline]
    pub fn toggle_special_1(&mut self) {
        self.m.flip_bit(7);
    }

    #[inline]
    pub fn toggle_capture(&mut self) {
        self.m.flip_bit(8);
    }

    #[inline]
    pub fn toggle_promotion(&mut self) {
        self.m.flip_bit(9);
    }

    #[inline]
    pub fn orig(&self) -> Square {
        // (self.m >> 10) & 0x3f
        self.m.extract_bits(10, 6) as Square
    }

    #[inline]
    pub fn dest(&self) -> Square {
        (self.m & 0x3f) as Square
    }

    // #[inline]
    // pub fn color(&self) -> u32 {
    //     // ((self.m >> 16) & 0x1)
    //     bits::extract_bits(self.m, 16, 1)
    // }

    // #[inline]
    // pub fn piece(&self) -> u32 {
    //     // ((self.m >> 17) & 0x7)
    //     bits::extract_bits(self.m, 17, 3)
    // }

    #[inline]
    pub fn special(&self) -> u32 {
        self.m.extract_bits(6, 2)
    }

    #[inline]
    pub fn has_special_0(&self) -> bool {
        self.m.test_bit(6)
    }

    #[inline]
    pub fn has_special_1(&self) -> bool {
        self.m.test_bit(7)
    }
    
    #[inline]
    pub fn is_quiet(&self) -> bool {
        MOV_QUIET == self.m.extract_bits(6, 4)
    }

    #[inline]
    pub fn is_capture(&self) -> bool {
        self.m.test_bit(8)
    }

    #[inline]
    pub fn is_capture_en_passant(&self) -> bool {
        MOV_CAPTURE_EP == self.m.extract_bits(6, 4)
    }

    #[inline]
    pub fn is_double_pawn_push(&self) -> bool {
        MOV_DPP == self.m.extract_bits(6, 4)
    }

    #[inline]
    pub fn is_promotion(&self) -> bool {
        self.m.test_bit(9)
    }

    #[inline]
    pub fn is_king_castle(&self) -> bool {
        MOV_K_CASTLE == self.m.extract_bits(6, 4)
    }

    #[inline]
    pub fn is_queen_castle(&self) -> bool {
        MOV_Q_CASTLE == self.m.extract_bits(6, 4)
    }

    // #[inline]
    // pub fn captured_piece(&self) -> u32 {
    //     // ((self.m >> 20) & 0x7)
    //     self.m.extract_bits(20, 3)
    // }

    // #[inline]
    // pub fn castling_before(&self) -> u32 {
    //     self.m.extract_bits(23, 4)
    // }

    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_encodes_unmake_information() {
        // 0..2:    Captured piece
        // 3:       Captured piece color
        // 4..7:    Castling rights before (KQkq)
        // 8..13:   En passant square
        // 14:      En passant available
        // 15..31:  Half move clock
        for cap_color in 0..2 {
            for cap_piece in 2..8 {
                for wcastling in 0..4 {
                    for bcastling in 0..4 {
                        for ep_square in 0..64 {
                            for halfmoves in 0..256 {
                                {
                                    let store = UnmakeInfo::new(cap_piece, cap_color, [wcastling, bcastling], ep_square, false, halfmoves);
                                    assert_eq!(cap_piece, store.captured_piece());
                                    assert_eq!(cap_color, store.captured_color());
                                    assert_eq!(wcastling, store.castling()[0]);
                                    assert_eq!(bcastling, store.castling()[1]);
                                    assert_eq!(ep_square, store.ep_square());
                                    assert_eq!(false, store.ep_available());
                                    assert_eq!(halfmoves, store.halfmoves());
                                }
                                {
                                    let store = UnmakeInfo::new(cap_piece, cap_color, [wcastling, bcastling], ep_square, false, halfmoves);
                                    assert_eq!(cap_piece, store.captured_piece());
                                    assert_eq!(cap_color, store.captured_color());
                                    assert_eq!(wcastling, store.castling()[0]);
                                    assert_eq!(bcastling, store.castling()[1]);
                                    assert_eq!(ep_square, store.ep_square());
                                    assert_eq!(false, store.ep_available());
                                    assert_eq!(halfmoves, store.halfmoves());
                                }
                            }
                        }
                    }
                }
            }
        }
        
    }

    #[test]
    fn it_encodes_moves() {
        for color in 0..2 { // WHITE..BLACK
            for piece in 2..8 { // PAWN..KING
                for from in 0..64 {
                    for to in 0..64 {
                        for i in 0..15 {
                            let prom = 0 != i & 0x8;
                            let cap = 0 != i & 0x4;
                            let spc1 = 0 != i & 0x2;
                            let spc0 = 0 != i & 0x1;
                            let flags = Move::make_flags(cap, prom, spc0, spc1);
                            assert_eq!(i, flags);
                            
                            let mut mov = Move::new(from, to, flags);
                            // standard fields
                            assert_eq!(from, mov.orig());
                            assert_eq!(to, mov.dest());
                            // assert_eq!(color, mov.color());
                            // assert_eq!(piece, mov.piece());

                            // flags
                            assert_eq!(prom, mov.is_promotion());
                            assert_eq!(cap, mov.is_capture());
                            assert_eq!(spc0, mov.has_special_0());
                            assert_eq!(spc1, mov.has_special_1());

                            mov.set_orig(63-from);
                            mov.set_dest(63-to);
                            // standard fields
                            assert_eq!(63-from, mov.orig());
                            assert_eq!(63-to, mov.dest());
                            // assert_eq!(color, mov.color());
                            // assert_eq!(piece, mov.piece());
                            
                            // flags
                            assert_eq!(prom, mov.is_promotion());
                            assert_eq!(cap, mov.is_capture());
                            assert_eq!(spc0, mov.has_special_0());
                            assert_eq!(spc1, mov.has_special_1());

                            
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn it_makes_flags() {
        for i in 0..15 {
            let prom = 0 != i & 0x8;
            let cap = 0 != i & 0x4;
            let spc1 = 0 != i & 0x2;
            let spc0 = 0 != i & 0x1;
            let flags = Move::make_flags(cap, prom, spc0, spc1);
            assert_eq!(i, flags);
        }
    }
}