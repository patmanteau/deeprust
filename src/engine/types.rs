use ::bits;
// use ::bitwise;
// use bitwise::{flip_bit, test_bit};

pub const WHITE: u32 = 0;
pub const BLACK: u32 = 1;
pub const PAWN: u32 = 2;
pub const KNIGHT: u32 = 3;
pub const BISHOP: u32 = 4;
pub const ROOK: u32 = 5;
pub const QUEEN: u32 = 6;
pub const KING: u32 = 7;

#[derive(Debug, Copy, Clone)]
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
    // |
    // 16:      Color
    // 17..19:  Moving Piece
    // 20..22:  Captured piece
    // 23..26:  Castling rights before (KQkq)
    
    m: u32
}

impl Move {
    /// Constructs a new Move
    #[inline]
    pub fn new(orig: u32, dest: u32, color: u32, piece: u32, flags: u32, extended: u32) -> Move {
        Move {
            m: ((orig & 0x3f) << 10) | 
               (dest & 0x3f) | 
               ((flags & 0xf) << 6) | 
               ((extended & 0x7f) << 20) | 
               (((color) & 0x1) << 16) | 
               (((piece) & 0x7) << 17),
        }
    }

    #[inline]
    pub fn make_flags(is_capture: bool, is_promotion: bool,
                      is_special_0: bool, is_special_1: bool) -> u32 {
        ((is_promotion as u32) << 3) | ((is_capture as u32) << 2) | ((is_special_1 as u32) << 1) | (is_special_0 as u32)
    }

    #[inline]
    pub fn make_extended(captured_piece: u32, castling_before: u32) -> u32 {
        (castling_before << 3) | ((captured_piece) & 0x7)
    }

    #[inline]
    pub fn set_orig(&mut self, from: u32) {
        self.m &= !0xfc00;
        self.m |= (from & 0x3f) << 10;
    }

    #[inline]
    pub fn set_dest(&mut self, to: u32) {
        self.m &= !0x3f;
        self.m |= to & 0x3f;
    }

    #[inline]
    pub fn toggle_special_0(&mut self) {
        self.m = bits::flip_bit(self.m, 6);
    }

    #[inline]
    pub fn toggle_special_1(&mut self) {
        self.m = bits::flip_bit(self.m, 7);
    }

    #[inline]
    pub fn toggle_capture(&mut self) {
        self.m = bits::flip_bit(self.m, 8);
    }

    #[inline]
    pub fn toggle_promotion(&mut self) {
        self.m = bits::flip_bit(self.m, 9);
    }

    #[inline]
    pub fn orig(&self) -> u32 {
        // (self.m >> 10) & 0x3f
        bits::extract_bits(self.m, 10, 6)
    }

    #[inline]
    pub fn dest(&self) -> u32 {
        (self.m & 0x3f)
    }

    #[inline]
    pub fn color(&self) -> u32 {
        // ((self.m >> 16) & 0x1)
        bits::extract_bits(self.m, 16, 1)
    }

    #[inline]
    pub fn piece(&self) -> u32 {
        // ((self.m >> 17) & 0x7)
        bits::extract_bits(self.m, 17, 3)
    }

    #[inline]
    pub fn special(&self) -> u32 {
        bits::extract_bits(self.m, 6, 2)
    }

    #[inline]
    pub fn has_special_0(&self) -> bool {
        bits::test_bit(self.m, 6)
    }

    #[inline]
    pub fn has_special_1(&self) -> bool {
        bits::test_bit(self.m, 7)
    }
    
    #[inline]
    pub fn is_quiet(&self) -> bool {
        0 == bits::extract_bits(self.m, 6, 4)
    }

    #[inline]
    pub fn is_capture(&self) -> bool {
        bits::test_bit(self.m, 8)
    }

    #[inline]
    pub fn is_capture_en_passant(&self) -> bool {
        5 == bits::extract_bits(self.m, 6, 4)
    }

    #[inline]
    pub fn is_double_pawn_push(&self) -> bool {
        1 == bits::extract_bits(self.m, 6, 4)
    }

    #[inline]
    pub fn is_promotion(&self) -> bool {
        bits::test_bit(self.m, 9)
    }

    #[inline]
    pub fn is_king_castle(&self) -> bool {
        2 == bits::extract_bits(self.m, 6, 4)
    }

    #[inline]
    pub fn is_queen_castle(&self) -> bool {
        3 == bits::extract_bits(self.m, 6, 4)
    }

    #[inline]
    pub fn captured_piece(&self) -> u32 {
        // ((self.m >> 20) & 0x7)
        bits::extract_bits(self.m, 20, 3)
    }

    #[inline]
    pub fn castling_before(&self) -> u32 {
        bits::extract_bits(self.m, 23, 4)
    }

    
}

#[cfg(test)]
mod tests {
    use super::*;

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
                            
                            let mut mov = Move::new(from, to, color, piece, flags, 0);
                            // standard fields
                            assert_eq!(from, mov.orig());
                            assert_eq!(to, mov.dest());
                            assert_eq!(color, mov.color());
                            assert_eq!(piece, mov.piece());

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
                            assert_eq!(color, mov.color());
                            assert_eq!(piece, mov.piece());
                            
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