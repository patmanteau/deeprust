use super::types;
use super::san::SAN;
use ::bits;

use std::str::FromStr;

pub struct Board {
    bb: [u64; 8],
    occupied: [u32; 64],
    to_move: u32,
    castling: u32,
    en_passant: Option<u32>,
    halfmoves: u32,
    fullmoves: u32,

    square_bb_t: [u64; 64],
}

pub fn c2s(x: u32, y: u32) -> u32 {
    (y * 8) + x
}

#[inline]
pub fn lookup_ep_capture(ep_square: u32) -> u32 {
    let table = [
         0,  0,  0,  0,  0,  0,  0,  0, 
         0,  0,  0,  0,  0,  0,  0,  0, 
        24, 25, 26, 27, 28, 29, 30, 31,
         0,  0,  0,  0,  0,  0,  0,  0, 
         0,  0,  0,  0,  0,  0,  0,  0, 
        32, 33, 34, 35, 36, 37, 38, 39,
         0,  0,  0,  0,  0,  0,  0,  0, 
         0,  0,  0,  0,  0,  0,  0,  0, 
    ];
    table[ep_square as usize]
}

pub fn occ_piece_code_to_str(code: u32) -> &'static str {
    match code {
        2 => "P",
        3 => "N",
        4 => "B",
        5 => "R",
        6 => "Q",
        7 => "K",
        10 => "p",
        11 => "n",
        12 => "b",
        13 => "r",
        14 => "q",
        15 => "k",
        _ => " ",
    }
}

