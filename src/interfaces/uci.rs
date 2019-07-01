use crate::board::Board;
use crate::color::{self, Color};
use crate::interfaces::fen;
use crate::piece::{self, Piece, PiecePrimitives};
use crate::square::{Square, SquarePrimitives};

use nom::IResult;

use nom::{
    branch::alt,
    bytes::complete::{
        is_a,
        tag,
        take, take_while, take_while1, take_while_m_n,
    },
    character::complete::{
        alpha1, digit0, digit1, multispace0, multispace1, one_of,
    },
    combinator::{
        map, map_res, opt, peek, verify,
    },
    multi::count,
    sequence::{
        preceded, terminated, tuple,
    },
};

use std::str::{FromStr, SplitWhitespace};

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

pub struct Uci {
    pub board: Board,
}

impl Default for Uci {
    fn default() -> Self {
        Self::new()
    }
}

fn next_token(input: &str) -> IResult<&str, &str> {
    terminated(
        alpha1,
        multispace0
    )(input)
}

impl Uci {
    pub fn new() -> Self {
        Self {
            board: Board::startpos()
        }
    }

    pub fn run(&mut self) {
        loop {
            let mut line = String::new();
            
            io::stdin()
                .read_line(&mut line)
                .expect("Failed to read line");

            let input = line.trim().split_whitespace();
            
            let cmd = input.next().unwrap();
            match cmd {
                "position" => self.cmd_position(&input),
                "m" | "move" => self.cmd_move(tokens[1..].to_vec()),
                "u" | "undo" => self.cmd_undo(),
                "fen" => println!("{}", self.board.to_fen_string()),
                "b" => self.cmd_b(),
                "p" | "perft" => self.cmd_perft(tokens[1..].to_vec()),
                "g" | "generate" => self.cmd_moves(),
                "uci" => {
                    println!("id name deeprust v{}", env!("CARGO_PKG_VERSION"));
                    println!("id author {}", env!("CARGO_PKG_AUTHORS"));
                    println!("uciok");
                }
                "isready" => println!("readyok"),
                "quit" | "q" => self.run = false,
                unknown => eprintln!("Unknown command: {}", unknown),
            }
            io::stdout().flush().unwrap();
        }
    }

    fn cmd_position(&mut self, input: &SplitWhitespace) -> IResult<&str, ()> {
        // if input.is_empty() {
        //     error!("Invalid position command")
        //     return Err::Error("");
        // }
        let subcmd = input.next().unwrap();
        match subcmd {
            "startpos" => {
                self.board = Board::startpos();
                return Ok((input, ()))
            },
            "fen" => {
                let (input, parsed_fen) = fen::fen(input)?;

                let s = Board::from_fen_str(&cmd[1..7].join(" "));
                match s {
                    Ok(b) => self.board = b,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return;
                    }
                }
                for _ in 0..6 {
                    tokens.next();
                }
            }
            _ => (),
        }

        if let Some(postype) = tokens.next() {
            match *postype {
                
            }
        }

        if tokens.next().is_some() {
            for mov in tokens {
                if mov.len() < 4 {
                    eprintln!("error: incomplete move");
                    return;
                }
                if let (Ok(from), Ok(to)) = (
                    Square::from_san_string(&mov[0..2]),
                    Square::from_san_string(&mov[2..4]),
                ) {
                    match self.board.input_move(from, to, None) {
                        Ok(_) => (),
                        Err(e) => eprintln!("error: could not make move: {}", e),
                    }
                } else {
                    eprintln!("error: invalid move");
                    return;
                }
            }
        }
    }

    fn cmd_move(&mut self, cmd: Vec<&str>) {
        if cmd.is_empty() {
            return;
        }

        for mov in cmd.iter() {
            if mov.len() < 4 {
                eprintln!("error: incomplete move");
                return;
            }
            if let (Ok(from), Ok(to)) = (
                Square::from_san_string(&mov[0..2]),
                Square::from_san_string(&mov[2..4]),
            ) {
                match self.board.input_move(from, to, None) {
                    Ok(_) => (),
                    Err(e) => eprintln!("error: could not make move: {}", e),
                }
            } else {
                eprintln!("error: invalid move");
                return;
            }
        }
    }

    fn cmd_undo(&mut self) {
        self.board.unmake_move();
    }

    fn cmd_b(&self) {
        println!("{}", self.board);
        println!(
            "w in check: {}, b in check: {}",
            MoveGenerator::is_in_check(&self.board, color::WHITE),
            MoveGenerator::is_in_check(&self.board, color::BLACK)
        );
        if !self.board.history().is_empty() {
            println!("last_move: {:#?}", self.board.history().last());
        }
        // for bb in bb::BB_FILE_MASK_EX.iter() {
        //     println!("{}", bb.to_debug_string());
        // }
    }

    fn cmd_moves(&mut self) {
        let moves = self.board.generate_moves();
        println!("count: {}", moves.len());
        for m in moves.iter() {
            println!("move: {:#?}", m);
        }
    }

    fn cmd_perft(&mut self, cmd: Vec<&str>) {
        let depth = if !cmd.is_empty() {
            cmd[0].parse::<u32>().unwrap()
        } else {
            3
        };

        let ctx = Board::perft(&mut self.board, depth);
        println!("perft result: {}", ctx);
    }
}
