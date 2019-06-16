use std::fmt;

use common::*;
use bitboard::*;

use square::{self, Square, SquarePrimitives};
use piece::{self, Piece, PiecePrimitives};
use color::{self, Color};
use moves::{Move, UnmakeInfo};
use move_stack::{MoveStack, MoveStackEntry};

use std::str::FromStr;

/// Represents a chess position
/// 
/// Uses 16 bitboards ((2 colors + 6 pieces) * (unflipped + flipped)) plus an occupancy array
/// 
#[derive(Clone)]
pub struct Board {
    bb: [[Bitboard; 8]; 2],
    occupied: [Piece; 64],
    to_move: Color,
    castling: [u32; 2],
    en_passant: Option<[Square; 2]>,
    halfmoves: u32,
    fullmoves: u32,
    move_stack: MoveStack,
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Board {{ fen: {}, movestack: {}, bb[0]: {:#?} }}",
            self.to_fen(), self.move_stack, self.bb)
    }
}

impl Board {
    pub fn new() -> Board {
        let mut board = Board { 
            bb: [[0; 8]; 2],
            occupied: [0; 64],
            to_move: color::WHITE,
            castling: [0, 0],
            en_passant: None,
            halfmoves: 0,
            fullmoves: 1,
            move_stack: MoveStack::new(),
        };
        board
    }

    pub fn equals(&self, board: &Board) -> bool {
        for side in 0..2 {
            for bb in 0..8 {
                if self.bb[side][bb] != board.bb[side][bb] {
                    return false;
                }
            }
            if self.castling[side] != board.castling[side] {
                return false;
            }
        }
        for sq in 0..64 {
            if self.occupied[sq] != board.occupied[sq] {
                return false;
            }
        }
        if self.to_move != board.to_move {
            return false;
        }
        if self.en_passant != board.en_passant {
            return false;
        }
        if self.halfmoves != board.halfmoves {
            return false;
        }
        if self.fullmoves != board.fullmoves {
            return false;
        }     
        true
    }

    pub fn startpos() -> Board {
        let mut board = Board::new();
        // pawns
        for x in 0..8 {
            board.set_piece(piece::PAWN, color::WHITE, Square::from_coords(x, 1));
            board.set_piece(piece::PAWN, color::BLACK, Square::from_coords(x, 6));
        }

        // knights
        board.set_piece(piece::KNIGHT, color::WHITE, square::B1);
        board.set_piece(piece::KNIGHT, color::WHITE, square::G1);
        board.set_piece(piece::KNIGHT, color::BLACK, square::B8);
        board.set_piece(piece::KNIGHT, color::BLACK, square::G8);

        // bishops
        board.set_piece(piece::BISHOP, color::WHITE, square::C1);
        board.set_piece(piece::BISHOP, color::WHITE, square::F1);
        board.set_piece(piece::BISHOP, color::BLACK, square::C8);
        board.set_piece(piece::BISHOP, color::BLACK, square::F8);

        // rooks
        board.set_piece(piece::ROOK, color::WHITE, square::A1);
        board.set_piece(piece::ROOK, color::WHITE, square::H1);
        board.set_piece(piece::ROOK, color::BLACK, square::A8);
        board.set_piece(piece::ROOK, color::BLACK, square::H8);

        // queens
        board.set_piece(piece::QUEEN, color::WHITE, square::D1);
        board.set_piece(piece::QUEEN, color::BLACK, square::D8);

        // kings
        board.set_piece(piece::KING, color::WHITE, square::E1);
        board.set_piece(piece::KING, color::BLACK, square::E8);

        board.castling = [3, 3];
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
                            'P' => board.set_piece(piece::PAWN, color::WHITE, Square::from_coords(x, y)),
                            'N' => board.set_piece(piece::KNIGHT, color::WHITE, Square::from_coords(x, y)),
                            'B' => board.set_piece(piece::BISHOP, color::WHITE, Square::from_coords(x, y)),
                            'R' => board.set_piece(piece::ROOK, color::WHITE, Square::from_coords(x, y)),
                            'Q' => board.set_piece(piece::QUEEN, color::WHITE, Square::from_coords(x, y)),
                            'K' => board.set_piece(piece::KING, color::WHITE, Square::from_coords(x, y)),
                            'p' => board.set_piece(piece::PAWN, color::BLACK, Square::from_coords(x, y)),
                            'n' => board.set_piece(piece::KNIGHT, color::BLACK, Square::from_coords(x, y)),
                            'b' => board.set_piece(piece::BISHOP, color::BLACK, Square::from_coords(x, y)),
                            'r' => board.set_piece(piece::ROOK, color::BLACK, Square::from_coords(x, y)),
                            'q' => board.set_piece(piece::QUEEN, color::BLACK, Square::from_coords(x, y)),
                            'k' => board.set_piece(piece::KING, color::BLACK, Square::from_coords(x, y)),
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
                "w" => board.to_move = color::WHITE,
                "b" => board.to_move = color::BLACK,
                _ => return Err("Invalid ToMove char")
            }
        } else {
            return Err("Invalid FEN string, don't know who moves next")
        }

