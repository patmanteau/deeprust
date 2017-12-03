use engine::board::Board;
use engine::moves::Move;

pub struct MoveGenerator {
    moves: [Move; 512],
    cur: usize,
}

impl Iterator for MoveGenerator {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Move::new(0, 0, 0, 0, 0))
    }
}

impl MoveGenerator {
    pub fn new() -> MoveGenerator {
        MoveGenerator{
            moves: [Move::new(0, 0, 0, 0, 0); 512],
            cur: 0,
        }
    }
    
    fn from_board(&mut self, board: &Board) {

    }

    fn gen_pawn_moves(&mut self) {

    }
}