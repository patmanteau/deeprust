use super::types;
use super::san::SAN;

use std::str::FromStr;

pub struct Board {
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
    pub fn new() -> Board {
        Board { 
            bb: [0; 8],
            to_move: types::WHITE,
            castling: 0,
            en_passant: None,
            halfmoves: 0,
            fullmoves: 1,
        }
    }

    pub fn startpos() -> Board {
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

    pub fn from_fen(fen_string: String) -> Result<Board, &'static str> {
        let mut fen_iter = fen_string.split_whitespace();

        let mut board = Board::new(); // Board { bb: [0; 8] };

        // position
        let mut x = 0;
        let mut y = 7;
        if let Some(position) = fen_iter.next() {
            for chr in position.chars() {
                if let Some(empty) = chr.to_digit(10) {
                    x += empty
                } else {
                    if chr == '/' {
                        x = 0;
                        y -= 1;
                    } else {
                        match chr {
                            'P' => board.set_piece(types::PAWN, types::WHITE, c2s(x, y)),
                            'N' => board.set_piece(types::KNIGHT, types::WHITE, c2s(x, y)),
                            'B' => board.set_piece(types::BISHOP, types::WHITE, c2s(x, y)),
                            'R' => board.set_piece(types::ROOK, types::WHITE, c2s(x, y)),
                            'Q' => board.set_piece(types::QUEEN, types::WHITE, c2s(x, y)),
                            'K' => board.set_piece(types::KING, types::WHITE, c2s(x, y)),
                            'p' => board.set_piece(types::PAWN, types::BLACK, c2s(x, y)),
                            'n' => board.set_piece(types::KNIGHT, types::BLACK, c2s(x, y)),
                            'b' => board.set_piece(types::BISHOP, types::BLACK, c2s(x, y)),
                            'r' => board.set_piece(types::ROOK, types::BLACK, c2s(x, y)),
                            'q' => board.set_piece(types::QUEEN, types::BLACK, c2s(x, y)),
                            'k' => board.set_piece(types::KING, types::BLACK, c2s(x, y)),
                            _ => { return Err("Invalid FEN string") },
                        }
                        x += 1;
                    }
                }
            }    
        } else {
            return Err("Invalid FEN string, no position found");
        }

        // to move
        if let Some(to_move) = fen_iter.next() {
            match to_move {
                "w" => board.to_move = types::WHITE,
                "b" => board.to_move = types::BLACK,
                _ => return Err("Invalid ToMove char")
            }
        } else {
            return Err("Invalid FEN string, don't know who moves next")
        }

        // Castling rights
        if let Some(castling) = fen_iter.next() {
            for chr in castling.chars() {
                match chr {
                    '-' => board.castling = 0,
                    'K' => board.castling |= 0x1,
                    'Q' => board.castling |= 0x2,
                    'k' => board.castling |= 0x4,
                    'q' => board.castling |= 0x8,
                    _ => return Err("Invalid castling char")
                }
            }
        } else {
            return Err("Invalid FEN string, no castling rights found")
        }

        // en passant
        if let Some(en_passant) = fen_iter.next() {
            if en_passant == "-" {
                board.en_passant = None;
            } else {
                match SAN::square_str_to_index(en_passant) {
                    Ok(eps) => board.en_passant = Some(eps),
                    Err(_) => return Err("Error parsing en passant field"),
                }
            }
        } else {
            return Err("Invalid FEN string, no en passant information")
        }

        // Halfmoves
        if let Some(halfmoves) = fen_iter.next() {
            match u32::from_str(halfmoves) {
                Ok(val) => board.halfmoves = val,
                Err(_) => return Err("Error parsing halfmoves"),
            }
        } else {
            return Err("Invalid FEN string, no halfmoves given")
        }
        
        // Fullmoves
        if let Some(fullmoves) = fen_iter.next() {
            match u32::from_str(fullmoves) {
                Ok(val) => board.fullmoves = val,
                Err(_) => return Err("Error parsing fullmoves"),
            }
        } else {
            return Err("Invalid FEN string, no fullmoves given")
        }
        
        Ok(board)
    }