        // Castling rights
        if let Some(castling) = fen_iter.next() {
            for chr in castling.chars() {
                match chr {
                    '-' => board.castling = [0, 0],
                    'K' => board.castling[color::WHITE as usize] |= 0x1,
                    'Q' => board.castling[color::WHITE as usize] |= 0x2,
                    'k' => board.castling[color::BLACK as usize] |= 0x1,
                    'q' => board.castling[color::BLACK as usize] |= 0x2,
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
                //match SAN::square_str_to_index(en_passant) {
                match Square::from_san_string(en_passant) {
                    Ok(eps) => board.en_passant = Some([eps, eps.flipped()]),
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
            let mut emptycount: u8 = 0;
            for x in 0..8 {
                if 0 == self.occupied[Square::from_coords(x, y) as usize] {
                    emptycount += 1;
                } else {
                    if emptycount > 0 { 
                        fen_string.push_str(&emptycount.to_string());
                        emptycount = 0;
                    };
                    fen_string.push_str(self.occupied[Square::from_coords(x, y) as usize].to_san_string());
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
            color::WHITE => 'w',
            color::BLACK => 'b',
            _ => 'w',
        };
        fen_string.push(to_move);

        // Castling rights
        fen_string.push(' ');
        if self.castling == [0, 0] {
            fen_string.push('-');
        } else {
            if 0 != self.castling[color::WHITE as usize] & 0x1 { fen_string.push('K'); }
            if 0 != self.castling[color::WHITE as usize] & 0x2 { fen_string.push('Q'); }
            if 0 != self.castling[color::BLACK as usize] & 0x1 { fen_string.push('k'); }
            if 0 != self.castling[color::BLACK as usize] & 0x2 { fen_string.push('q'); }
        }

        // en passant
        fen_string.push(' ');
        if let Some(eps) = self.en_passant {
            let san = eps[0].to_san_string();
            fen_string.push_str(&san)
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
    pub fn bb(&self) -> &[[Bitboard; 8]; 2] {
        &self.bb
    }

    // don't actually return flipped boards for now
    #[inline]
    pub fn bb_own(&self, color: Color) -> Bitboard {
        // self.bb[self.to_move as usize][self.to_move as usize]
        self.bb[0][color as usize]
    }

    #[inline]
    pub fn bb_opponent(&self, color: Color) -> Bitboard {
        // self.bb[self.to_move as usize][1 ^ self.to_move as usize]
        self.bb[0][(1 ^ color) as usize]
    }

    #[inline]
    pub fn bb_pawns(&self, color: Color) -> Bitboard {
        // self.bb[self.to_move as usize][piece::PAWN as usize]
        self.bb[0][piece::PAWN as usize] & self.bb_own(color)
    }

    #[inline]
    pub fn bb_knights(&self, color: Color) -> Bitboard {
        // self.bb[self.to_move as usize][piece::PAWN as usize]
        self.bb[0][piece::KNIGHT as usize] & self.bb_own(color)
    }

    #[inline]
    pub fn bb_bishops(&self, color: Color) -> Bitboard {
        // self.bb[self.to_move as usize][piece::PAWN as usize]
        self.bb[0][piece::BISHOP as usize] & self.bb_own(color)
    }

    #[inline]
    pub fn bb_rooks(&self, color: Color) -> Bitboard {
        // self.bb[self.to_move as usize][piece::PAWN as usize]
        self.bb[0][piece::ROOK as usize] & self.bb_own(color)
    }

    #[inline]
    pub fn bb_queens(&self, color: Color) -> Bitboard {
        // self.bb[self.to_move as usize][piece::PAWN as usize]
        self.bb[0][piece::QUEEN as usize] & self.bb_own(color)
    }

    #[inline]
    pub fn bb_king(&self, color: Color) -> Bitboard {
        // self.bb[self.to_move as usize][piece::PAWN as usize]
        self.bb[0][piece::KING as usize] & self.bb_own(color)
    }

    #[inline]
    pub fn bb_empty(&self) -> Bitboard {
        !(self.bb_own(color::WHITE) | self.bb_opponent(color::WHITE))
    }

    #[inline]
    pub fn to_move(&self) -> Color {
        self.to_move
    }

    #[inline]
    pub fn castling(&self) -> [u32; 2] {
        self.castling
    }

    #[inline]
    pub fn en_passant(&self) -> Option<[Square; 2]> {
        self.en_passant
    }

    pub fn move_stack(&self) -> &MoveStack {
        &self.move_stack
    }

    #[inline]
    pub fn occupied(&self) -> &[Piece; 64] {
        &self.occupied
    }

    #[inline]
    fn get_piece_and_color(&self, square: Square) -> (Piece, Color) {
        let raw = self.occupied[square as usize];
        ((raw & 0x7), (raw >> 3) & 0x1)
    }

    // fn get_pieces(&self, piece: u32, color: u32) -> u64 {
    //     self.bb[piece as usize] & self.bb[color as usize]
    // }

    fn check_piece(&self, piece: Piece, color: Color, square: Square) -> bool {
        self.occupied[square as usize] == piece | color
    }

    #[inline]
    fn set_piece(&mut self, piece: Piece, color: Color, to: Square) {
        // update unflipped bb
        self.bb[0][color as usize].set_bit(to);
        self.bb[0][piece as usize].set_bit(to);
        
        // update flipped bb
        self.bb[1][color as usize].set_bit(to ^ 56);
        self.bb[1][piece as usize].set_bit(to ^ 56);
        
        // update occupancy array
        self.occupied[to as usize] = ((color & 0x1) << 3) | (piece & 0x7);
    }

    #[inline]
    fn remove_piece(&mut self, piece: Piece, color: Color, from: Square) {
        // update unflipped bb
        self.bb[0][color as usize].clear_bit(from);
        self.bb[0][piece as usize].clear_bit(from);
        
        // update flipped bb
        self.bb[1][color as usize].clear_bit(from ^ 56);
        self.bb[1][piece as usize].clear_bit(from ^ 56);
        
        // update occupancy array
        self.occupied[from as usize] = 0;
    }

    #[inline]
    fn replace_piece(&mut self, old_piece: Piece, old_color: Color, new_piece: Piece, new_color: Color, square: Square) {
        // remove from unflipped bb
        self.bb[0][old_color as usize].clear_bit(square);
        self.bb[0][old_piece as usize].clear_bit(square);
        
        // remove from flipped bb
        self.bb[1][old_color as usize].clear_bit(square ^ 56);
        self.bb[1][old_piece as usize].clear_bit(square ^ 56);

        self.set_piece(new_piece, new_color, square);
    }

    fn break_helper(&self) {
        println!("oh-oh")
    }

    pub fn make_move(&mut self, mov: Move) {
        // fail if no piece at origin square
        // debug_assert!(self.check_piece(mov.piece(), mov.color(), mov.from()));
        
        let orig_square = mov.orig();
        // let orig_piece = self.occupied[orig_square as usize] & 0x7;
        // let orig_color = (self.occupied[orig_square as usize] >> 3) & 0x1;
        let dest_square = mov.dest();
        // let mut dest_piece = self.occupied[dest_square as usize] & 0x7;
        // let mut dest_color = (self.occupied[dest_square as usize] >> 3) & 0x1;
        let (orig_piece, orig_color) = self.get_piece_and_color(orig_square);
        let (mut dest_piece, mut dest_color) = self.get_piece_and_color(dest_square);
        // let is_capture = 0 != dest_piece;
        let is_capture = mov.is_capture();

        let ep_allowed = self.en_passant != None;
        let ep_square = match self.en_passant {
            Some(sq) => sq[0],
            None => 64,
        };

        // debug_assert_eq!(orig_color, self.to_move);
        if orig_color != self.to_move {
            self.break_helper();
            eprintln!("{:?}", self);
            eprintln!("offending move: {:?}", mov);
            panic!("orig_color != self.to_move");
        }
        
        // reset en passant
        self.en_passant = None;

        // promotions change pieces
        if mov.is_promotion() { 
            let prom_piece = mov.special() as Piece + 3;
            self.remove_piece(orig_piece, orig_color, orig_square);
            if is_capture {
                self.replace_piece(dest_piece, dest_color, prom_piece, orig_color, dest_square);
            } else {
                self.set_piece(prom_piece, orig_color, dest_square);
            }
        } else if mov.is_quiet() {
            self.remove_piece(orig_piece, orig_color, orig_square);
            self.set_piece(orig_piece, orig_color, dest_square);
        } else if mov.is_capture_en_passant() {
            self.remove_piece(orig_piece, orig_color, orig_square);
            dest_piece = piece::PAWN;
            dest_color = 1 ^ orig_color;
            self.remove_piece(dest_piece, dest_color, square::ep_capture_square(dest_square));
            self.set_piece(orig_piece, orig_color, dest_square);
        } else if is_capture {
            self.remove_piece(orig_piece, orig_color, orig_square);
            self.replace_piece(dest_piece, dest_color, orig_piece, orig_color, dest_square);
        } else if mov.is_double_pawn_push() {
            self.remove_piece(orig_piece, orig_color, orig_square);
            let new_ep_square = (dest_square as i32 - [8i32, -8i32][orig_color as usize]) as Square;
            self.en_passant = Some([new_ep_square, new_ep_square.flipped()]);
            self.set_piece(orig_piece, orig_color, dest_square);
        } else if mov.is_king_castle() {
            self.remove_piece(orig_piece, orig_color, orig_square);
            self.set_piece(orig_piece, orig_color, dest_square);
            // move rook
            self.remove_piece(piece::ROOK, orig_color, dest_square + 1);
            self.set_piece(piece::ROOK, orig_color, dest_square - 1);
        } else if mov.is_queen_castle() {
            self.remove_piece(orig_piece, orig_color, orig_square);
            self.set_piece(orig_piece, orig_color, dest_square);
            // move rook
            self.remove_piece(piece::ROOK, orig_color, dest_square - 2);
            self.set_piece(piece::ROOK, orig_color, dest_square + 1);
        } else {
            panic!("shouldn't come here")
        }

        let unmake_info = UnmakeInfo::new(dest_piece, dest_color, self.castling,
            ep_square, ep_allowed, self.halfmoves);

        self.move_stack.push(MoveStackEntry::new(mov, unmake_info));

        // clear castling rights on king move
        if piece::KING == orig_piece {
            self.castling[self.to_move as usize].clear_bit(0);
            self.castling[self.to_move as usize].clear_bit(1);
        }

        if piece::ROOK == orig_piece {
            if orig_square == 0 {
                self.castling[color::WHITE as usize].clear_bit(1);
            } else if orig_square == 7 {
                self.castling[color::WHITE as usize].clear_bit(0);
            } else if orig_square == 56 {
                self.castling[color::BLACK as usize].clear_bit(1);
            } else if orig_square == 63 {
                self.castling[color::BLACK as usize].clear_bit(0);
            }
        }

        // Full move clock needs to be incremented after black moves
        // piece::WHITE == 0 and piece::BLACK == 1, so we use that to save an if :-)
        self.fullmoves += self.to_move as u32;
        
        // set half move clock
        if orig_piece == piece::PAWN || is_capture {
            self.halfmoves = 0; // reset half move clock on pawn moves and captures
        } else {
            self.halfmoves += 1;
        }

        // flip to move
        self.to_move ^= 1;
    }

    pub fn unmake_move(&mut self) {
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
        self.fullmoves -= 1 ^ (self.to_move as u32);
        
        // Half moves come from the unmake struct
        self.halfmoves = unmake_info.halfmoves();

        // En passant comes from the unmake struct
        self.en_passant = if unmake_info.ep_available() {
            let ep_square = unmake_info.ep_square();
            Some([ep_square, ep_square.flipped()])
        } else {
            None
        };

        // Castling rights come from the unmake struct
        self.castling[0] = unmake_info.castling()[0];
        self.castling[1] = unmake_info.castling()[1];

        let captured_piece = unmake_info.captured_piece();
        let captured_color = unmake_info.captured_color();
        let was_capture = 0 < captured_piece;

        
        // promotions change pieces
        if last_move.is_promotion() { 
            piece = piece::PAWN;
            if was_capture {
                // self.remove_piece(piece, color, dest_square);
                // self.set_piece(captured_piece, captured_color, dest_square);
                self.replace_piece(piece, color, captured_piece, captured_color, dest_square);
                self.set_piece(piece, color, orig_square);
            } else {
                self.remove_piece(piece, color, dest_square);
                self.set_piece(piece, color, orig_square);
            }
        } else if last_move.is_quiet() || last_move.is_double_pawn_push() {
            self.remove_piece(piece, color, dest_square);
            self.set_piece(piece, color, orig_square);
        } else if last_move.is_capture_en_passant() {
            self.remove_piece(piece, color, dest_square);
            self.set_piece(piece::PAWN, 1 ^ color, square::ep_capture_square(dest_square));
            self.set_piece(piece, color, orig_square);
        } else if was_capture {
            self.replace_piece(piece, color, captured_piece, captured_color, dest_square);
            // self.remove_piece(piece, color, dest_square);
            // self.set_piece(captured_piece, captured_color, dest_square);
            self.set_piece(piece, color, orig_square);
        } else if last_move.is_king_castle() {
            self.remove_piece(piece, color, dest_square);
            self.set_piece(piece, color, orig_square);
            // move rook
            self.remove_piece(piece::ROOK, color, dest_square - 1);
            self.set_piece(piece::ROOK, color, dest_square + 1);
        } else if last_move.is_queen_castle() {
            self.remove_piece(piece, color, dest_square);
            self.set_piece(piece, color, orig_square);
            // move rook
            self.remove_piece(piece::ROOK, color, dest_square + 1);
            self.set_piece(piece::ROOK, color, dest_square - 2);
        } else {
            panic!("shouldn't come here")
        }

        // flip to move
        self.to_move ^= 1;
    }

    pub fn input_move(&mut self, orig: Square, dest: Square, promote_to: Option<Piece>) -> Result<bool, &'static str> {
        let (mut is_capture, mut is_promotion, mut is_special_0, mut is_special_1) = (false, false, false, false);
        let (piece, color) = self.get_piece_and_color(orig);
        let (dest_piece, dest_color) = self.get_piece_and_color(dest);
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
        if piece == piece::PAWN && Some([dest, dest.flipped()]) == self.en_passant {
            is_capture = true;
            is_special_0 = true;
            is_special_1 = false;
        }

        // set flags for capture
        if dest_piece >= 2 {
            is_capture = true;
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
        
        let mov = Move::new(orig, dest, Move::make_flags(is_capture, is_promotion, is_special_0, is_special_1));
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
    use move_generator::MoveGenerator;
    use bitboard as bb;

    #[test]
    fn it_has_correct_piece_enum_values() {
        assert_eq!(0, color::WHITE as usize);
        assert_eq!(1, color::BLACK as usize);
        assert_eq!(2, piece::PAWN as usize);
        assert_eq!(3, piece::KNIGHT as usize);
        assert_eq!(4, piece::BISHOP as usize);
        assert_eq!(5, piece::ROOK as usize);
        assert_eq!(6, piece::QUEEN as usize);
        assert_eq!(7, piece::KING as usize);
    }

    #[test]
    fn it_has_correct_color_enum_values() {
        assert_eq!(0, color::WHITE as usize);
        assert_eq!(1, color::BLACK as usize);
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
                    assert!(0 != board.bb[0][color as usize] & bb::BB_SQUARES[square as usize]);
                    assert!(0 != board.bb[0][piece as usize] & bb::BB_SQUARES[square as usize]);
                    assert!(0 != board.bb[1][color as usize] & bb::BB_SQUARES[(square ^ 56) as usize]);
                    assert!(0 != board.bb[1][piece as usize] & bb::BB_SQUARES[(square ^ 56) as usize]);
                    assert_eq!(piece, board.occupied[square as usize] & 0x7);
                    assert_eq!(color, board.occupied[square as usize] >> 3);

                    //assert_eq!(board.bb[1][color as usize], bits::swap_bytes(board.bb[0][color as usize]));
                    //assert_eq!(board.bb[1][piece as usize], bits::swap_bytes(board.bb[0][piece as usize]));

                    assert_eq!(board.bb[1][color as usize], board.bb[0][color as usize].swap_bytes());
                    assert_eq!(board.bb[1][piece as usize], board.bb[0][piece as usize].swap_bytes());
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
                    assert!(0 != board.bb[0][color as usize] & bb::BB_SQUARES[square as usize]);
                    assert!(0 != board.bb[0][piece as usize] & bb::BB_SQUARES[square as usize]);
                    assert!(0 != board.bb[1][color as usize] & bb::BB_SQUARES[(square ^ 56) as usize]);
                    assert!(0 != board.bb[1][piece as usize] & bb::BB_SQUARES[(square ^ 56) as usize]);
                    assert_eq!(piece, board.occupied[square as usize] & 0x7);
                    assert_eq!(color, board.occupied[square as usize] >> 3);

                    assert_eq!(board.bb[1][color as usize], board.bb[0][color as usize].swap_bytes());
                    assert_eq!(board.bb[1][piece as usize], board.bb[0][piece as usize].swap_bytes());
                }
            }
        }
    }
    
    #[test]
    fn it_sets_correct_startpos() {
        // let b = Board::startpos(); // Board { bb: [0; 8] };
        
        // // color boards
        // assert_eq!(0xffff, b.bb[piece::WHITE as usize]);
        // assert_eq!(0xffff << 6*8, b.bb[piece::BLACK as usize]);

        // // pawn boards
        // assert_eq!(0xff << 8, b.bb[piece::PAWN as usize] & b.bb[piece::WHITE as usize]);
        // assert_eq!(0xff << 8, b.get_pieces(piece::PAWN, piece::WHITE));
        // assert_eq!(0xff << 6*8, b.bb[piece::PAWN as usize] & b.bb[piece::BLACK as usize]);
        // assert_eq!(0xff << 6*8, b.get_pieces(piece::PAWN, piece::BLACK));

        // // rook boards
        // assert_eq!(0x81, b.bb[piece::ROOK as usize] & b.bb[piece::WHITE as usize]);
        // assert_eq!(0x81, b.get_pieces(piece::ROOK, piece::WHITE));
        // assert_eq!(0x81 << 7*8, b.bb[piece::ROOK as usize] & b.bb[piece::BLACK as usize]);
        // assert_eq!(0x81 << 7*8, b.get_pieces(piece::ROOK, piece::BLACK));
        

        // // bishop boards
        // assert_eq!(0x24, b.bb[piece::BISHOP as usize] & b.bb[piece::WHITE as usize]);
        // assert_eq!(0x24, b.get_pieces(piece::BISHOP, piece::WHITE));
        // assert_eq!(0x24 << 7*8, b.bb[piece::BISHOP as usize] & b.bb[piece::BLACK as usize]);
        // assert_eq!(0x24 << 7*8, b.get_pieces(piece::BISHOP, piece::BLACK));

        // // queen boards
        // assert_eq!(0x8, b.bb[piece::QUEEN as usize] & b.bb[piece::WHITE as usize]);
        // assert_eq!(0x8, b.get_pieces(piece::QUEEN, piece::WHITE));
        // assert_eq!(0x8 << 7*8, b.bb[piece::QUEEN as usize] & b.bb[piece::BLACK as usize]);
        // assert_eq!(0x8 << 7*8, b.get_pieces(piece::QUEEN, piece::BLACK));

        // // king boards
        // assert_eq!(0x10, b.bb[piece::KING as usize] & b.bb[piece::WHITE as usize]);
        // assert_eq!(0x10, b.get_pieces(piece::KING, piece::WHITE));
        // assert_eq!(0x10 << 7*8, b.bb[piece::KING as usize] & b.bb[piece::BLACK as usize]);
        // assert_eq!(0x10 << 7*8, b.get_pieces(piece::KING, piece::BLACK));
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
            assert_eq!(Square::from_coords(x, 3), square::ep_capture_square(Square::from_coords(x, 2)));
            // black
            assert_eq!(Square::from_coords(x, 4), square::ep_capture_square(Square::from_coords(x, 5)));
        }
    }

    #[test]
    fn it_makes_moves() {
        if let Ok(mut board) = Board::from_fen(String::from("rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 1")) {
            assert_eq!(0, board.move_stack.len());
            board.input_move(square::D7, square::D6, None);
            assert_eq!(1, board.move_stack.len());
            assert_eq!(None, board.en_passant);
            let move_stack_entry = board.move_stack.peek();
            assert_eq!(move_stack_entry.mov.orig(), square::D7);
            assert_eq!(move_stack_entry.mov.dest(), square::D6);

            assert!(move_stack_entry.store.ep_available());
            assert_eq!(square::D3, move_stack_entry.store.ep_square());
        }

        let mut board = Board::from_fen(String::from("8/3p4/8/4P/8/8/8/8 b - - 0 1")).unwrap();
        board.input_move(square::D7, square::D5, None);
        board.input_move(square::E5, square::D6, None);
        assert_eq!(0, board.occupied[square::D5 as usize]);
    }

    #[test]
    fn it_unmakes_moves() {
        if let Ok(mut board) = Board::from_fen(String::from("rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 1")) {
            let before_unmake = board.to_fen();
            board.input_move(square::D7, square::D5, None);
            board.undo_move();
            let after_unmake = board.to_fen();
            assert_eq!(before_unmake, after_unmake);
        }

        // castling, king moves
        if let Ok(mut board) = Board::from_fen(String::from("8/8/8/8/8/8/8/R3K2R w KQkq - 0 1")) {
            let before_unmake = board.to_fen();
            board.input_move(square::E1, square::G1, None);
            board.undo_move();
            let after_unmake = board.to_fen();
            assert_eq!(before_unmake, after_unmake);
        }
        if let Ok(mut board) = Board::from_fen(String::from("8/8/8/8/8/8/8/R3K2R w KQkq - 0 1")) {
            let before_unmake = board.to_fen();
            board.input_move(square::E1, square::C1, None);
            board.undo_move();
            let after_unmake = board.to_fen();
            assert_eq!(before_unmake, after_unmake);
        }
        if let Ok(mut board) = Board::from_fen(String::from("r3k2r/8/8/8/8/8/8/8 b KQkq - 0 1")) {
            let before_unmake = board.to_fen();
            board.input_move(square::E8, square::G8, None);
            board.undo_move();
            let after_unmake = board.to_fen();
            assert_eq!(before_unmake, after_unmake);
        }
        if let Ok(mut board) = Board::from_fen(String::from("r3k2r/8/8/8/8/8/8/8 b KQkq - 0 1")) {
            let before_unmake = board.to_fen();
            board.input_move(square::E8, square::C8, None);
            board.undo_move();
            let after_unmake = board.to_fen();
            assert_eq!(before_unmake, after_unmake);
        }

        // castling, rook moves
        if let Ok(mut board) = Board::from_fen(String::from("8/8/8/8/8/8/8/R3K2R w KQkq - 0 1")) {
            let before_unmake = board.to_fen();
            board.input_move(square::A1, square::B1, None);
            board.undo_move();
            let after_unmake = board.to_fen();
            assert_eq!(before_unmake, after_unmake);
        }
        if let Ok(mut board) = Board::from_fen(String::from("8/8/8/8/8/8/8/R3K2R w KQkq - 0 1")) {
            let before_unmake = board.to_fen();
            board.input_move(square::H1, square::G1, None);
            board.undo_move();
            let after_unmake = board.to_fen();
            assert_eq!(before_unmake, after_unmake);
        }
        if let Ok(mut board) = Board::from_fen(String::from("r3k2r/8/8/8/8/8/8/8 b KQkq - 0 1")) {
            let before_unmake = board.to_fen();
            board.input_move(square::A8, square::B8, None);
            board.undo_move();
            let after_unmake = board.to_fen();
            assert_eq!(before_unmake, after_unmake);
        }
        if let Ok(mut board) = Board::from_fen(String::from("r3k2r/8/8/8/8/8/8/8 b KQkq - 0 1")) {
            let before_unmake = board.to_fen();
            board.input_move(square::H8, square::G8, None);
            board.undo_move();
            let after_unmake = board.to_fen();
            assert_eq!(before_unmake, after_unmake);
        }

        if let Ok(mut board) = Board::from_fen(String::from("8/3p4/8/4P/8/8/8/8 b - - 0 1")) {
            let before_unmake = board.to_fen();
            board.input_move(square::D7, square::D5, None);
            board.input_move(square::E5, square::D6, None);
            board.undo_move();
            board.undo_move();
            let after_unmake = board.to_fen();
            assert_eq!(before_unmake, after_unmake);
        }
    }

    #[test]
    fn it_unwinds_its_move_stack() {
        {
            let fen = String::from("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
            let board_orig = Board::from_fen(fen.clone()).unwrap();
            let mut board = Board::from_fen(fen.clone()).unwrap();
            MoveGenerator::perft(&mut board, 4);
            assert_eq!(fen, board.to_fen());
            assert!(board_orig.equals(&board));
        }
        {
            let mut board_orig = Board::startpos();
            let mut board = board_orig.clone();
            MoveGenerator::perft(&mut board, 4);
            assert!(board_orig.equals(&board));
        }
    }
}