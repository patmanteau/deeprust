use std::arch::x86_64::*;

pub type Sq = u32;
pub type Word = u32;

pub trait BitTwiddling {
    fn bit_at(pos: u32) -> Self;
    fn test_bit(&self, at: u32) -> bool;
    fn set_bit(&mut self, at: u32) -> &mut Self;
    fn flip_bit(&mut self, at: u32) -> &mut Self;
    fn clear_bit(&mut self, at: u32) -> &mut Self;
    fn extract_bits(&self, at: u32, length: u32) -> Self;

    fn clear_lsb(&mut self) -> &mut Self;
}
macro_rules!  twiddle_impl {
    ($T:ty) => {
        impl BitTwiddling for $T {
            #[inline]
            fn bit_at(pos: u32) -> $T {
                1 << pos
            }

            #[inline]
            fn test_bit(&self, pos: u32) -> bool {
                self & Self::bit_at(pos) > 0
            }

            #[inline]
            fn set_bit(&mut self, pos: u32) -> &mut Self {
                *self |= Self::bit_at(pos);
                self
            }

            #[inline]
            fn flip_bit(&mut self, pos: u32) -> &mut Self {
                *self ^= Self::bit_at(pos);
                self
            }

            #[inline]
            fn clear_bit(&mut self, pos: u32) -> &mut Self {
                *self &= !Self::bit_at(pos);
                self
            }

            #[inline]
            fn extract_bits(&self, pos: u32, len: u32) -> Self {
                unsafe {
                    _bextr_u64(*self as u64, pos, len) as Self
                }
            }

            #[inline]
            fn clear_lsb(&mut self) -> &mut Self {
                unsafe {
                    *self = _blsr_u64(*self as u64) as Self;
                }
                self
            }
        }
    };
}

twiddle_impl!(u32);
twiddle_impl!(u64);