    pub fn to_fen(&self) -> String {
        let mut fen_string = String::new();

        // Position
        for y in (0..8).rev() {
            let mut emptycount = 0;
            for x in 0..8 {
                if 0 == ( (self.bb[types::WHITE as usize] | self.bb[types::BLACK as usize]) & (1 << ((y << 3) + x))) {
                    emptycount += 1;
                } else {
                    if emptycount > 0 { 
                        fen_string.push_str(&emptycount.to_string());
                        emptycount = 0;
                    };

                    if 0 != (self.bb[types::PAWN as usize] & self.bb[types::WHITE as usize] & (1 << c2s(x, y))) { fen_string.push('P')}
                    else if 0 != self.bb[types::PAWN as usize] & self.bb[types::BLACK as usize] & (1 << c2s(x, y)) { fen_string.push('p')}
                    else if 0 != self.bb[types::KNIGHT as usize] & self.bb[types::WHITE as usize] & (1 << c2s(x, y)) { fen_string.push('N')}
                    else if 0 != self.bb[types::KNIGHT as usize] & self.bb[types::BLACK as usize] & (1 << c2s(x, y)) { fen_string.push('n')}
                    else if 0 != self.bb[types::BISHOP as usize] & self.bb[types::WHITE as usize] & (1 << c2s(x, y)) { fen_string.push('B')}
                    else if 0 != self.bb[types::BISHOP as usize] & self.bb[types::BLACK as usize] & (1 << c2s(x, y)) { fen_string.push('b')}
                    else if 0 != self.bb[types::ROOK as usize] & self.bb[types::WHITE as usize] & (1 << c2s(x, y)) { fen_string.push('R')}
                    else if 0 != self.bb[types::ROOK as usize] & self.bb[types::BLACK as usize] & (1 << c2s(x, y)) { fen_string.push('r')}
                    else if 0 != self.bb[types::QUEEN as usize] & self.bb[types::WHITE as usize] & (1 << c2s(x, y)) { fen_string.push('Q')}
                    else if 0 != self.bb[types::QUEEN as usize] & self.bb[types::BLACK as usize] & (1 << c2s(x, y)) { fen_string.push('q')}
                    else if 0 != self.bb[types::KING as usize] & self.bb[types::WHITE as usize] & (1 << c2s(x, y)) { fen_string.push('K')}
                    else if 0 != self.bb[types::KING as usize] & self.bb[types::BLACK as usize] & (1 << c2s(x, y)) { fen_string.push('k')};
                }
            }
            if emptycount > 0 {
                fen_string.push_str(&emptycount.to_string());
                // emptycount = 0;
            };
            if y > 0 { fen_string.push('/'); }
        }

        // To move
        fen_string.push(' ');
        let to_move = match self.to_move {
            types::WHITE => 'w',
            types::BLACK => 'b',
            _ => 'w',
        };
        fen_string.push(to_move);

        // Castling rights
        fen_string.push(' ');
        if self.castling == 0 {
            fen_string.push('-');
        } else {
            if 0 != self.castling & 0x1 { fen_string.push('K'); }
            if 0 != self.castling & 0x2 { fen_string.push('Q'); }
            if 0 != self.castling & 0x4 { fen_string.push('k'); }
            if 0 != self.castling & 0x8 { fen_string.push('q'); }
        }

        // en passant
        fen_string.push(' ');
        if let Some(eps) = self.en_passant {
            let san = SAN::from_square(eps);
            fen_string.push_str(&san.s.to_string())
        } else {
            fen_string.push('-')
        }

        // Halfmoves
        fen_string.push(' ');
        fen_string.push_str(&self.halfmoves.to_string());
        
        // Fullmoves
        fen_string.push(' ');
        fen_string.push_str(&self.fullmoves.to_string());
        
        fen_string
    }


    fn get_pieces(&self, piece: u32, color: u32) -> u64 {
        self.bb[piece as usize] & self.bb[color as usize]
    }

    fn check_piece(&self, color: u32, piece: u32, square: u32) -> bool {
        return 0 != 
            (self.bb[color as usize] & (1 << square)) &
            (self.bb[piece as usize] & (1 << square))
    }

    fn set_piece(&mut self, piece: u32, color: u32, square: u32) {
        self.bb[piece as usize] |= 1 << square;
        self.bb[color as usize] |= 1 << square;
    }

    pub fn piece_at_square(&self, square: u32) -> Result<(u32, u32), &'static str> {
        for color in 0..2 {
            for piece in 2..8 {
                if 0 != (self.bb[piece as usize] & self.bb[color as usize] & (1 << square)) { return Ok((piece, color)); }
            }
        }
        return Err("No piece at square");
    }

    fn make_move(&mut self, mov: types::Move) {
        debug_assert!(self.check_piece(mov.color(), mov.piece(), mov.from()));
    }

    pub fn input_move(&mut self, from: u32, to: u32) -> Result<bool, &'static str> {
        match self.piece_at_square(from) {
            Ok((piece, color)) => {
                let mov = types::Move::new(from, to, color, piece, 0, 0);
                self.make_move(mov);
            },
            Err(e) => return Err(e)
        }
        Ok(true)
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

    #[test]
    fn it_makes_correct_fen_strings() {
        let board = Board::startpos();
        assert_eq!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", board.to_fen());
    }

    #[test]
    fn it_parses_fen_strings_correctly() {
        let b = Board::from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"));
        match b {
            Err(e) => assert!(false, e),
            Ok(board) => assert_eq!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", board.to_fen()),
        }

        let b = Board::from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - e3 0 1"));
        match b {
            Err(e) => assert!(false, e),
            Ok(board) => assert_eq!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - e3 0 1", board.to_fen()),
        }
    }

    #[test]
    fn it_rejects_invalid_fen_strings() {
        let b = Board::from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR"));
        match b {
            Err(e) => assert!(true),
            Ok(board) => assert!(false),
        }

        let b = Board::from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq abcdefg 0 1"));
        match b {
            Err(e) => assert!(true),
            Ok(board) => assert!(false),
        }

        let b = Board::from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR y KQkq e3 0 1"));
        match b {
            Err(e) => assert!(true),
            Ok(board) => assert!(false),
        }

        let b = Board::from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w HFhf e3 0 1"));
        match b {
            Err(e) => assert!(true),
            Ok(board) => assert!(false),
        }
    }

}