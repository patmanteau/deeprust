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
    ($T:ty, $ARRIDENT:ident) => {
        impl BitTwiddling<$T> for $T {
            #[inline]
            fn bit_at(pos: u32) -> $T {
                1 << pos
                // $ARRIDENT[pos as usize]
            }

            #[inline]
            fn test_bit(&self, pos: u32) -> bool {
                self & Self::bit_at(pos) > 0
                // unsafe { return 0 < _bittest64(&(*self as i64), pos as i64) }
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
                unsafe { _bextr_u64(*self as u64, pos, len) as Self }
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

lazy_static! {
    pub static ref BT_U8: [u8; std::mem::size_of::<u8>() * 8] = {
        const BT_ARRSIZE: usize = std::mem::size_of::<u8>() * 8;
        let mut arr: [u8; BT_ARRSIZE] = [0; BT_ARRSIZE];
        for (index, entry) in arr.iter_mut().enumerate().take(BT_ARRSIZE) {
            *entry = 1 << index;
        }
        arr
    };
    pub static ref BT_U16: [u16; std::mem::size_of::<u16>() * 8] = {
        const BT_ARRSIZE: usize = std::mem::size_of::<u16>() * 8;
        let mut arr: [u16; BT_ARRSIZE] = [0; BT_ARRSIZE];
        for (index, entry) in arr.iter_mut().enumerate().take(BT_ARRSIZE) {
            *entry = 1 << index;
        }
        arr
    };
    pub static ref BT_U32: [u32; std::mem::size_of::<u32>() * 8] = {
        const BT_ARRSIZE: usize = std::mem::size_of::<u32>() * 8;
        let mut arr: [u32; BT_ARRSIZE] = [0; BT_ARRSIZE];
        for (index, entry) in arr.iter_mut().enumerate().take(BT_ARRSIZE) {
            *entry = 1 << index;
        }
        arr
    };
    pub static ref BT_U64: [u64; std::mem::size_of::<u64>() * 8] = {
        const BT_ARRSIZE: usize = std::mem::size_of::<u64>() * 8;
        let mut arr: [u64; BT_ARRSIZE] = [0; BT_ARRSIZE];
        for (index, entry) in arr.iter_mut().enumerate().take(BT_ARRSIZE) {
            *entry = 1 << index;
        }
        arr
    };
}
twiddle_impl!(u8, BT_U8);
twiddle_impl!(u16, BT_U16);
twiddle_impl!(u32, BT_U32);
twiddle_impl!(u64, BT_U64);

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
