use crate::color::{self, Color};
use crate::piece::{self, Piece, PiecePrimitives};
use crate::square::{Square, SquarePrimitives};

use crate::board::Board;
use crate::color::*;
use crate::common::BitTwiddling;
use crate::piece::*;
use crate::position::Position;

//use regex::Regex;
// use pest::Parser;

use std::error::Error;
use std::fmt;
use std::str::FromStr;
use std::string::String;

use nom::{
    IResult,
    error::{ErrorKind, ParseError, VerboseError, convert_error},
};

use nom::{
    branch::alt,
    bytes::complete::{
        is_a,
        tag,
        take, take_while, take_while1, take_while_m_n,
    },
    character::complete::{
        digit0, digit1, multispace0, multispace1, one_of,
    },
    combinator::{
        map, map_res, opt, peek, verify,
    },
    multi::count,
    sequence::{
        preceded, terminated, tuple,
    },
};

use std::io::{self, Write};

//type IResult<I, O, E = (I, ErrorKind)> = Result<(I, O), Err<E>>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Needed {
  Unknown,
  Size(u32)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Err<E> {
    //Incomplete(Needed),
    Error(E),
    Failure(E)
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ParsedFen {
    pub placement: Vec<Piece>,
    pub to_move: Color,
    pub castling: [u32; 2],
    pub ep_target: Option<Square>,
    pub halfmoves: u32,
    pub fullmoves: u32,
}

fn fen(input: &str) -> IResult <&str, ParsedFen> {
    let result = tuple((
        placement,
        multispace1,
        to_move,
        multispace1,
        castling,
        multispace1,
        ep_target,
        multispace1,
        halfmoves,
        multispace1,
        fullmoves,
    ))(input);
    
    match result {
        Ok(tup) => {
            let (
                rest, (
                    placement,
                    _, to_move,
                    _, castling,
                    _, ep_target,
                    _, halfmoves,
                    _, fullmoves
                )
            ) = tup;
            Ok((rest, ParsedFen {
                placement,
                to_move, 
                castling,
                ep_target,
                halfmoves,
                fullmoves
            }))
        },
        Err(e) => Err(e),
    }
}

fn placement(input: &str) -> IResult <&str, Vec<u8>> {
    let res = count(rank, 8)(input);
    match res {
        Ok((rest, vect)) => { 
            Ok((
                rest, 
                vect.into_iter()
                    .rev()
                    .flatten()
                    .collect::<Vec<u8>>()
            )) 
        },
        Err(e) => Err(e),
    }
}

fn rank_str_to_piece_vec(line: &str) -> Result<Vec<Piece>, &'static str> {
    let mut res: Vec<u8> = vec![piece::EMPTY; 8];
    let mut cursor = 0_usize;
    for c in line.chars() {
        if cursor > 8 {
            break;
        }
        if c.is_digit(10) {
            let skip = c.to_digit(10).unwrap() as usize;
            cursor += skip;
        } else {
            let (piece_code, color) = match c {
                'P' => (piece::PAWN, color::WHITE),
                'N' => (piece::KNIGHT, color::WHITE),
                'B' => (piece::BISHOP, color::WHITE),
                'R' => (piece::ROOK, color::WHITE),
                'Q' => (piece::QUEEN, color::WHITE),
                'K' => (piece::KING, color::WHITE),
                'p' => (piece::PAWN, color::BLACK),
                'n' => (piece::KNIGHT, color::BLACK),
                'b' => (piece::BISHOP, color::BLACK),
                'r' => (piece::ROOK, color::BLACK),
                'q' => (piece::QUEEN, color::BLACK),
                'k' => (piece::KING, color::BLACK),
                _ => unreachable!("Something has gone very wrong with the FEN parser"),
            };
            res[cursor] = Piece::new(piece_code, color);
            cursor += 1;
        }
    }
    Ok(res)
}

fn rank(input: &str) -> IResult<&str, Vec<u8>> {
    map_res(
        terminated(
            is_a("12345678KkQqRrBbNnPp"),
            opt(tag("/")),
        ),
        rank_str_to_piece_vec,
    )(input)
}

fn to_move(input: &str) -> IResult<&str, Color> {
    let (rest, c) = one_of("bw")(input)?;
    match c {
        'b' => Ok((rest, color::BLACK)),
        'w' => Ok((rest, color::WHITE)),
        _ => unreachable!("Internal parser error: to_move")
    }
}

