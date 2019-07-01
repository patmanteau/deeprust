use crate::color::{self, Color, ColorPrimitives};
use crate::piece::{self, Piece, PiecePrimitives};
use crate::square::{Square, SquarePrimitives};

use crate::board::Board;
use crate::color::*;
use crate::common::BitTwiddling;
use crate::position::Position;

//use regex::Regex;
// use pest::Parser;

use std::fmt;
use std::str::FromStr;
use std::string::String;

use nom::IResult;

use nom::{
    branch::alt,
    bytes::complete::{is_a, tag, take},
    character::complete::{digit1, multispace1, one_of},
    combinator::{map, map_res, peek},
    multi::{many1, separated_nonempty_list},
    sequence::{preceded, tuple},
};


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ParsedFen {
    pub placement: Vec<Piece>,
    pub to_move: Color,
    pub castling: [u32; 2],
    pub ep_target: Option<Square>,
    pub halfmoves: u32,
    pub fullmoves: u32,
}

pub fn fen(input: &str) -> IResult<&str, ParsedFen> {
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
                rest,
                (placement, _, to_move, _, castling, _, ep_target, _, halfmoves, _, fullmoves),
            ) = tup;
            if placement.len() != 64 {
                panic!(
                    "Invalid FEN string, found {} squares when there should be 64",
                    placement.len()
                );
            }
            Ok((
                rest,
                ParsedFen {
                    placement,
                    to_move,
                    castling,
                    ep_target,
                    halfmoves,
                    fullmoves,
                },
            ))
        }
        Err(e) => Err(e),
    }
}

fn placement(input: &str) -> IResult<&str, Vec<u8>> {
    map(separated_nonempty_list(tag("/"), rank), |l| {
        l.into_iter().rev().flatten().collect::<Vec<u8>>()
    })(input)
}

fn rank(input: &str) -> IResult<&str, Vec<u8>> {
    map(many1(alt((empty_square, occupied_square))), |l| {
        l.into_iter().flatten().collect()
    })(input)
}

fn occupied_square(input: &str) -> IResult<&str, Vec<Piece>> {
    many1(map(one_of("KkQqRrBbNnPp"), PiecePrimitives::from_char))(input)
}

fn empty_square(input: &str) -> IResult<&str, Vec<Piece>> {
    map(one_of("12345678"), |n| {
        vec![Piece::empty(); n.to_digit(10).unwrap() as usize]
    })(input)
}

fn to_move(input: &str) -> IResult<&str, Color> {
    map(one_of("bw"), ColorPrimitives::from_char)(input)
}

fn castling(input: &str) -> IResult<&str, [u32; 2]> {
    let (rest, c) = alt((tag("-"), is_a("KQkq")))(input)?;

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
        peek(tuple((one_of("abcdefgh"), one_of("36")))),
        take(2_usize),
    )(input)
}

fn ep_target(input: &str) -> IResult<&str, Option<Square>> {
    let (rest, ep) = alt((tag("-"), ep_square))(input)?;

    if ep == "-" {
        Ok((rest, None))
    } else {
        match Square::from_san_string(ep) {
            Ok(sq) => Ok((rest, Some(sq))),
            Err(_e) => unreachable!("Error parsing EP square"),
        }
    }
}

fn halfmoves(input: &str) -> IResult<&str, u32> {
    map_res(digit1, u32::from_str)(input)
}

fn fullmoves(input: &str) -> IResult<&str, u32> {
    preceded(peek(one_of("123456789")), halfmoves)(input)
}

pub fn parse(input: &str) -> IResult<&str, ParsedFen> {
    fen(input)
}

#[derive(Debug, Clone)]
pub enum FenParseError {
    Empty,
    Invalid,
    // ParserError(Box<nom::error::ParseError>),
}

impl fmt::Display for FenParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

pub trait FenInterface<T = Self> {
    type Err;
    fn from_fen_str(s: &str) -> Result<T, Self::Err>;
    fn to_fen_string(&self) -> String;
}

impl FenInterface for Position {
    type Err = FenParseError;

    fn from_fen_str(s: &str) -> Result<Self, FenParseError> {
        if s.is_empty() {
            return Err(FenParseError::Empty);
        }

        // let mut board = Self::new();
        // let mut position = Self::new();

        let parser = fen(s);
        let result = match parser {
            Ok((_, res)) => res,
            Err(_e) => return Err(FenParseError::Invalid),
        };

        // position
        let mut position = Self {
            bb: [[0_u64; 8]; 2],
            occupied: [piece::EMPTY; 64],
            to_move: result.to_move,   //color::WHITE,
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

    // fn from_fen_str(s: &str) -> Result<Self, Self::Err> {
    //     Position::from_fen_str(s).and_then(|pos| {
    //         let b = Board::new();
    //         b.set_position(&pos);
    //         Ok(b)
    //     }).or_else(|err| Err(err));
    // }
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
            }
            Err(_e) => Err(FenParseError::Invalid),
        }
    }

    fn to_fen_string(&self) -> String {
        let mut fen_string = String::new();

        // Position
        for y in (0..8).rev() {
            let mut emptycount: u8 = 0;
            for x in 0..8 {
                if 0 == self.current().occupied()[Square::from_coords(x, y) as usize] {
                    emptycount += 1;
                } else {
                    if emptycount > 0 {
                        fen_string.push_str(&emptycount.to_string());
                        emptycount = 0;
                    };
                    fen_string.push_str(
                        self.current().occupied()[Square::from_coords(x, y) as usize]
                            .to_san_string(),
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
        let to_move = match self.current().to_move() {
            WHITE => 'w',
            BLACK => 'b',
            _ => 'w',
        };
        fen_string.push(to_move);

        // Castling rights
        fen_string.push(' ');
        if self.current().castling() == [0, 0] {
            fen_string.push('-');
        } else {
            if 0 != self.current().castling()[WHITE as usize].extract_bits(0, 1) {
                fen_string.push('K');
            }
            if 0 != self.current().castling()[WHITE as usize].extract_bits(1, 1) {
                fen_string.push('Q');
            }
            if 0 != self.current().castling()[BLACK as usize].extract_bits(0, 1) {
                fen_string.push('k');
            }
            if 0 != self.current().castling()[BLACK as usize].extract_bits(1, 1) {
                fen_string.push('q');
            }
        }

        // en passant
        fen_string.push(' ');
        if let Some(eps) = self.current().en_passant() {
            let san = eps[0].to_san_string();
            fen_string.push_str(&san)
        } else {
            fen_string.push('-')
        }

        // Halfmoves
        fen_string.push(' ');
        fen_string.push_str(&self.current().halfmoves().to_string());

        // Fullmoves
        fen_string.push(' ');
        fen_string.push_str(&self.current().fullmoves().to_string());

        fen_string
    }
}
