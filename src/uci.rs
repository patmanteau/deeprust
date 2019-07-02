use std::io;
use std::io::Write;

use crate::board::*;
use crate::color;
use crate::interfaces::FenInterface;
use crate::move_generator::MoveGenerator;
use crate::search::Search;
use crate::square::{Square, SquarePrimitives};

pub struct UCIInterface {
    pub board: Board,
    // gen: MoveGenerator,
    run: bool,
}

impl Default for UCIInterface {
    fn default() -> Self {
        Self::new()
    }
}

impl UCIInterface {
    pub fn new() -> UCIInterface {
        UCIInterface {
            board: Board::new(),
            //gen: MoveGenerator::new(),
            run: true,
        }
    }

    fn cmd_position(&mut self, cmd: Vec<&str>) {
        if cmd.is_empty() {
            return;
        }
        let mut tokens = cmd.iter();

        if let Some(postype) = tokens.next() {
            match *postype {
                "startpos" => self.board = Board::startpos(),
                "fen" => {
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

    fn cmd_divide(&mut self, cmd: Vec<&str>) {
        let depth = if !cmd.is_empty() {
            cmd[0].parse::<u32>().unwrap()
        } else {
            3
        };

        let moves = self.board.generate_moves();
        let mut nodes = 0;
        for mov in &moves {
            self.board.make_move(*mov);
            if self.board.is_in_check(1 ^ self.board.current().to_move()) {
                self.board.unmake_move();
            } else {
                let res = self.board.perft(depth - 1);
                self.board.unmake_move();
                println!("{} {}", mov, res.nodes);
                nodes += res.nodes;
            }
        }
        println!();
        println!("{} moves, {} nodes", moves.len(), nodes);
    }

    pub fn parse(&mut self, cmd: String) {
        let tokens: Vec<&str> = cmd.trim().split_whitespace().collect();

        if !tokens.is_empty() {
            match tokens[0] {
                "position" => self.cmd_position(tokens[1..].to_vec()),
                "m" | "move" => self.cmd_move(tokens[1..].to_vec()),
                "u" | "undo" => self.cmd_undo(),
                "fen" => println!("{}", self.board.to_fen_string()),
                "b" => self.cmd_b(),
                "p" | "perft" => self.cmd_perft(tokens[1..].to_vec()),
                "d" | "divide" => self.cmd_divide(tokens[1..].to_vec()),
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
        }
    }

    pub fn run(&mut self) {
        io::stdout().flush().unwrap();
        while self.run {
            let mut command_line = String::new();
            io::stdin()
                .read_line(&mut command_line)
                .expect("Failed to read line");
            self.parse(command_line);

            io::stdout().flush().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isready() {}

    #[test]
    fn it_handles_startpos() {
        let mut c = UCIInterface::new();
        c.parse(String::from("position startpos"));
        assert_eq!(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            c.board.to_fen_string()
        );
    }

    #[test]
    fn it_handles_fen_positions() {
        let mut c = UCIInterface::new();
        c.parse(String::from(
            "position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        ));
        assert_eq!(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            c.board.to_fen_string()
        );

        c.parse(String::from(
            "position fen 8/k7/3p4/p2P1p2/P2P1P2/8/8/K7 w - - 0 1",
        ));
        assert_eq!(
            "8/k7/3p4/p2P1p2/P2P1P2/8/8/K7 w - - 0 1",
            c.board.to_fen_string()
        );
    }

    // TODO: make full UCI commands
    // #[test]
    fn it_handles_moves() {
        let mut c = UCIInterface::new();
        let one_move_strs = vec![
            (
                "rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 1",
                " moves d7d5",
            ),
            // castling, king moves
            ("4k3/8/8/8/8/8/8/R3K2R w KQkq - 0 1", " moves e1g1"),
            ("4k3/8/8/8/8/8/8/R3K2R w KQkq - 0 1", " moves e1c1"),
            ("r3k2r/8/8/8/8/8/8/4K3 b KQkq - 0 1", " moves e8g8"),
            ("r3k2r/8/8/8/8/8/8/4K3 b KQkq - 0 1", " moves e8c8"),
            // castling, rook moves
            ("4k3/8/8/8/8/8/8/R3K2R w KQkq - 0 1", " moves a1b1"),
            ("4k3/8/8/8/8/8/8/R3K2R w KQkq - 0 1", " moves h1g1"),
            ("r3k2r/8/8/8/8/8/8/4K3 b KQkq - 0 1", " moves a8b8"),
            ("r3k2r/8/8/8/8/8/8/4K3 b KQkq - 0 1", " moves h8g8"),
        ];

        let two_move_strs = vec![
            // en passant
            ("8/3p4/8/4P3/8/8/8/8 b - - 0 1", " moves d7d5 e5d6"),
        ];

        for (one_mover_fen, one_mover_moves) in one_move_strs {
            if let Ok(mut board) =
                Board::from_fen_str(&(String::from(one_mover_fen) + &String::from(one_mover_moves)))
            {
                board.unmake_move();
                assert_eq!(one_mover_fen, board.to_fen_string());
            } else {
                panic!();
            }
        }

        for (two_mover_fen, two_mover_moves) in two_move_strs {
            if let Ok(mut board) =
                Board::from_fen_str(&(String::from(two_mover_fen) + &String::from(two_mover_moves)))
            {
                board.unmake_move();
                board.unmake_move();
                assert_eq!(two_mover_fen, board.to_fen_string());
            } else {
                panic!();
            }
        }
        let uci_strs = vec![
            ("position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 moves d2d4 d7d5 e2e4 e7e5", "rnbqkbnr/ppp2ppp/8/3pp3/3PP3/8/PPP2PPP/RNBQKBNR w KQkq e6 0 3"),
            // castling, king moves
            ("position fen 8/8/8/8/8/8/8/R3K2R w KQkq - 0 1 moves e1g1", "8/8/8/8/8/8/8/R4RK1 b kq - 1 1"),
            ("position fen 8/8/8/8/8/8/8/R3K2R w KQkq - 0 1 moves e1c1", "8/8/8/8/8/8/8/2KR3R b kq - 1 1"),
            ("position fen r3k2r/8/8/8/8/8/8/8 b KQkq - 0 1 moves e8g8", "r4rk1/8/8/8/8/8/8/8 w KQ - 1 2"),
            ("position fen r3k2r/8/8/8/8/8/8/8 b KQkq - 0 1 moves e8c8", "2kr3r/8/8/8/8/8/8/8 w KQ - 1 2"),
            // castling, rook moves
            ("position fen 8/8/8/8/8/8/8/R3K2R w KQkq - 0 1 moves a1b1", "8/8/8/8/8/8/8/1R2K2R b Kkq - 1 1"),
            ("position fen 8/8/8/8/8/8/8/R3K2R w KQkq - 0 1 moves h1g1", "8/8/8/8/8/8/8/R3K1R1 b Qkq - 1 1"),
            ("position fen r3k2r/8/8/8/8/8/8/8 b KQkq - 0 1 moves a8b8", "1r2k2r/8/8/8/8/8/8/8 w KQk - 1 2"),
            ("position fen r3k2r/8/8/8/8/8/8/8 b KQkq - 0 1 moves h8g8", "r3k1r1/8/8/8/8/8/8/8 w KQq - 1 2"),
            // pawn move
            ("position fen 8/8/8/8/8/8/4P3/8 w KQkq - 0 1 moves e2e4", "8/8/8/8/4P3/8/8/8 b KQkq e3 0 1"),
        ];

        for (uci_in, fen_out) in uci_strs {
            c.parse(String::from(uci_in));
            assert_eq!(fen_out, c.board.to_fen_string())
        }
    }
}
