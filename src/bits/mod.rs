use bitwise::*;
use std::ops::{Not, BitAnd, BitOr, BitXor, Shl, Shr};

#[inline]
pub fn test_bit<T: TestBit>(lhs: T, at: usize) -> bool {
   lhs.test_bit(at)
}

#[inline]
pub fn flip_bit<T: FlipBit>(lhs: T, at: usize) -> T {
   lhs.flip_bit(at)
}

#[inline]
pub fn clear_bit<T: ClearBit>(lhs: T, at: usize) -> T {
   lhs.clear_bit(at)
}

#[inline]
pub fn extract_bits<T: ExtractBits>(lhs: T, at: usize, length: usize) -> T {
    lhs.extract_bits(at, length)
}

#[inline]
pub fn swap_bytes<T: SwapBytes>(lhs: T) -> T {
    lhs.swap_bytes()
}

#[inline]
pub fn count_trailing_zeros<T: CountTrailingZeros>(lhs: T) -> T {
    lhs.count_trailing_zeros()
}

#[inline]
pub fn count_leading_zeros<T: CountLeadingZeros>(lhs: T) -> T {
    lhs.count_leading_zeros()
}


#[inline]
pub fn clear_least_significant_one<T: ClearLeastSignificantOne>(lhs: T) -> T {
    lhs.clear_least_significant_one()
}

// #[inline]
// pub fn set_bits_geq<T: SetBitsGeq>(lhs: T, n: usize) -> T {
//     lhs.set_bits_geq(n)
// }

// #[inline]
// pub fn set_bits_leq<T: SetBitsLeq + BitOr>(lhs: T, n: usize) -> T {
//     // lhs.set_bits_leq(n)
//     lhs | (1 << n) - 1
// }


// #[inline]
// pub fn set_bits_g<T: SetBitsGeq + ClearBit>(lhs: T, n: usize) -> T {
//     lhs.set_bits_geq(n).clear_bit(n)
// }

// #[inline]
// pub fn set_bits_l<T: SetBitsLeq +  ClearBit>(lhs: T, n: usize) -> T {
//     lhs.set_bits_leq(n).clear_bit(n)
// }
