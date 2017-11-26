use std::io;
use std::io::Write;

use engine;

pub struct UCIInterface {
    board: engine::board::Board,
    run: bool,
}

impl UCIInterface {
    pub fn new() -> UCIInterface {
        UCIInterface { board: engine::board::Board::new(), run: true }
    }

    fn cmd_position(&mut self, cmd: Vec<&str>) {
        if cmd.len() < 1 { return; }
        let mut tokens = cmd.iter();

        if let Some(postype) = tokens.next() {
            match postype {
                &"startpos" => self.board = engine::board::Board::startpos(),
                &"fen" => {
                    let s = engine::board::Board::from_fen(cmd[1..7].join(" "));
                    match s {
                        Ok(b) => self.board = b,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            return;
                        }
                    }
                    for i in 0..6 { tokens.next(); }
                }
                &&_ => ()
            }
        }

        if let Some(_) = tokens.next() {
            while let Some(mov) = tokens.next() {
                if mov.len() < 4 { 
                    eprintln!("error: incomplete move");
                    return;
                }
                if let (Ok(from), Ok(to)) = (engine::san::SAN::square_str_to_index(&mov[0..2]), engine::san::SAN::square_str_to_index(&mov[2..4])) {
                    eprintln!("from: {}, to: {}", from, to);
                    match self.board.input_move(from, to) {
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

    fn parse(&mut self, cmd: String) {
        let tokens: Vec<&str> = cmd.trim().split_whitespace().collect();

        if tokens.len() > 0 {
            match tokens[0] {
                "position" => self.cmd_position(tokens[1..].to_vec()),
                "getpos" => println!("{}", self.board.to_fen()),
                "isready" => println!("readyok"),
                "quit" => self.run = false,
                unknown => eprintln!("Unknown command: {}", unknown),
            }
        }
    }

    pub fn run(&mut self) {
        println!("id name deeprust v{}", env!("CARGO_PKG_VERSION"));
        println!("id author {}", env!("CARGO_PKG_AUTHORS"));
        println!("uciok");
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
    fn isready() {
        
    }

    #[test]
    fn it_handles_startpos() {
        let mut c = UCIInterface::new();
        c.parse(String::from("position startpos"));
        assert_eq!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", c.board.to_fen());
    }

    #[test]
    fn it_handles_fen_positions() {
        let mut c = UCIInterface::new();
        c.parse(String::from("position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"));
        assert_eq!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", c.board.to_fen());

        c.parse(String::from("position fen 8/k7/3p4/p2P1p2/P2P1P2/8/8/K7 w - - 0 1"));
        assert_eq!("8/k7/3p4/p2P1p2/P2P1P2/8/8/K7 w - - 0 1", c.board.to_fen());
    }

    #[test]
    fn it_handles_moves() {
        let mut c = UCIInterface::new();
        c.parse(String::from("position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 moves d2d4 d7d5 e2e4 e7e5"));
        assert_eq!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", c.board.to_fen());
    }
}
