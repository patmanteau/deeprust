use crate::board::Board;
use crate::color::*;
use crate::common::BitTwiddling;
use crate::piece::*;
use crate::position::Position;
use crate::square::{Square, SquarePrimitives};

use regex::Regex;
use std::error::Error;
use std::fmt;
use std::str::FromStr;
use std::string::String;

// pub struct FenString {
//     s: String,
// }

const VALID_FEN_RE: &str = r"/^\s*([rnbqkpRNBQKP1-8]+/){7}([rnbqkpRNBQKP1-8]+)\s[bw]\s(-|K?Q?k?q?)\s(-|[a-h][36])\s(0|[1-9][0-9]*)\s([1-9][0-9]*)/";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FenParseError {
    Empty,
    Invalid,
    InvalidPosition,
    InvalidToMove,
    InvalidCastling,
    InvalidEnPassant,
    InvalidHalfmoves,
    InvalidFullmoves,
    InvalidMove,
}

impl fmt::Display for FenParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Error for FenParseError {}

pub trait BoardFen {
    type Err;
    fn validate_fen(s: &str) -> bool;
    fn from_fen_str(s: &str) -> Result<Board, Self::Err>;
    fn to_fen_string(&self) -> String;
}

impl BoardFen for Board {
    type Err = FenParseError;
    fn validate_fen(s: &str) -> bool {
        // Regex::new(VALID_FEN_RE)
        //     .unwrap()
        //     .is_match(s)
        true
    }

    fn from_fen_str(s: &str) -> Result<Board, Self::Err> {
        if s.is_empty() {
            return Err(FenParseError::Empty);
        }

        if !Self::validate_fen(s) {
            return Err(FenParseError::Invalid);
        }

        let mut board = Self::new();
        let mut position = Position::new();
        let mut fen_iter = s.split_whitespace();

        // position
        if let Some(piece_list) = fen_iter.next() {
            let ranks: Vec<&str> = piece_list.split('/').collect();
            if ranks.len() != 8 {
                return Err(FenParseError::InvalidPosition);
            }

            for (rank, rank_string) in ranks.iter().rev().enumerate() {
                let mut file = 0;
                for chr in rank_string.chars() {
                    if chr.is_digit(10) {
                        file += chr.to_digit(10).unwrap();
                    } else {
                        let (piece_code, color) = match chr {
                            'P' => (PAWN, WHITE),
                            'N' => (KNIGHT, WHITE),
                            'B' => (BISHOP, WHITE),
                            'R' => (ROOK, WHITE),
                            'Q' => (QUEEN, WHITE),
                            'K' => (KING, WHITE),
                            'p' => (PAWN, BLACK),
                            'n' => (KNIGHT, BLACK),
                            'b' => (BISHOP, BLACK),
                            'r' => (ROOK, BLACK),
                            'q' => (QUEEN, BLACK),
                            'k' => (KING, BLACK),
                            _ => return Err(FenParseError::InvalidPosition),
                        };
                        position.set_piece(
                            piece_code,
                            color,
                            Square::from_coords(file, rank as u32),
                        );
                        file += 1;
                    }
                }
            }
        } else {
            return Err(FenParseError::InvalidPosition);
        }

        // to move
        if let Some(to_move) = fen_iter.next() {
            match to_move {
                "w" => position.to_move = WHITE,
                "b" => position.to_move = BLACK,
                _ => return Err(FenParseError::InvalidToMove),
            }
        } else {
            return Err(FenParseError::InvalidToMove);
        }

        // Castling rights
        if let Some(castling) = fen_iter.next() {
            for chr in castling.chars() {
                match chr {
                    '-' => position.castling = [0, 0],
                    'K' => position.castling[WHITE as usize] |= 0x1,
                    'Q' => position.castling[WHITE as usize] |= 0x2,
                    'k' => position.castling[BLACK as usize] |= 0x1,
                    'q' => position.castling[BLACK as usize] |= 0x2,
                    _ => return Err(FenParseError::InvalidCastling),
                }
            }
        } else {
            return Err(FenParseError::InvalidCastling);
        }

        // en passant
        if let Some(en_passant) = fen_iter.next() {
            if en_passant == "-" {
                position.en_passant = None;
            } else {
                //match SAN::square_str_to_index(en_passant) {
                match Square::from_san_string(en_passant) {
                    Ok(eps) => position.en_passant = Some([eps, eps.flipped()]),
                    Err(_) => return Err(FenParseError::InvalidEnPassant),
                }
            }
        } else {
            return Err(FenParseError::InvalidEnPassant);
        }

        // Halfmoves
        if let Some(halfmoves) = fen_iter.next() {
            match u32::from_str(halfmoves) {
                Ok(val) => position.halfmoves = val,
                Err(_) => return Err(FenParseError::InvalidHalfmoves),
            }
        } else {
            return Err(FenParseError::InvalidHalfmoves);
        }

        // Fullmoves
        if let Some(fullmoves) = fen_iter.next() {
            match u32::from_str(fullmoves) {
                Ok(val) => position.fullmoves = val,
                Err(_) => return Err(FenParseError::InvalidFullmoves),
            }
        } else {
            return Err(FenParseError::InvalidFullmoves);
        }

        board.set_position(&position);

        if let Some(move_token) = fen_iter.next() {
            if move_token == "moves" {
                for mov in fen_iter {
                    match board.input_san_move(mov) {
                        Ok(_) => continue,
                        Err(err) => return Err(FenParseError::InvalidMove),
                    }
                }
            }
        }
        Ok(board)
    }

    fn to_fen_string(&self) -> String {
        let mut fen_string = String::new();

        // Position
        for y in (0..8).rev() {
            let mut emptycount: u8 = 0;
            for x in 0..8 {
                if 0 == self.occupied()[Square::from_coords(x, y) as usize] {
                    emptycount += 1;
                } else {
                    if emptycount > 0 {
                        fen_string.push_str(&emptycount.to_string());
                        emptycount = 0;
                    };
                    fen_string.push_str(
                        self.occupied()[Square::from_coords(x, y) as usize].to_san_string(),
                    );
                }
            }
            if emptycount > 0 {
                fen_string.push_str(&emptycount.to_string());
                // emptycount = 0;
            };
            if y > 0 {
                fen_string.push('/');
            }
        }

        // To move
        fen_string.push(' ');
        let to_move = match self.to_move() {
            WHITE => 'w',
            BLACK => 'b',
            _ => 'w',
        };
        fen_string.push(to_move);

        // Castling rights
        fen_string.push(' ');
        if self.castling() == [0, 0] {
            fen_string.push('-');
        } else {
            if 0 != self.castling()[WHITE as usize].extract_bits(0, 1) {
                fen_string.push('K');
            }
            if 0 != self.castling()[WHITE as usize].extract_bits(1, 1) {
                fen_string.push('Q');
            }
            if 0 != self.castling()[BLACK as usize].extract_bits(0, 1) {
                fen_string.push('k');
            }
            if 0 != self.castling()[BLACK as usize].extract_bits(1, 1) {
                fen_string.push('q');
            }
        }

        // en passant
        fen_string.push(' ');
        if let Some(eps) = self.en_passant() {
            let san = eps[0].to_san_string();
            fen_string.push_str(&san)
        } else {
            fen_string.push('-')
        }

        // Halfmoves
        fen_string.push(' ');
        fen_string.push_str(&self.halfmoves().to_string());

        // Fullmoves
        fen_string.push(' ');
        fen_string.push_str(&self.fullmoves().to_string());

        fen_string
    }
}
