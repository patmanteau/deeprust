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
    //    13     1    1    0    0      capture to promotion to bishop
    //    14     1    1    0    0      capture to promotion to rook
    //    15     1    1    0    0      capture to promotion to queen
    //  /
    // /
    // 10..15:  Origin square
    // |
    // 16:      Color
    // 17..19:  Moving Piece
    // 20..22:  Captured piece
    // 23:      King castle allowed before
    // 24:      Queen castle allowed before
    
    m: u32
}

impl Move {
    #[inline]
    pub fn new(from: u32, to: u32, color: u32, piece: u32, flags: u32, extended: u32) -> Move {
        Move {
            m: ((from & 0x3f) << 10) | 
               (to & 0x3f) | 
               ((flags & 0xf) << 6) | 
               ((extended & 0x1ff) << 16) | 
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
    pub fn make_extended(captured_piece: u32, kcastle_before: bool, qcastle_before: bool) -> u32 {
        ((qcastle_before as u32) << 4) | ((kcastle_before as u32) << 3) | ((captured_piece) & 0x7)
    }

    #[inline]
    pub fn set_from(&mut self, from: u32) {
        self.m &= !0xfc00;
        self.m |= (from & 0x3f) << 10;
    }

    #[inline]
    pub fn set_to(&mut self, to: u32) {
        self.m &= !0x3f;
        self.m |= to & 0x3f;
    }

    #[inline]
    pub fn from(&self) -> u32 {
        (self.m >> 10) & 0x3f
    }

    #[inline]
    pub fn to(&self) -> u32 {
        (self.m & 0x3f)
    }

    #[inline]
    pub fn color(&self) -> u32 {
        ((self.m >> 16) & 0x1)
    }

    #[inline]
    pub fn piece(&self) -> u32 {
        ((self.m >> 17) & 0x7)
    }

    #[inline]
    pub fn is_promotion(&self) -> bool {
        0 != (self.m >> 9) & 0x1
    }

    #[inline]
    pub fn is_capture(&self) -> bool {
        0 != (self.m >> 8) & 0x1
    }

    #[inline]
    pub fn has_special_1(&self) -> bool {
        0 != (self.m >> 7) & 0x1
    }
    
    #[inline]
    pub fn has_special_0(&self) -> bool {
        0 != (self.m >> 6) & 0x1
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
                            assert_eq!(from, mov.from());
                            assert_eq!(to, mov.to());
                            assert_eq!(color, mov.color());
                            assert_eq!(piece, mov.piece());

                            // flags
                            assert_eq!(prom, mov.is_promotion());
                            assert_eq!(cap, mov.is_capture());
                            assert_eq!(spc0, mov.has_special_0());
                            assert_eq!(spc1, mov.has_special_1());

                            mov.set_from(63-from);
                            mov.set_to(63-to);
                            // standard fields
                            assert_eq!(63-from, mov.from());
                            assert_eq!(63-to, mov.to());
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