extern crate deeprust;

use deeprust::bitboard as bb;

fn print_array_of_bbs(vals: &[bb::Bitboard]) {
    print!("[");
    for val in vals {
        print!("0x{:016X}, ", val);
    }
    print!("]");
}

fn main() {
    print_array_of_bbs(&bb::BB_PAWN_ATTACKS[0]);
    print_array_of_bbs(&bb::BB_PAWN_ATTACKS[1]);
}