fn castling(input: &str) -> IResult<&str, [u32; 2]> {
    let (rest, c) = alt((
        tag("-"),
        is_a("KQkq")
    ))(input)?;

    let mut cast = [0_u32; 2];

    for chr in c.chars() {
        match chr {
            '-' => cast = [0, 0],
            'K' => cast[color::WHITE as usize] |= 0x1,
            'Q' => cast[color::WHITE as usize] |= 0x2,
            'k' => cast[color::BLACK as usize] |= 0x1,
            'q' => cast[color::BLACK as usize] |= 0x2,
            _ => unreachable!("Internal parser error: castling")
            // _ => return Err(Error),
        }
    }
    Ok((rest, cast))
}

fn ep_square(input: &str) -> IResult<&str, &str> {
    preceded(
        peek(
            tuple((
                one_of("abcdefgh"),
                one_of("36"))
            )
        ),
        take(2_usize)
    )(input)
}

fn ep_target(input: &str) -> IResult<&str, Option<Square>> {
    let (rest, ep) = alt((
        tag("-"),
        ep_square
    ))(input)?;
    
    if ep == "-" {
        Ok((rest, None))
    } else {
        let sq_res = Square::from_san_string(ep);
        match sq_res {
            Ok(sq) => Ok((rest, Some(sq))),
            Err(e) => unreachable!("Error parsing EP square"),
        }
    }
}

fn halfmoves(input: &str) -> IResult<&str, u32> {
    map_res(
        digit1, 
        u32::from_str
    )(input)
}

fn fullmoves(input: &str) -> IResult<&str, u32> {
    preceded(
        peek(one_of("123456789")),
        halfmoves
    )(input)
}

