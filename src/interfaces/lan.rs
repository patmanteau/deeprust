use crate::color;
use crate::piece::{self, Piece, PiecePrimitives};
use crate::square::Square;

use nom::IResult;

use nom::{
    bytes::complete::take,
    character::complete::one_of,
    combinator::{map, opt, peek},
    sequence::{preceded, tuple},
};

use std::collections::HashMap;
use std::str::FromStr;

//type IResult<I, O, E = (I, ErrorKind)> = Result<(I, O), Err<E>>;

#[derive(Debug, Clone)]
pub enum LanParseError {
    Empty,
    Invalid,
    // ParserError(Box<nom::error::ParseError>),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Needed {
    Unknown,
    Size(u32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Err<E> {
    Incomplete(Needed),
    Error(E),
    Failure(E),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ParsedMove {
    pub from: Square,
    pub to: Square,
    pub prom: Option<Piece>,
}

pub fn lan(input: &str) -> Result<ParsedMove, LanParseError> {
    let result = tuple((square, square, opt(prom)))(input);

    match result {
        Ok(tup) => {
            let (rest, (from, to, prom)) = tup;
            Ok(ParsedMove { from, to, prom })
        }
        Err(e) => Err(LanParseError::Invalid),
    }
}

fn square(input: &str) -> IResult<&str, Square> {
    map(
        preceded(
            peek(tuple((one_of("abcdefgh"), one_of("12345678")))),
            take(2_usize),
        ),
        from_san_string,
    )(input)
}

fn str_to_piececode(input: char) -> Piece {
    Piece::new(
        match input {
            'N' | 'n' => piece::KNIGHT,
            'B' | 'b' => piece::BISHOP,
            'R' | 'r' => piece::ROOK,
            'Q' | 'q' => piece::QUEEN,
            _ => unreachable!("LAN parser error: Invalid promotion character"),
        },
        color::WHITE,
    )
}

fn prom(input: &str) -> IResult<&str, Piece> {
    map(one_of("QqRrBbNn"), str_to_piececode)(input)
}

fn from_san_string(square: &str) -> Square {
    if square.len() != 2 {
        unreachable!("LAN parser error: Invalid square string");
    }

    let file_lookup: HashMap<&'static str, Square> = [
        ("a", 0),
        ("b", 1),
        ("c", 2),
        ("d", 3),
        ("e", 4),
        ("f", 5),
        ("g", 6),
        ("h", 7),
    ]
    .iter()
    .cloned()
    .collect();

    let x = match file_lookup.get(&square[0..1]) {
        Some(val) => val,
        None => unreachable!("LAN parser error: Invalid file character"),
    };
    let y = match u16::from_str(&square[1..2]) {
        Ok(val) => val - 1,
        Err(_) => unreachable!("LAN parser error: Invalid square character"),
    };

    (y << 3) + *x
}

// pub fn parse(input: &str) -> IResult<&str, ParsedMove> {
//     lan(input)
// }
