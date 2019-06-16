use std::arch::x86_64::*;
use square::{Square, SquarePrimitives};

pub trait BitTwiddling {
    fn bit_at(pos: Square) -> Self;
    fn test_bit(&self, at: Square) -> bool;
    fn set_bit(&mut self, at: Square) -> &mut Self;
    fn flip_bit(&mut self, at: Square) -> &mut Self;
    fn clear_bit(&mut self, at: Square) -> &mut Self;
    fn extract_bits(&self, at: Square, length: Square) -> Self;

    fn clear_lsb(&mut self) -> &mut Self;
}
macro_rules!  twiddle_impl {
    ($T:ty) => {
        impl BitTwiddling for $T {
            #[inline]
            fn bit_at(pos: Square) -> $T {
                1 << pos
            }

            #[inline]
            fn test_bit(&self, pos: Square) -> bool {
                self & Self::bit_at(pos) > 0
            }

            #[inline]
            fn set_bit(&mut self, pos: Square) -> &mut Self {
                *self |= Self::bit_at(pos);
                self
            }

            #[inline]
            fn flip_bit(&mut self, pos: Square) -> &mut Self {
                *self ^= Self::bit_at(pos);
                self
            }

            #[inline]
            fn clear_bit(&mut self, pos: Square) -> &mut Self {
                *self &= !Self::bit_at(pos);
                self
            }

            #[inline]
            fn extract_bits(&self, pos: Square, len: Square) -> Self {
                unsafe {
                    _bextr_u64(*self as u64, pos as u32, len as u32) as Self
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

twiddle_impl!(u16);
twiddle_impl!(u32);
twiddle_impl!(u64);