impl Board {
    pub fn new() -> Board {
        let mut board = Board { 
            bb: [0; 8],
            occupied: [0; 64],
            to_move: types::WHITE,
            castling: 0,
            en_passant: None,
            halfmoves: 0,
            fullmoves: 1,
            square_bb_t: [0; 64],
        };

        for i in 0..64 {
            board.square_bb_t[i] = 1 << i;
        }

        board
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
                if 0 == self.occupied[c2s(x, y) as usize] {
                    emptycount += 1;
                } else {
                    if emptycount > 0 { 
                        fen_string.push_str(&emptycount.to_string());
                        emptycount = 0;
                    };
                    fen_string.push_str(occ_piece_code_to_str(self.occupied[c2s(x, y) as usize]));
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

    #[inline]
    pub fn occupied(&self) -> &[u32; 64] {
        &self.occupied
    }

    #[inline]
    fn square_bb(&self, square: u32) -> u64 {
        self.square_bb_t[square as usize]
    }

    #[inline]
    fn get_piece_and_color(&self, square: u32) -> (u32, u32) {
        let raw = self.occupied[square as usize];
        ((raw & 0x7), (raw >> 3))
    }

    fn get_pieces(&self, piece: u32, color: u32) -> u64 {
        self.bb[piece as usize] & self.bb[color as usize]
    }

    fn check_piece(&self, piece: u32, color: u32, square: u32) -> bool {
        0 != self.occupied[square as usize]
    }

    #[inline]
    fn set_piece(&mut self, piece: u32, color: u32, to: u32) {
        self.bb[piece as usize] |= self.square_bb(to);
        self.bb[color as usize] |= self.square_bb(to);
        self.occupied[to as usize] = (color << 3) | (piece & 0x7);
    }

    #[inline]
    fn remove_piece(&mut self, piece: u32, color: u32, from: u32) {
        // self.bb[color as usize] ^= self.square_bb(from);
        bits::flip_bit(self.bb[color as usize], from as usize);
        // self.bb[piece as usize] ^= self.square_bb(from);
        bits::flip_bit(self.bb[piece as usize], from as usize);
        
        self.occupied[from as usize] = 0;
    }

    #[inline]
    fn replace_piece(&mut self, old_piece: u32, old_color: u32, new_piece: u32, new_color: u32, square: u32) {
        self.bb[old_color as usize] ^= self.square_bb(square);
        self.bb[old_piece as usize] ^= self.square_bb(square);
        self.set_piece(new_piece, new_color, square);
    }

    // pub fn piece_at_square(&self, square: u32) -> Result<(u32, u32), &'static str> {
    //     for color in 0..2 {
    //         for piece in 2..8 {
    //             if 0 != (self.bb[piece as usize] & self.bb[color as usize] & self.square_bb(square)) { return Ok((piece, color)); }
    //         }
    //     }
    //     return Err("No piece at square");
    // }

    fn make_move(&mut self, mov: types::Move) {
        // fail if no piece at origin square
        // debug_assert!(self.check_piece(mov.piece(), mov.color(), mov.from()));
        
        // Full move clock needs to be incremented after black moves
        // types::WHITE == 0 and types::BLACK == 1, so we use that to save an if :-)
        self.fullmoves += mov.color();
        
        // set half move clock
        if mov.piece() == types::PAWN || mov.is_capture() {
            self.halfmoves = 0; // reset half move clock on pawn moves and captures
        } else {
            self.halfmoves += 1;
        }

        // remove the origin piece
        self.remove_piece(mov.piece(), mov.color(), mov.orig());

        // promotions change pieces
        let piece = if mov.is_promotion() { 
            mov.special() + 3
        } else {
            mov.piece()
        };
    
        if mov.is_capture_en_passant() {
            self.remove_piece(types::PAWN, 1 ^ mov.color(), lookup_ep_capture(mov.dest()));
            self.set_piece(piece, mov.color(), mov.dest());
        } else if mov.is_capture() {
            self.replace_piece(mov.captured_piece(), 1 ^ mov.color(), piece, mov.color(), mov.dest());
        } else if mov.is_king_castle() {
            self.set_piece(piece, mov.color(), mov.dest());
            // move rook
            self.remove_piece(types::ROOK, mov.color(), mov.dest()+1);
            self.set_piece(types::ROOK, mov.color(), mov.dest()-1);
        } else if mov.is_queen_castle() {
            self.set_piece(piece, mov.color(), mov.dest());
            // move rook
            self.remove_piece(types::ROOK, mov.color(), mov.dest()-2);
            self.set_piece(types::ROOK, mov.color(), mov.dest()+1);
        } else {
            if mov.is_double_pawn_push() {
                self.en_passant = Some((mov.dest() as i32 - [8i32, -8i32][mov.color() as usize]) as u32);
                // self.en_passant = Some(mov.dest().wrapping_sub([8i32, -8i32][mov.color() as usize]));
            } else {
                self.en_passant = None;
            }
            self.set_piece(piece, mov.color(), mov.dest());
        }

        // flip to move
        self.to_move ^= 1;
    }

    pub fn input_move(&mut self, orig: u32, dest: u32, promote_to: Option<u32>) -> Result<bool, &'static str> {
        let (mut is_capture, mut is_promotion, mut is_special_0, mut is_special_1) = (false, false, false, false);
        let (piece, color) = self.get_piece_and_color(orig);
        if 0 == piece {
            return Err("No piece at given square")
        };
        
        let (cap_piece, _) = self.get_piece_and_color(dest);
        is_capture = 0 != cap_piece;

        // set special flags for double pawn push
        if piece == types::PAWN && ((orig + 16 == dest) || (dest + 16 == orig)) {
            is_special_0 = true;
            is_special_1 = false;
        }

        // set flags for en passant capture
        if piece == types::PAWN && Some(dest) == self.en_passant {
            is_special_0 = true;
            is_special_1 = false;
        }

        // set flags for promotion
        if let Some(promoted_piece) = promote_to {
            is_promotion = true;
            if (types::BISHOP == promoted_piece) || (types::QUEEN == promoted_piece) {
                is_special_0 = true;
            }
            if (types::ROOK == promoted_piece) || (types::QUEEN == promoted_piece) {
                is_special_1 = true;
            }
        }

        // set flags for castling
        if piece == types::KING {
            if  2 == dest.wrapping_sub(orig) { // King castle
                is_special_0 = false;
                is_special_1 = true;
            } else if 3 == orig.wrapping_sub(dest) { // Queen castle
                is_special_0 = true;
                is_special_1 = true;
            }
        }
        
        let mov = types::Move::new(orig, dest, color, piece, 
                                    types::Move::make_flags(is_capture, is_promotion, is_special_0, is_special_1),
                                    types::Move::make_extended(cap_piece, self.castling));
        self.make_move(mov);
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::fs::File;
    use std::io::{BufReader, BufRead};
    use std::path::Path;

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
    fn it_sets_pieces() {
        // full board
        let mut board = Board::new();
        for square in 0..64 {
            for color in 0..2 {
                for piece in 2..8 {
                    board.set_piece(piece, color, square);
                    assert!(board.check_piece(piece, color, square));
                    assert!(0 != board.bb[color as usize] & board.square_bb(square));
                    assert!(0 != board.bb[piece as usize] & board.square_bb(square));
                    assert_eq!(piece, board.occupied[square as usize] & 0x7);
                    assert_eq!(color, board.occupied[square as usize] >> 3);
                }
            }
        }

        // single pieces
        for square in 0..64 {
            for color in 0..2 {
                for piece in 2..8 {
                    let mut board = Board::new();
                    board.set_piece(piece, color, square);
                    assert!(board.check_piece(piece, color, square));
                    assert!(0 != board.bb[color as usize] & board.square_bb(square));
                    assert!(0 != board.bb[piece as usize] & board.square_bb(square));
                    assert_eq!(piece, board.occupied[square as usize] & 0x7);
                    assert_eq!(color, board.occupied[square as usize] >> 3);
                }
            }
        }
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

        let pospath = Path::new("tests/hyatt-4000-openings.epd");
        let mut posfile = match File::open(&pospath) {
            Err(why) => panic!("Could not open {}: {}", pospath.display(), why.description()),
            Ok(file) => file,
        };

        for position in BufReader::new(posfile).lines().map(|l| l.unwrap()) {
            let b = Board::from_fen(position.clone());
            match b {
                Err(e) => assert!(false, e),
                Ok(board) => assert_eq!(position, board.to_fen()),
            }
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

    #[test]
    fn it_calculates_ep_squares_correctly() {
        for x in 0..8 {
            // white
            assert_eq!(c2s(x, 3), lookup_ep_capture(c2s(x, 2)));
            // black
            assert_eq!(c2s(x, 4), lookup_ep_capture(c2s(x, 5)));
        }
    }
}