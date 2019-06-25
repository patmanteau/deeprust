use crate::color::{self, Color};
use crate::interfaces::fen;
use crate::piece::{self, Piece, PiecePrimitives};
use crate::square::{Square, SquarePrimitives};

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

use std::str::FromStr;

// use nom::{
//     one_of,
// };

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
                    .flatten()
                    .rev()
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

fn ep_rank(input: &str) -> IResult<&str, char> {
    one_of("36")(input)
}

fn file_letter(input: &str) -> IResult<&str, char> {
    one_of("abcdefgh")(input)
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
