use std::arch::x86_64::*;
use square::{Square, SquarePrimitives};

pub trait BitTwiddling<T> {
    fn bit_at(pos: u32) -> Self;
    fn test_bit(&self, at: u32) -> bool;
    fn set_bit(&mut self, at: u32);
    fn flip_bit(&mut self, at: u32);
    fn clear_bit(&mut self, at: u32);
    fn extract_bits(&self, at: u32, length: u32) -> Self;

    fn clear_lsb(&mut self);
}
macro_rules!  twiddle_impl {
    ($T:ty) => {
        impl BitTwiddling<$T> for $T {
            #[inline]
            fn bit_at(pos: u32) -> $T {
                1 << pos
            }

            #[inline]
            fn test_bit(&self, pos: u32) -> bool {
                self & Self::bit_at(pos) > 0
            }

            #[inline]
            fn set_bit(&mut self, pos: u32) {
                *self |= Self::bit_at(pos);
            }

            #[inline]
            fn flip_bit(&mut self, pos: u32) {
                *self ^= Self::bit_at(pos);
            }

            #[inline]
            fn clear_bit(&mut self, pos: u32) {
                *self &= !Self::bit_at(pos);
            }

            #[inline]
            fn extract_bits(&self, pos: u32, len: u32) -> Self {
                unsafe {
                    _bextr_u64(*self as u64, pos, len) as Self
                }
            }

            #[inline]
            fn clear_lsb(&mut self) {
                unsafe {
                    *self = _blsr_u64(*self as u64) as Self;
                }
            }
        }
    };
}

twiddle_impl!(u8);
twiddle_impl!(u16);
twiddle_impl!(u32);
twiddle_impl!(u64);