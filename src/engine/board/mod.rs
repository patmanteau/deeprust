pub mod utils;

use super::types::*;
use super::types;
use super::san::SAN;

// use std::str::FromStr;

struct Board {
    bb: [u64; 8],
    to_move: u32,
    castling: u8,
    en_passant: Option<u32>,
    halfmoves: u32,
    fullmoves: u32,
}

fn c2s(x: u32, y: u32) -> u32 {
    (y * 8) + x
}

impl Board {
    fn new() -> Board {
        Board { 
            bb: [0; 8],
            to_move: types::WHITE,
            castling: 0,
            en_passant: None,
            halfmoves: 0,
            fullmoves: 1,
        }
    }

    fn startpos() -> Board {
        let mut board = Board::new();
        // pawns
        for x in 0..8 {
            board.set_piece(types::PAWN, types::WHITE, c2s(x, 1));
            board.set_piece(types::PAWN, types::BLACK, c2s(x, 6));
        }

        // knights
        board.set_piece(types::KNIGHT, types::WHITE, c2s(1, 0));
        board.set_piece(types::KNIGHT, types::WHITE, c2s(6, 0));
        board.set_piece(types::KNIGHT, types::BLACK, c2s(1, 7));
        board.set_piece(types::KNIGHT, types::BLACK, c2s(6, 7));

        // bishops
        board.set_piece(types::BISHOP, types::WHITE, c2s(2, 0));
        board.set_piece(types::BISHOP, types::WHITE, c2s(5, 0));
        board.set_piece(types::BISHOP, types::BLACK, c2s(2, 7));
        board.set_piece(types::BISHOP, types::BLACK, c2s(5, 7));

        // rooks
        board.set_piece(types::ROOK, types::WHITE, c2s(0, 0));
        board.set_piece(types::ROOK, types::WHITE, c2s(7, 0));
        board.set_piece(types::ROOK, types::BLACK, c2s(0, 7));
        board.set_piece(types::ROOK, types::BLACK, c2s(7, 7));

        // queens
        board.set_piece(types::QUEEN, types::WHITE, c2s(3, 0));
        board.set_piece(types::QUEEN, types::BLACK, c2s(3, 7));

        // kings
        board.set_piece(types::KING, types::WHITE, c2s(4, 0));
        board.set_piece(types::KING, types::BLACK, c2s(4, 7));

        board.castling = 0xf;
        board
    }

    

    fn get_pieces(&self, piece: u32, color: u32) -> u64 {
        self.bb[piece as usize] & self.bb[color as usize]
    }

    fn set_piece(&mut self, piece: u32, color: u32, square: u32) {
        self.bb[piece as usize] |= 1 << square;
        self.bb[color as usize] |= 1 << square;
    }

    fn make_move(&mut self, piece: u32, color: u32, mov: Move) {

    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_has_correct_piece_enum_values() {
        assert_eq!(0, types::WHITE as usize);
        assert_eq!(1, types::BLACK as usize);
        assert_eq!(2, types::PAWN as usize);
        assert_eq!(3, types::KNIGHT as usize);
        assert_eq!(4, types::BISHOP as usize);
        assert_eq!(5, types::ROOK as usize);
        assert_eq!(6, types::QUEEN as usize);
        assert_eq!(7, types::KING as usize);
    }

    #[test]
    fn it_has_correct_color_enum_values() {
        assert_eq!(0, types::WHITE as usize);
        assert_eq!(1, types::BLACK as usize);
    }
    
    #[test]
    fn it_sets_correct_startpos() {
        let b = Board::startpos(); // Board { bb: [0; 8] };
        
        // color boards
        assert_eq!(0xffff, b.bb[types::WHITE as usize]);
        assert_eq!(0xffff << 6*8, b.bb[types::BLACK as usize]);

        // pawn boards
        assert_eq!(0xff << 8, b.bb[types::PAWN as usize] & b.bb[types::WHITE as usize]);
        assert_eq!(0xff << 8, b.get_pieces(types::PAWN, types::WHITE));
        assert_eq!(0xff << 6*8, b.bb[types::PAWN as usize] & b.bb[types::BLACK as usize]);
        assert_eq!(0xff << 6*8, b.get_pieces(types::PAWN, types::BLACK));

        // rook boards
        assert_eq!(0x81, b.bb[types::ROOK as usize] & b.bb[types::WHITE as usize]);
        assert_eq!(0x81, b.get_pieces(types::ROOK, types::WHITE));
        assert_eq!(0x81 << 7*8, b.bb[types::ROOK as usize] & b.bb[types::BLACK as usize]);
        assert_eq!(0x81 << 7*8, b.get_pieces(types::ROOK, types::BLACK));
        

        // bishop boards
        assert_eq!(0x24, b.bb[types::BISHOP as usize] & b.bb[types::WHITE as usize]);
        assert_eq!(0x24, b.get_pieces(types::BISHOP, types::WHITE));
        assert_eq!(0x24 << 7*8, b.bb[types::BISHOP as usize] & b.bb[types::BLACK as usize]);
        assert_eq!(0x24 << 7*8, b.get_pieces(types::BISHOP, types::BLACK));

        // queen boards
        assert_eq!(0x8, b.bb[types::QUEEN as usize] & b.bb[types::WHITE as usize]);
        assert_eq!(0x8, b.get_pieces(types::QUEEN, types::WHITE));
        assert_eq!(0x8 << 7*8, b.bb[types::QUEEN as usize] & b.bb[types::BLACK as usize]);
        assert_eq!(0x8 << 7*8, b.get_pieces(types::QUEEN, types::BLACK));

        // king boards
        assert_eq!(0x10, b.bb[types::KING as usize] & b.bb[types::WHITE as usize]);
        assert_eq!(0x10, b.get_pieces(types::KING, types::WHITE));
        assert_eq!(0x10 << 7*8, b.bb[types::KING as usize] & b.bb[types::BLACK as usize]);
        assert_eq!(0x10 << 7*8, b.get_pieces(types::KING, types::BLACK));
    }
}