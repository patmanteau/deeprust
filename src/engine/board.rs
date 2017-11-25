use super::types::{Piece, Color, Move};
use super::san::SAN;

use std::str::FromStr;

struct Board {
    bb: [u64; 8],
    to_move: Color,
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
            to_move: Color::White,
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
            board.set_piece(Piece::Pawn, Color::White, c2s(x, 1));
            board.set_piece(Piece::Pawn, Color::Black, c2s(x, 6));
        }

        // knights
        board.set_piece(Piece::Knight, Color::White, c2s(1, 0));
        board.set_piece(Piece::Knight, Color::White, c2s(6, 0));
        board.set_piece(Piece::Knight, Color::Black, c2s(1, 7));
        board.set_piece(Piece::Knight, Color::Black, c2s(6, 7));

        // bishops
        board.set_piece(Piece::Bishop, Color::White, c2s(2, 0));
        board.set_piece(Piece::Bishop, Color::White, c2s(5, 0));
        board.set_piece(Piece::Bishop, Color::Black, c2s(2, 7));
        board.set_piece(Piece::Bishop, Color::Black, c2s(5, 7));

        // rooks
        board.set_piece(Piece::Rook, Color::White, c2s(0, 0));
        board.set_piece(Piece::Rook, Color::White, c2s(7, 0));
        board.set_piece(Piece::Rook, Color::Black, c2s(0, 7));
        board.set_piece(Piece::Rook, Color::Black, c2s(7, 7));

        // queens
        board.set_piece(Piece::Queen, Color::White, c2s(3, 0));
        board.set_piece(Piece::Queen, Color::Black, c2s(3, 7));

        // kings
        board.set_piece(Piece::King, Color::White, c2s(4, 0));
        board.set_piece(Piece::King, Color::Black, c2s(4, 7));

        board.castling = 0xf;
        board
    }

    fn from_fen(fen_string: String) -> Result<Board, &'static str> {
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
                            'P' => board.set_piece(Piece::Pawn, Color::White, c2s(x, y)),
                            'N' => board.set_piece(Piece::Knight, Color::White, c2s(x, y)),
                            'B' => board.set_piece(Piece::Bishop, Color::White, c2s(x, y)),
                            'R' => board.set_piece(Piece::Rook, Color::White, c2s(x, y)),
                            'Q' => board.set_piece(Piece::Queen, Color::White, c2s(x, y)),
                            'K' => board.set_piece(Piece::King, Color::White, c2s(x, y)),
                            'p' => board.set_piece(Piece::Pawn, Color::Black, c2s(x, y)),
                            'n' => board.set_piece(Piece::Knight, Color::Black, c2s(x, y)),
                            'b' => board.set_piece(Piece::Bishop, Color::Black, c2s(x, y)),
                            'r' => board.set_piece(Piece::Rook, Color::Black, c2s(x, y)),
                            'q' => board.set_piece(Piece::Queen, Color::Black, c2s(x, y)),
                            'k' => board.set_piece(Piece::King, Color::Black, c2s(x, y)),
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
                "w" => board.to_move = Color::White,
                "b" => board.to_move = Color::Black,
                val => return Err("Invalid ToMove char")
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
                    val => return Err("Invalid castling char")
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
                    Err(e) => return Err("Error parsing en passant field"),
                }
            }
        } else {
            return Err("Invalid FEN string, no en passant information")
        }

        // Halfmoves
        if let Some(halfmoves) = fen_iter.next() {
            match u32::from_str(halfmoves) {
                Ok(val) => board.halfmoves = val,
                Err(e) => return Err("Error parsing halfmoves"),
            }
        } else {
            return Err("Invalid FEN string, no halfmoves given")
        }
        
        // Fullmoves
        if let Some(fullmoves) = fen_iter.next() {
            match u32::from_str(fullmoves) {
                Ok(val) => board.fullmoves = val,
                Err(e) => return Err("Error parsing fullmoves"),
            }
        } else {
            return Err("Invalid FEN string, no fullmoves given")
        }

        
        Ok(board)
    }

    fn to_fen(&self) -> String {
        let mut fen_string = String::new();

        // Position
        for y in (0..8).rev() {
            let mut emptycount = 0;
            for x in 0..8 {
                if 0 == ( (self.bb[Color::White as usize] | self.bb[Color::Black as usize]) & (1 << ((y << 3) + x))) {
                    emptycount += 1;
                } else {
                    if emptycount > 0 { 
                        fen_string.push_str(&emptycount.to_string());
                        emptycount = 0;
                    };

                    if 0 != (self.bb[Piece::Pawn as usize] & self.bb[Color::White as usize] & (1 << c2s(x, y))) { fen_string.push('P')}
                    else if 0 != self.bb[Piece::Pawn as usize] & self.bb[Color::Black as usize] & (1 << c2s(x, y)) { fen_string.push('p')}
                    else if 0 != self.bb[Piece::Knight as usize] & self.bb[Color::White as usize] & (1 << c2s(x, y)) { fen_string.push('N')}
                    else if 0 != self.bb[Piece::Knight as usize] & self.bb[Color::Black as usize] & (1 << c2s(x, y)) { fen_string.push('n')}
                    else if 0 != self.bb[Piece::Bishop as usize] & self.bb[Color::White as usize] & (1 << c2s(x, y)) { fen_string.push('B')}
                    else if 0 != self.bb[Piece::Bishop as usize] & self.bb[Color::Black as usize] & (1 << c2s(x, y)) { fen_string.push('b')}
                    else if 0 != self.bb[Piece::Rook as usize] & self.bb[Color::White as usize] & (1 << c2s(x, y)) { fen_string.push('R')}
                    else if 0 != self.bb[Piece::Rook as usize] & self.bb[Color::Black as usize] & (1 << c2s(x, y)) { fen_string.push('r')}
                    else if 0 != self.bb[Piece::Queen as usize] & self.bb[Color::White as usize] & (1 << c2s(x, y)) { fen_string.push('Q')}
                    else if 0 != self.bb[Piece::Queen as usize] & self.bb[Color::Black as usize] & (1 << c2s(x, y)) { fen_string.push('q')}
                    else if 0 != self.bb[Piece::King as usize] & self.bb[Color::White as usize] & (1 << c2s(x, y)) { fen_string.push('K')}
                    else if 0 != self.bb[Piece::King as usize] & self.bb[Color::Black as usize] & (1 << c2s(x, y)) { fen_string.push('k')};
                }
            }
            if emptycount > 0 {
                fen_string.push_str(&emptycount.to_string());
                emptycount = 0;
            };
            if y > 0 { fen_string.push('/'); }
        }

        // To move
        fen_string.push(' ');
        let to_move = match self.to_move {
            Color::White => 'w',
            Color::Black => 'b',
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

    fn get_pieces(&self, piece: Piece, color: Color) -> u64 {
        self.bb[piece as usize] & self.bb[color as usize]
    }

    fn set_piece(&mut self, piece: Piece, color: Color, square: u32) {
        self.bb[piece as usize] |= 1 << square;
        self.bb[color as usize] |= 1 << square;
    }

    fn make_move(&mut self, piece: Piece, color: Color, mov: Move) {

    }
}

mod utils {
    use super::*;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_has_correct_piece_enum_values() {
        assert_eq!(0, Piece::White as usize);
        assert_eq!(1, Piece::Black as usize);
        assert_eq!(2, Piece::Pawn as usize);
        assert_eq!(3, Piece::Knight as usize);
        assert_eq!(4, Piece::Bishop as usize);
        assert_eq!(5, Piece::Rook as usize);
        assert_eq!(6, Piece::Queen as usize);
        assert_eq!(7, Piece::King as usize);
    }

    #[test]
    fn it_has_correct_color_enum_values() {
        assert_eq!(0, Piece::White as usize);
        assert_eq!(1, Piece::Black as usize);
    }
    
    #[test]
    fn it_sets_correct_startpos() {
        let b = Board::startpos(); // Board { bb: [0; 8] };
        
        // color boards
        assert_eq!(0xffff, b.bb[Color::White as usize]);
        assert_eq!(0xffff << 6*8, b.bb[Color::Black as usize]);

        // pawn boards
        assert_eq!(0xff << 8, b.bb[Piece::Pawn as usize] & b.bb[Color::White as usize]);
        assert_eq!(0xff << 8, b.get_pieces(Piece::Pawn, Color::White));
        assert_eq!(0xff << 6*8, b.bb[Piece::Pawn as usize] & b.bb[Color::Black as usize]);
        assert_eq!(0xff << 6*8, b.get_pieces(Piece::Pawn, Color::Black));

        // rook boards
        assert_eq!(0x81, b.bb[Piece::Rook as usize] & b.bb[Color::White as usize]);
        assert_eq!(0x81, b.get_pieces(Piece::Rook, Color::White));
        assert_eq!(0x81 << 7*8, b.bb[Piece::Rook as usize] & b.bb[Color::Black as usize]);
        assert_eq!(0x81 << 7*8, b.get_pieces(Piece::Rook, Color::Black));
        

        // bishop boards
        assert_eq!(0x24, b.bb[Piece::Bishop as usize] & b.bb[Color::White as usize]);
        assert_eq!(0x24, b.get_pieces(Piece::Bishop, Color::White));
        assert_eq!(0x24 << 7*8, b.bb[Piece::Bishop as usize] & b.bb[Color::Black as usize]);
        assert_eq!(0x24 << 7*8, b.get_pieces(Piece::Bishop, Color::Black));

        // queen boards
        assert_eq!(0x8, b.bb[Piece::Queen as usize] & b.bb[Color::White as usize]);
        assert_eq!(0x8, b.get_pieces(Piece::Queen, Color::White));
        assert_eq!(0x8 << 7*8, b.bb[Piece::Queen as usize] & b.bb[Color::Black as usize]);
        assert_eq!(0x8 << 7*8, b.get_pieces(Piece::Queen, Color::Black));

        // king boards
        assert_eq!(0x10, b.bb[Piece::King as usize] & b.bb[Color::White as usize]);
        assert_eq!(0x10, b.get_pieces(Piece::King, Color::White));
        assert_eq!(0x10 << 7*8, b.bb[Piece::King as usize] & b.bb[Color::Black as usize]);
        assert_eq!(0x10 << 7*8, b.get_pieces(Piece::King, Color::Black));
    }

    #[test]
    fn it_makes_correct_fen_strings() {
        let b = Board::startpos();
        assert_eq!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", b.to_fen());
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