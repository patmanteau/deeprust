use board::common::*;

pub type Bitboard = u64;

pub trait BitboardPrimitives {
    fn count(self) -> u32;
    fn scan(self) -> u32;
    
    fn to_debug_string(&self) -> String;
}

impl BitboardPrimitives for Bitboard {
    #[inline]
    fn count(self) -> u32 {
        self.count_ones()
    }

    #[inline]
    fn scan(self) -> u32 {
        self.trailing_zeros()
    }

    fn to_debug_string(&self) -> String {
        let mut out = String::new();

        out.push_str(&format!("DEBUG(bitboard): 0x{:016X}\n", *self));

        out.push_str("+--------+\n");
        for i in 0..8 {
            out.push_str("|");
            for j in 0..8 {
                let s = 8 * (7-i) + j;
                out.push_str(&format!("{:b}", self.test_bit(s) as usize));
            }
            out.push_str(&format!("|{}\n", (7-i) + 1));
        }
        out.push_str("+--------+\n abcdefgh\n");
        out
    }
}
