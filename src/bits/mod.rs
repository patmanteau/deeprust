use bitwise::*;

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