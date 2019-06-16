use std::io;
use std::io::Write;

use board::*;
use bitboard::*;
use util::*;
use move_generator::MoveGenerator;
use square::{Square, SquarePrimitives};

pub struct UCIInterface {
    pub board: Board,
    gen: MoveGenerator,
    run: bool,
}

impl UCIInterface {
    pub fn new() -> UCIInterface {
        UCIInterface { 
            board: Board::new(),
            gen: MoveGenerator::new(),
            run: true }
    }

    fn cmd_position(&mut self, cmd: Vec<&str>) {
        if cmd.len() < 1 { return; }
        let mut tokens = cmd.iter();

        if let Some(postype) = tokens.next() {
            match postype {
                &"startpos" => self.board = Board::startpos(),
                &"fen" => {
                    let s = Board::from_fen(cmd[1..7].join(" "));
                    match s {
                        Ok(b) => self.board = b,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            return;
                        }
                    }
                    for i in 0..6 { tokens.next(); }
                }
                &_ => ()
            }
        }

        if let Some(_) = tokens.next() {
            while let Some(mov) = tokens.next() {
                if mov.len() < 4 { 
                    eprintln!("error: incomplete move");
                    return;
                }
                if let (Ok(from), Ok(to)) = (Square::from_san_string(&mov[0..2]), Square::from_san_string(&mov[2..4])) {
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
        if cmd.len() < 1 { return; }

        let mut tokens = cmd.iter();

        while let Some(mov) = tokens.next() {
            if mov.len() < 4 { 
                eprintln!("error: incomplete move");
                return;
            }
            if let (Ok(from), Ok(to)) = (Square::from_san_string(&mov[0..2]), Square::from_san_string(&mov[2..4])) {
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
        self.board.undo_move();
    }

    fn cmd_b(&self) {
        let occ = self.board.occupied();
        
        println!();
        for y in (0..8).rev() {
            println!("+---+---+---+---+---+---+---+---+");
            for x in 0..8 {
                print!("| {} ", Board::occ_piece_code_to_str(occ[Square::from_coords(x, y) as usize]));
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
        println!("w in check: {}, b in check: {}", MoveGenerator::is_in_check(&self.board, piece::WHITE), MoveGenerator::is_in_check(&self.board, piece::BLACK));
        if self.board.move_stack().len() > 0 {
            println!("last_move: {:#?}", self.board.move_stack().peek());
        }
    }

    fn cmd_bb(&self) {
        let to_move = self.board.to_move();
        println!("{}", bb::north_one(self.board.bb_pawns(to_move)).to_debug_string());
        println!("{}", self.board.bb_empty().to_debug_string());
        println!("{}", bb::BB_RANKS[0].to_debug_string());
        println!("{}", self.board.bb_knights(to_move).to_debug_string());
        println!("{}", bb::BB_DIAG[squares::E4 as usize].to_debug_string());
        println!("{}", bb::BB_ANTI_DIAG[squares::E4 as usize].to_debug_string());
        println!("{}", bb::BB_BISHOP_ATTACKS[squares::E4 as usize].to_debug_string());
        println!("{}", bb::BB_ROOK_ATTACKS[squares::E4 as usize].to_debug_string());
        println!("{}", bb::BB_QUEEN_ATTACKS[squares::E4 as usize].to_debug_string());
        println!("{}", bb::BB_RAYS_WEST[squares::E1 as usize].to_debug_string());
        println!("{}", bb::BB_RAYS_EAST[squares::E1 as usize].to_debug_string());
        println!("{}", bb::BB_KG_FILL_UP_ATTACKS[squares::H1 as usize][0b111111].to_debug_string());
        println!("{}", bb::diagonal_attacks(squares::D4, 0b11111111_00000000_11111111).to_debug_string());
        println!("{}", bb::anti_diagonal_attacks(squares::D4, 0b11111111_00000000_11111111).to_debug_string());
        println!("{}", bb::file_attacks(squares::D4, 0b11000011_11111111_00000000_00000000_11000011_11111111_00000000_11111111).to_debug_string());
        println!("{}", 0b11111111_01111111_00000000_10000000_00000001_00000000_11111110_11111111.to_debug_string());
        println!("file_attacks: \n{}", bb::file_attacks(squares::H8, 0b11111111_01111111_00000000_10000000_00000001_00000000_11111110_11111111).to_debug_string());
    }

    fn cmd_moves(&mut self) {
        let moves = MoveGenerator::from_board(&self.board);
        println!("count: {}", moves.len());
        for m in moves.iter() {
            println!("move: {:#?}", m);
        }
        
    }

    fn cmd_perft(&mut self, cmd: Vec<&str>) {
        let mut depth = 3;
        if cmd.len() >= 1 { 
            depth = cmd[0].parse::<u32>().unwrap();
        }

        let movecount = MoveGenerator::perft(&mut self.board, depth);
        println!("perft result: {}", movecount);
    }

    pub fn parse(&mut self, cmd: String) {
        let tokens: Vec<&str> = cmd.trim().split_whitespace().collect();

        if tokens.len() > 0 {
            match tokens[0] {
                "position" => self.cmd_position(tokens[1..].to_vec()),
                "m" | "move" => self.cmd_move(tokens[1..].to_vec()),
                "u" | "undo" => self.cmd_undo(),
                "fen" => println!("{}", self.board.to_fen()),
                "b" => self.cmd_b(),
                "bb" => self.cmd_bb(),
                "p" | "perft" => self.cmd_perft(tokens[1..].to_vec()),
                "g" | "generate" => self.cmd_moves(),
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

        // castling, king moves
        c.parse(String::from("position fen 8/8/8/8/8/8/8/R3K2R w KQkq - 0 1 moves e1g1"));
        assert_eq!("8/8/8/8/8/8/8/R4RK1 b kq - 1 1", c.board.to_fen());
        c.parse(String::from("position fen 8/8/8/8/8/8/8/R3K2R w KQkq - 0 1 moves e1c1"));
        assert_eq!("8/8/8/8/8/8/8/2KR3R b kq - 1 1", c.board.to_fen());
        c.parse(String::from("position fen r3k2r/8/8/8/8/8/8/8 b KQkq - 0 1 moves e8g8"));
        assert_eq!("r4rk1/8/8/8/8/8/8/8 w KQ - 1 2", c.board.to_fen());
        c.parse(String::from("position fen r3k2r/8/8/8/8/8/8/8 b KQkq - 0 1 moves e8c8"));
        assert_eq!("2kr3r/8/8/8/8/8/8/8 w KQ - 1 2", c.board.to_fen());
        
        // castling, rook moves
        c.parse(String::from("position fen 8/8/8/8/8/8/8/R3K2R w KQkq - 0 1 moves a1b1"));
        assert_eq!("8/8/8/8/8/8/8/1R2K2R b Kkq - 1 1", c.board.to_fen());
        c.parse(String::from("position fen 8/8/8/8/8/8/8/R3K2R w KQkq - 0 1 moves h1g1"));
        assert_eq!("8/8/8/8/8/8/8/R3K1R1 b Qkq - 1 1", c.board.to_fen());
        c.parse(String::from("position fen r3k2r/8/8/8/8/8/8/8 b KQkq - 0 1 moves a8b8"));
        assert_eq!("1r2k2r/8/8/8/8/8/8/8 w KQk - 1 2", c.board.to_fen());
        c.parse(String::from("position fen r3k2r/8/8/8/8/8/8/8 b KQkq - 0 1 moves h8g8"));
        assert_eq!("r3k1r1/8/8/8/8/8/8/8 w KQq - 1 2", c.board.to_fen());
        
        c.parse(String::from("position fen 8/8/8/8/8/8/4P3/8 w KQkq - 0 1 moves e2e4"));
        assert_eq!("8/8/8/8/4P3/8/8/8 b KQkq e3 0 1", c.board.to_fen());
    }
}
