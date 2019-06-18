use std::arch::x86_64::*;

pub trait BitTwiddling<T> {
    fn bit_at(pos: u32) -> Self;
    fn test_bit(&self, at: u32) -> bool;
    fn set_bit(&mut self, at: u32);
    fn flip_bit(&mut self, at: u32);
    fn clear_bit(&mut self, at: u32);
    fn extract_bits(&self, at: u32, length: u32) -> Self;

    fn clear_lsb(&mut self);
}
macro_rules! twiddle_impl {
    ($T:ty) => {
        impl BitTwiddling<$T> for $T {
            fn bit_at(pos: u32) -> $T {
                1 << pos
            }

            fn test_bit(&self, pos: u32) -> bool {
                //self & Self::bit_at(pos) > 0
                unsafe { return 0 < _bittest64(&(*self as i64), pos as i64) }
            }

            fn set_bit(&mut self, pos: u32) {
                *self |= Self::bit_at(pos);
            }

            fn flip_bit(&mut self, pos: u32) {
                *self ^= Self::bit_at(pos);
            }

            fn clear_bit(&mut self, pos: u32) {
                *self &= !Self::bit_at(pos);
            }

            fn extract_bits(&self, pos: u32, len: u32) -> Self {
                unsafe { _bextr_u64(*self as u64, pos, len) as Self }
            }

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

#[cfg(test)]
mod tests {

    use test::Bencher;

    #[bench]
    fn empty(b: &mut Bencher) {
        b.iter(|| 1)
    }

    #[bench]
    fn bench_u64_test_bit(_b: &mut Bencher) {

        // let _8 = 0xffu8;
        // let mut res = false;
        // b.iter(|| {
        //     let n = test::black_box(1000000);
        //     (0..n).map(|i| {
        //         let m = test::black_box(1000000000);
        //         for j in 0..m {
        //             res = _8.test_bit(8);
        //         }
        //     })
        // });
    }
}