pub fn parse(input: &str) -> IResult<&str, ParsedFen> {
    fen(input)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FenParseError {
    Empty,
    Invalid,
    InvalidPlacement,
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

pub trait FenInterface<T=Self> { 
    type Err;
    fn from_fen_str(s: &str) -> Result<T, Self::Err>;
    fn to_fen_string(&self) -> String;
}

impl FenInterface for Position {
    type Err = FenParseError;

    fn from_fen_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(FenParseError::Empty);
        }

        // let mut board = Self::new();
        // let mut position = Self::new();
        
        let parser = fen(s);
        let result = match parser {
            Ok((_, res)) => res,
            Err(e) => return Err(FenParseError::Invalid),
        };

        // position
        let mut position = Self {
            bb: [[0_u64; 8]; 2],
            occupied: [piece::EMPTY; 64],
            to_move: result.to_move, //color::WHITE,
            castling: result.castling, //[0_u32; 2],
            en_passant: match result.ep_target {
                Some(eps) => Some([eps, eps.flipped()]),
                None => None,
            },
            halfmoves: result.halfmoves,
            fullmoves: result.fullmoves,
        };

        for i in 0..64 {
            if result.placement[i] != piece::EMPTY {
                let piece = result.placement[i] as Piece;
                position.set_piece(piece.code(), piece.color(), i as u16);
            }
        }

        Ok(position)

        // board.set_position(&position);

        // if let Some(move_token) = fen_iter.next() {
        //     if move_token == "moves" {
        //         for mov in fen_iter {
        //             match board.input_san_move(mov) {
        //                 Ok(_) => continue,
        //                 Err(_) => return Err(FenParseError::InvalidMove),
        //             }
        //         }
        //     }
        // }
        // Ok(board)
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

impl FenInterface for Board {
    type Err = FenParseError;

    // fn from_fen_str(s: &str) -> Result<Board, Self::Err> {
    //     if s.is_empty() {
    //         return Err(FenParseError::Empty);
    //     }

    //     let mut board = Self::new();
    //     let mut position = Position::new();
    //     let mut fen_iter = s.split_whitespace();

    //     // position
    //     if let Some(piece_list) = fen_iter.next() {
    //         let ranks: Vec<&str> = piece_list.split('/').collect();
    //         if ranks.len() != 8 {
    //             return Err(FenParseError::InvalidPlacement);
    //         }

    //         for (rank, rank_string) in ranks.iter().rev().enumerate() {
    //             let mut file = 0;
    //             for chr in rank_string.chars() {
    //                 if chr.is_digit(10) {
    //                     file += chr.to_digit(10).unwrap();
    //                 } else {
    //                     let (piece_code, color) = match chr {
    //                         'P' => (PAWN, WHITE),
    //                         'N' => (KNIGHT, WHITE),
    //                         'B' => (BISHOP, WHITE),
    //                         'R' => (ROOK, WHITE),
    //                         'Q' => (QUEEN, WHITE),
    //                         'K' => (KING, WHITE),
    //                         'p' => (PAWN, BLACK),
    //                         'n' => (KNIGHT, BLACK),
    //                         'b' => (BISHOP, BLACK),
    //                         'r' => (ROOK, BLACK),
    //                         'q' => (QUEEN, BLACK),
    //                         'k' => (KING, BLACK),
    //                         _ => return Err(FenParseError::InvalidPlacement),
    //                     };
    //                     position.set_piece(
    //                         piece_code,
    //                         color,
    //                         Square::from_coords(file, rank as u32),
    //                     );
    //                     file += 1;
    //                 }
    //             }
    //         }
    //     } else {
    //         return Err(FenParseError::InvalidPlacement);
    //     }

    //     // to move
    //     if let Some(to_move) = fen_iter.next() {
    //         match to_move {
    //             "w" => position.to_move = WHITE,
    //             "b" => position.to_move = BLACK,
    //             _ => return Err(FenParseError::InvalidToMove),
    //         }
    //     } else {
    //         return Err(FenParseError::InvalidToMove);
    //     }

    //     // Castling rights
    //     if let Some(castling) = fen_iter.next() {
    //         for chr in castling.chars() {
    //             match chr {
    //                 '-' => position.castling = [0, 0],
    //                 'K' => position.castling[WHITE as usize] |= 0x1,
    //                 'Q' => position.castling[WHITE as usize] |= 0x2,
    //                 'k' => position.castling[BLACK as usize] |= 0x1,
    //                 'q' => position.castling[BLACK as usize] |= 0x2,
    //                 _ => return Err(FenParseError::InvalidCastling),
    //             }
    //         }
    //     } else {
    //         return Err(FenParseError::InvalidCastling);
    //     }

    //     // en passant
    //     if let Some(en_passant) = fen_iter.next() {
    //         if en_passant == "-" {
    //             position.en_passant = None;
    //         } else {
    //             //match SAN::square_str_to_index(en_passant) {
    //             match Square::from_san_string(en_passant) {
    //                 Ok(eps) => position.en_passant = Some([eps, eps.flipped()]),
    //                 Err(_) => return Err(FenParseError::InvalidEnPassant),
    //             }
    //         }
    //     } else {
    //         return Err(FenParseError::InvalidEnPassant);
    //     }

    //     // Halfmoves
    //     if let Some(halfmoves) = fen_iter.next() {
    //         match u32::from_str(halfmoves) {
    //             Ok(val) => position.halfmoves = val,
    //             Err(_) => return Err(FenParseError::InvalidHalfmoves),
    //         }
    //     } else {
    //         return Err(FenParseError::InvalidHalfmoves);
    //     }

    //     // Fullmoves
    //     if let Some(fullmoves) = fen_iter.next() {
    //         match u32::from_str(fullmoves) {
    //             Ok(val) => position.fullmoves = val,
    //             Err(_) => return Err(FenParseError::InvalidFullmoves),
    //         }
    //     } else {
    //         return Err(FenParseError::InvalidFullmoves);
    //     }

    //     board.set_position(&position);

    //     if let Some(move_token) = fen_iter.next() {
    //         if move_token == "moves" {
    //             for mov in fen_iter {
    //                 match board.input_san_move(mov) {
    //                     Ok(_) => continue,
    //                     Err(_) => return Err(FenParseError::InvalidMove),
    //                 }
    //             }
    //         }
    //     }
    //     Ok(board)
    // }

    fn from_fen_str(s: &str) -> Result<Board, Self::Err> {
        if s.is_empty() {
            return Err(FenParseError::Empty);
        }

        let mut board = Self::new();
        match Position::from_fen_str(s) {
            Ok(p) => {
                board.set_position(&p);
                Ok(board)
            },
            Err(e) => Err(FenParseError::Invalid)
        }
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