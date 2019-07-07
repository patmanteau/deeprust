use crate::common::BitTwiddling;
use crate::primitives::color::Color;

pub type CastlingSide = u32;

pub mod sides {
    use super::*;
    pub const KING_SIDE: CastlingSide = 0;
    pub const QUEEN_SIDE: CastlingSide = 1;
}

#[derive(Clone, Copy, Debug, Eq)]
pub struct Castling(pub u32);

impl PartialEq for Castling {
    fn eq(&self, other: &Castling) -> bool {
        self.0 == other.0
    }
}

impl Castling {
    pub fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub fn get(self, color: Color, side: CastlingSide) -> bool {
        // self.0.extract_bits((u32::from(color) << 1) + side, 1) > 0
        self.0.test_bit((u32::from(color) << 1) + side)
    }

    #[inline]
    pub fn set(&mut self, color: Color, side: CastlingSide) {
        self.0.set_bit((u32::from(color) << 1) + side);
        // self.0 |= 1 << ((u32::from(color) << 1) + side);
    }

    #[inline]
    pub fn clear(&mut self, color: Color, side: CastlingSide) {
        self.0.clear_bit((u32::from(color) << 1) + side);
    }

    #[inline]
    pub fn clear_color(&mut self, color: Color) {
        self.0.clear_bit(u32::from(color) << 1);
        self.0.clear_bit((u32::from(color) << 1) + 1);
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}
