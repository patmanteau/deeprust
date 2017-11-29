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

    fn cmd_b(&self) {
        let occ = self.board.occupied();
        
        println!();
        for y in (0..8).rev() {
            println!("+---+---+---+---+---+---+---+---+");
            for x in 0..8 {
                print!("| {} ", engine::board::occ_piece_code_to_str(occ[engine::board::c2s(x, y) as usize]));
                if x == 7 {
                    println!("| {}", y+1);
                }
            }
            if y == 0 {
                println!("+---+---+---+---+---+---+---+---+");
                println!("  A   B   C   D   E   F   G   H");
            }
        }
        println!();
        println!("fen: {}", self.board.to_fen());
    }

    fn parse(&mut self, cmd: String) {
        let tokens: Vec<&str> = cmd.trim().split_whitespace().collect();

        if tokens.len() > 0 {
            match tokens[0] {
                "position" => self.cmd_position(tokens[1..].to_vec()),
                "fen" => println!("{}", self.board.to_fen()),
                "b" => self.cmd_b(),
                "uci" => {
                    println!("id name deeprust v{}", env!("CARGO_PKG_VERSION"));
                    println!("id author {}", env!("CARGO_PKG_AUTHORS"));
                    println!("uciok");
                },
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
        assert_eq!("rnbqkbnr/ppp2ppp/8/3pp3/3PP3/8/PPP2PPP/RNBQKBNR w KQkq e6 0 3", c.board.to_fen());
    }
}
