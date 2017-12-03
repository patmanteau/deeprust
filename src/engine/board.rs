use ::engine::types::{Sq};
use super::util;
use super::util::{piece, squares};
use super::san::SAN;
use super::moves::{Move, UnmakeInfo};
use super::move_stack::{MoveStack, MoveStackEntry};
use ::bits;

use std::str::FromStr;

/// Represents a chess position
pub struct Board {
    bb: [u64; 8],
    occupied: [u32; 64],
    to_move: u32,
    castling: u32,
    en_passant: Option<Sq>,
    halfmoves: u32,
    fullmoves: u32,

    move_stack: MoveStack,
    square_bb_t: [u64; 64],
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
            to_move: piece::WHITE,
            castling: 0,
            en_passant: None,
            halfmoves: 0,
            fullmoves: 1,
            move_stack: MoveStack::new(),
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
            board.set_piece(piece::PAWN, piece::WHITE, util::square(x, 1));
            board.set_piece(piece::PAWN, piece::BLACK, util::square(x, 6));
        }

        // knights
        board.set_piece(piece::KNIGHT, piece::WHITE, squares::B1);
        board.set_piece(piece::KNIGHT, piece::WHITE, squares::G1);
        board.set_piece(piece::KNIGHT, piece::BLACK, squares::B8);
        board.set_piece(piece::KNIGHT, piece::BLACK, squares::G8);

        // bishops
        board.set_piece(piece::BISHOP, piece::WHITE, squares::C1);
        board.set_piece(piece::BISHOP, piece::WHITE, squares::F1);
        board.set_piece(piece::BISHOP, piece::BLACK, squares::C8);
        board.set_piece(piece::BISHOP, piece::BLACK, squares::F8);

        // rooks
        board.set_piece(piece::ROOK, piece::WHITE, squares::A1);
        board.set_piece(piece::ROOK, piece::WHITE, squares::H1);
        board.set_piece(piece::ROOK, piece::BLACK, squares::A8);
        board.set_piece(piece::ROOK, piece::BLACK, squares::H8);

        // queens
        board.set_piece(piece::QUEEN, piece::WHITE, squares::D1);
        board.set_piece(piece::QUEEN, piece::BLACK, squares::D8);

        // kings
        board.set_piece(piece::KING, piece::WHITE, squares::E1);
        board.set_piece(piece::KING, piece::BLACK, squares::E8);

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
                            'P' => board.set_piece(piece::PAWN, piece::WHITE, util::square(x, y)),
                            'N' => board.set_piece(piece::KNIGHT, piece::WHITE, util::square(x, y)),
                            'B' => board.set_piece(piece::BISHOP, piece::WHITE, util::square(x, y)),
                            'R' => board.set_piece(piece::ROOK, piece::WHITE, util::square(x, y)),
                            'Q' => board.set_piece(piece::QUEEN, piece::WHITE, util::square(x, y)),
                            'K' => board.set_piece(piece::KING, piece::WHITE, util::square(x, y)),
                            'p' => board.set_piece(piece::PAWN, piece::BLACK, util::square(x, y)),
                            'n' => board.set_piece(piece::KNIGHT, piece::BLACK, util::square(x, y)),
                            'b' => board.set_piece(piece::BISHOP, piece::BLACK, util::square(x, y)),
                            'r' => board.set_piece(piece::ROOK, piece::BLACK, util::square(x, y)),
                            'q' => board.set_piece(piece::QUEEN, piece::BLACK, util::square(x, y)),
                            'k' => board.set_piece(piece::KING, piece::BLACK, util::square(x, y)),
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
                "w" => board.to_move = piece::WHITE,
                "b" => board.to_move = piece::BLACK,
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
                if 0 == self.occupied[util::square(x, y) as usize] {
                    emptycount += 1;
                } else {
                    if emptycount > 0 { 
                        fen_string.push_str(&emptycount.to_string());
                        emptycount = 0;
                    };
                    fen_string.push_str(occ_piece_code_to_str(self.occupied[util::square(x, y) as usize]));
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
            piece::WHITE => 'w',
            piece::BLACK => 'b',
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

    pub fn move_stack(&self) -> &MoveStack {
        &self.move_stack
    }

    #[inline]
    pub fn occupied(&self) -> &[u32; 64] {
        &self.occupied
    }

    #[inline]
    fn square_bb(&self, square: Sq) -> u64 {
        self.square_bb_t[square as usize]
    }

    #[inline]
    fn get_piece_and_color(&self, square: Sq) -> (u32, u32) {
        let raw = self.occupied[square as usize];
        ((raw & 0x7), (raw >> 3))
    }

    fn get_pieces(&self, piece: u32, color: u32) -> u64 {
        self.bb[piece as usize] & self.bb[color as usize]
    }

    fn check_piece(&self, piece: u32, color: u32, square: Sq) -> bool {
        0 != self.occupied[square as usize]
    }

    #[inline]
    fn set_piece(&mut self, piece: u32, color: u32, to: Sq) {
        self.bb[color as usize] |= util::bb::BB_SQUARES[to as usize];
        self.bb[piece as usize] |= util::bb::BB_SQUARES[to as usize];
        self.occupied[to as usize] = (color << 3) | (piece & 0x7);
    }

    #[inline]
    fn remove_piece(&mut self, piece: u32, color: u32, from: Sq) {
        self.bb[color as usize] ^= util::bb::BB_SQUARES[from as usize];
        self.bb[piece as usize] ^= util::bb::BB_SQUARES[from as usize];
        self.occupied[from as usize] = 0;
    }

    #[inline]
    fn replace_piece(&mut self, old_piece: u32, old_color: u32, new_piece: u32, new_color: u32, square: Sq) {
        self.bb[old_color as usize] ^= self.square_bb(square);
        self.bb[old_piece as usize] ^= self.square_bb(square);
        self.set_piece(new_piece, new_color, square);
    }

    fn make_move(&mut self, mov: Move) {
        // fail if no piece at origin square
        // debug_assert!(self.check_piece(mov.piece(), mov.color(), mov.from()));
        
        let orig_square = mov.orig();
        let dest_square = mov.dest();
        let (orig_piece, orig_color) = self.get_piece_and_color(orig_square);
        let (dest_piece, dest_color) = self.get_piece_and_color(dest_square);
        let is_capture = 0 != dest_piece;

        let ep_allowed = self.en_passant != None;
        let ep_square = match self.en_passant {
            Some(sq) => sq,
            None => 64,
        };
        let unmake_info = UnmakeInfo::new(dest_piece, dest_color, self.castling,
            ep_square, ep_allowed, self.halfmoves);

        self.move_stack.push(MoveStackEntry::new(mov, unmake_info));

        // Full move clock needs to be incremented after black moves
        // piece::WHITE == 0 and piece::BLACK == 1, so we use that to save an if :-)
        self.fullmoves += orig_color;
        
        // set half move clock
        if orig_piece == piece::PAWN || is_capture {
            self.halfmoves = 0; // reset half move clock on pawn moves and captures
        } else {
            self.halfmoves += 1;
        }

        // reset en passant
        self.en_passant = None;

        // remove the origin piece
        self.remove_piece(orig_piece, orig_color, orig_square);

        // promotions change pieces
        let piece = if mov.is_promotion() { 
            mov.special() + 3
        } else {
            orig_piece
        };

        if mov.is_quiet() {
            self.set_piece(piece, orig_color, dest_square);
        } else if mov.is_capture_en_passant() {
            self.remove_piece(piece::PAWN, 1 ^ orig_color, util::ep_capture_square(dest_square));
            self.set_piece(piece, orig_color, dest_square);
        } else if is_capture {
            self.replace_piece(dest_piece, dest_color, piece, orig_color, dest_square);
        } else if mov.is_double_pawn_push() {
            self.en_passant = Some((dest_square as i32 - [8i32, -8i32][orig_color as usize]) as Sq);
            self.set_piece(piece, orig_color, dest_square);
        } else if mov.is_king_castle() {
            self.set_piece(piece, orig_color, dest_square);
            // move rook
            self.remove_piece(piece::ROOK, orig_color, dest_square + 1);
            self.set_piece(piece::ROOK, orig_color, dest_square - 1);
        } else if mov.is_queen_castle() {
            self.set_piece(piece, orig_color, dest_square);
            // move rook
            self.remove_piece(piece::ROOK, orig_color, dest_square - 2);
            self.set_piece(piece::ROOK, orig_color, dest_square + 1);
        }

        if piece::KING == orig_piece {
            self.castling = bits::clear_bit(self.castling, 0 + (orig_color << 1) as usize);
            self.castling = bits::clear_bit(self.castling, 1 + (orig_color << 1) as usize);
        }

        if piece::ROOK == orig_piece {
            match orig_square {
                0 => self.castling = bits::clear_bit(self.castling, 1),
                7 => self.castling = bits::clear_bit(self.castling, 0),
                56 => self.castling = bits::clear_bit(self.castling, 3),
                63 => self.castling = bits::clear_bit(self.castling, 2),
                _ => (),
            }
        }

        // flip to move
        self.to_move ^= 1;
    }

    fn unmake_move(&mut self) {
        let entry = self.move_stack.pop();
        let last_move = entry.mov;
        let unmake_info = entry.store;

        let orig_square = last_move.orig();
        let dest_square = last_move.dest();
        
        // let orig_color = last_move.color();

        // let (orig_piece, orig_color) = self.get_piece_and_color(orig_square);
        let (mut piece, color) = self.get_piece_and_color(dest_square);
        
        // Full move clock needs to be decremented after black unmakes
        // piece::WHITE == 0 and piece::BLACK == 1, so we use that to save an if :-)
        self.fullmoves -= color;
        
        // Half moves come from the unmake struct
        self.halfmoves = unmake_info.halfmoves();

        // En passant comes from the unmake struct
        self.en_passant = if unmake_info.ep_available() {
            Some(unmake_info.ep_square())
        } else {
            None
        };

        // Castling rights come from the unmake struct
        self.castling = unmake_info.castling();

        let captured_piece = unmake_info.captured_piece();
        let captured_color = unmake_info.captured_color();
        let was_capture = 2 <= captured_piece;

        //let unmake_info = UnmakeInfo::new(dest_piece, color, self.castling,
        //    ep_square, ep_allowed, self.halfmoves);

        // remove the destination piece
        self.remove_piece(piece, color, dest_square);

        // promotions change pieces
        if last_move.is_promotion() { 
            piece = piece::PAWN
        };

        if last_move.is_quiet() || last_move.is_double_pawn_push() {
            self.set_piece(piece, color, orig_square);
        } else if last_move.is_capture_en_passant() {
            self.set_piece(piece::PAWN, 1 ^ color, util::ep_capture_square(dest_square));
            self.set_piece(piece, color, orig_square);
        } else if was_capture {
            self.set_piece(captured_piece, captured_color, dest_square);
            self.set_piece(piece, color, orig_square);
        } else if last_move.is_king_castle() {
            self.set_piece(piece, color, orig_square);
            // move rook
            self.remove_piece(piece::ROOK, color, dest_square - 1);
            self.set_piece(piece::ROOK, color, dest_square + 1);
        } else if last_move.is_queen_castle() {
            self.set_piece(piece, color, orig_square);
            // move rook
            self.remove_piece(piece::ROOK, color, dest_square + 1);
            self.set_piece(piece::ROOK, color, dest_square - 2);
        }

        // flip to move
        self.to_move ^= 1;
    }

    pub fn input_move(&mut self, orig: Sq, dest: Sq, promote_to: Option<u32>) -> Result<bool, &'static str> {
        let (mut is_capture, mut is_promotion, mut is_special_0, mut is_special_1) = (false, false, false, false);
        let (piece, color) = self.get_piece_and_color(orig);
        if 0 == piece {
            return Err("No piece at given square")
        };
        
        // let (cap_piece, _) = self.get_piece_and_color(dest);
        // is_capture = 0 != cap_piece;

        // set special flags for double pawn push
        if piece == piece::PAWN && ((orig + 16 == dest) || (dest + 16 == orig)) {
            is_special_0 = true;
            is_special_1 = false;
        }

        // set flags for en passant capture
        if piece == piece::PAWN && Some(dest) == self.en_passant {
            is_special_0 = true;
            is_special_1 = false;
        }

        // set flags for promotion
        if let Some(promoted_piece) = promote_to {
            is_special_0 = false;
            is_special_1 = false;
            is_promotion = true;
            if (piece::BISHOP == promoted_piece) || (piece::QUEEN == promoted_piece) {
                is_special_0 = true;
            }
            if (piece::ROOK == promoted_piece) || (piece::QUEEN == promoted_piece) {
                is_special_1 = true;
            }
        }

        // set flags for castling
        if piece == piece::KING {
            if  2 == dest.wrapping_sub(orig) { // King castle
                is_special_0 = false;
                is_special_1 = true;
            } else if 2 == orig.wrapping_sub(dest) { // Queen castle
                is_special_0 = true;
                is_special_1 = true;
            }
        }
        
        let mov = Move::new(orig, dest, color, piece, 
                            Move::make_flags(is_capture, is_promotion, is_special_0, is_special_1));
        self.make_move(mov);
        Ok(true)
    }

    pub fn undo_move(&mut self) {
        self.unmake_move();
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
        assert_eq!(0, piece::WHITE as usize);
        assert_eq!(1, piece::BLACK as usize);
        assert_eq!(2, piece::PAWN as usize);
        assert_eq!(3, piece::KNIGHT as usize);
        assert_eq!(4, piece::BISHOP as usize);
        assert_eq!(5, piece::ROOK as usize);
        assert_eq!(6, piece::QUEEN as usize);
        assert_eq!(7, piece::KING as usize);
    }

    #[test]
    fn it_has_correct_color_enum_values() {
        assert_eq!(0, piece::WHITE as usize);
        assert_eq!(1, piece::BLACK as usize);
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
        assert_eq!(0xffff, b.bb[piece::WHITE as usize]);
        assert_eq!(0xffff << 6*8, b.bb[piece::BLACK as usize]);

        // pawn boards
        assert_eq!(0xff << 8, b.bb[piece::PAWN as usize] & b.bb[piece::WHITE as usize]);
        assert_eq!(0xff << 8, b.get_pieces(piece::PAWN, piece::WHITE));
        assert_eq!(0xff << 6*8, b.bb[piece::PAWN as usize] & b.bb[piece::BLACK as usize]);
        assert_eq!(0xff << 6*8, b.get_pieces(piece::PAWN, piece::BLACK));

        // rook boards
        assert_eq!(0x81, b.bb[piece::ROOK as usize] & b.bb[piece::WHITE as usize]);
        assert_eq!(0x81, b.get_pieces(piece::ROOK, piece::WHITE));
        assert_eq!(0x81 << 7*8, b.bb[piece::ROOK as usize] & b.bb[piece::BLACK as usize]);
        assert_eq!(0x81 << 7*8, b.get_pieces(piece::ROOK, piece::BLACK));
        

        // bishop boards
        assert_eq!(0x24, b.bb[piece::BISHOP as usize] & b.bb[piece::WHITE as usize]);
        assert_eq!(0x24, b.get_pieces(piece::BISHOP, piece::WHITE));
        assert_eq!(0x24 << 7*8, b.bb[piece::BISHOP as usize] & b.bb[piece::BLACK as usize]);
        assert_eq!(0x24 << 7*8, b.get_pieces(piece::BISHOP, piece::BLACK));

        // queen boards
        assert_eq!(0x8, b.bb[piece::QUEEN as usize] & b.bb[piece::WHITE as usize]);
        assert_eq!(0x8, b.get_pieces(piece::QUEEN, piece::WHITE));
        assert_eq!(0x8 << 7*8, b.bb[piece::QUEEN as usize] & b.bb[piece::BLACK as usize]);
        assert_eq!(0x8 << 7*8, b.get_pieces(piece::QUEEN, piece::BLACK));

        // king boards
        assert_eq!(0x10, b.bb[piece::KING as usize] & b.bb[piece::WHITE as usize]);
        assert_eq!(0x10, b.get_pieces(piece::KING, piece::WHITE));
        assert_eq!(0x10 << 7*8, b.bb[piece::KING as usize] & b.bb[piece::BLACK as usize]);
        assert_eq!(0x10 << 7*8, b.get_pieces(piece::KING, piece::BLACK));
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
            assert_eq!(util::square(x, 3), util::ep_capture_square(util::square(x, 2)));
            // black
            assert_eq!(util::square(x, 4), util::ep_capture_square(util::square(x, 5)));
        }
    }

    #[test]
    fn it_makes_moves() {
        if let Ok(mut board) = Board::from_fen(String::from("rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 1")) {
            assert_eq!(0, board.move_stack.len());
            board.input_move(squares::D7, squares::D6, None);
            assert_eq!(1, board.move_stack.len());
            assert_eq!(None, board.en_passant);
            let move_stack_entry = board.move_stack.peek();
            assert_eq!(move_stack_entry.mov.orig(), squares::D7);
            assert_eq!(move_stack_entry.mov.dest(), squares::D6);

            assert!(move_stack_entry.store.ep_available());
            assert_eq!(squares::D3, move_stack_entry.store.ep_square());
        }
    }

    #[test]
    fn it_unmakes_moves() {
        if let Ok(mut board) = Board::from_fen(String::from("rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 1")) {
            let before_unmake = board.to_fen();
            board.input_move(squares::D7, squares::D5, None);
            board.undo_move();
            let after_unmake = board.to_fen();
            assert_eq!(before_unmake, after_unmake);
        }

        // castling, king moves
        if let Ok(mut board) = Board::from_fen(String::from("8/8/8/8/8/8/8/R3K2R w KQkq - 0 1")) {
            let before_unmake = board.to_fen();
            board.input_move(squares::E1, squares::G1, None);
            board.undo_move();
            let after_unmake = board.to_fen();
            assert_eq!(before_unmake, after_unmake);
        }
        if let Ok(mut board) = Board::from_fen(String::from("8/8/8/8/8/8/8/R3K2R w KQkq - 0 1")) {
            let before_unmake = board.to_fen();
            board.input_move(squares::E1, squares::C1, None);
            board.undo_move();
            let after_unmake = board.to_fen();
            assert_eq!(before_unmake, after_unmake);
        }
        if let Ok(mut board) = Board::from_fen(String::from("r3k2r/8/8/8/8/8/8/8 b KQkq - 0 1")) {
            let before_unmake = board.to_fen();
            board.input_move(squares::E8, squares::G8, None);
            board.undo_move();
            let after_unmake = board.to_fen();
            assert_eq!(before_unmake, after_unmake);
        }
        if let Ok(mut board) = Board::from_fen(String::from("r3k2r/8/8/8/8/8/8/8 b KQkq - 0 1")) {
            let before_unmake = board.to_fen();
            board.input_move(squares::E8, squares::C8, None);
            board.undo_move();
            let after_unmake = board.to_fen();
            assert_eq!(before_unmake, after_unmake);
        }

        // castling, rook moves
        if let Ok(mut board) = Board::from_fen(String::from("8/8/8/8/8/8/8/R3K2R w KQkq - 0 1")) {
            let before_unmake = board.to_fen();
            board.input_move(squares::A1, squares::B1, None);
            board.undo_move();
            let after_unmake = board.to_fen();
            assert_eq!(before_unmake, after_unmake);
        }
        if let Ok(mut board) = Board::from_fen(String::from("8/8/8/8/8/8/8/R3K2R w KQkq - 0 1")) {
            let before_unmake = board.to_fen();
            board.input_move(squares::H1, squares::G1, None);
            board.undo_move();
            let after_unmake = board.to_fen();
            assert_eq!(before_unmake, after_unmake);
        }
        if let Ok(mut board) = Board::from_fen(String::from("r3k2r/8/8/8/8/8/8/8 b KQkq - 0 1")) {
            let before_unmake = board.to_fen();
            board.input_move(squares::A8, squares::B8, None);
            board.undo_move();
            let after_unmake = board.to_fen();
            assert_eq!(before_unmake, after_unmake);
        }
        if let Ok(mut board) = Board::from_fen(String::from("r3k2r/8/8/8/8/8/8/8 b KQkq - 0 1")) {
            let before_unmake = board.to_fen();
            board.input_move(squares::H8, squares::G8, None);
            board.undo_move();
            let after_unmake = board.to_fen();
            assert_eq!(before_unmake, after_unmake);
        }
    }
}