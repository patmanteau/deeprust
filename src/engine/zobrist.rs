use rand::{RngCore, SeedableRng};
use rand_pcg::Pcg64;

pub type ZobKey = u64;

#[derive(Clone)]
pub struct Zobrist {
    pub pieces: [[ZobKey; 64]; 12],
    pub black_to_move: ZobKey,
    pub castling: [[ZobKey; 2]; 2],
    pub ep_files: [ZobKey; 8],
}

impl Default for Zobrist {
    fn default() -> Self {
        Self::new()
    }
}

impl Zobrist {
    pub fn new() -> Self {
        let mut z = Self {
            pieces: [[0; 64]; 12],
            black_to_move: 0,
            castling: [[0; 2]; 2],
            ep_files: [0; 8],
        };

        let mut prng = Pcg64::seed_from_u64(0);


        for table in z.pieces.iter_mut() {
            for square in table.iter_mut() {
                *square = prng.next_u64();
            }
        }

        z.black_to_move = prng.next_u64();

        for color in z.castling.iter_mut() {
            for side in color.iter_mut() {
                *side = prng.next_u64();
            }
        }

        for file in z.ep_files.iter_mut() {
            *file = prng.next_u64();
        }

        z
    }
}

lazy_static! {
    pub static ref ZobTables: Zobrist = {
        Zobrist::new()
    };
}
