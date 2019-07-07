use std::fmt;

use crate::engine::bitboard::*;

use crate::interfaces::lan;
use crate::interfaces::lan::LanParseError;
use crate::interfaces::FenInterface;
// use crate::primitives::r#move::{Move, MoveStack};
// use crate::primitives::piece::Piece;
use crate::engine::position::{Position, PositionStack};
// use crate::primitives::square::{Square, SquarePrimitives};

use crate::primitives::*;

//use regex::Regex;

//use std::str::FromStr;

pub const PSTACK_SIZE: usize = 64;

#[derive(Clone)]
pub struct Board {
    positions: PositionStack,
    pcursor: usize,
    history: MoveStack,
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Board {{ Position: {:?}, pcursor: {}, move_stack: {:?} }}",
            self.current(),
            self.pcursor,
            self.history
        )
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.current()).unwrap();
        writeln!(f).unwrap();
        writeln!(f, "fen: {}", self.current().to_fen_string()).unwrap();
        write!(f, "moves: ")?;
        for mov in &self.history {
            write!(f, "{} ", mov)?;
        }
        writeln!(f)?;
        writeln!(f, "to_move: {}", self.current().to_move()).unwrap();
        writeln!(f)
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    pub fn new() -> Board {
        Board {
            positions: vec![Position::new(); PSTACK_SIZE],
            pcursor: 0,
            history: MoveStack::with_capacity(32),
        }
    }

    #[inline]
    pub fn current(&self) -> &Position {
        &self.positions[self.pcursor]
    }

    pub fn startpos() -> Board {
        let mut b = Self::new();
        b.set_position(
            &Position::from_fen_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                .unwrap(),
        );
        b
        // Self::from_fen_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    #[inline]
    pub fn set_position(&mut self, position: &Position) {
        self.positions[self.pcursor] = *position;
    }

    #[inline]
    pub fn bb(&self) -> &[Bitboard; 14] {
        &self.current().bb()
    }

    #[inline]
    pub fn history(&self) -> &MoveStack {
        &self.history
    }

    #[inline]
    pub fn make_move(&mut self, mov: Move) {
        debug_assert!(self.pcursor + 1 < PSTACK_SIZE);
        // self.positions[self.pcursor + 1] = self.current().clone();
        self.positions[self.pcursor + 1] = self.positions[self.pcursor]; //*self.current();
        self.positions[self.pcursor + 1].make_move(mov);
        self.history.push(mov);
        self.pcursor += 1;
    }

    #[inline]
    pub fn unmake_move(&mut self) {
        debug_assert!(self.pcursor > 0);
        self.history.pop();
        self.pcursor -= 1;
    }

    pub fn input_move(
        &mut self,
        orig: Square,
        dest: Square,
        promote_to: Option<Piece>,
    ) -> Result<bool, &'static str> {
        debug_assert!(self.pcursor + 1 < PSTACK_SIZE);
        self.positions[self.pcursor + 1] = *self.current();
        match self.positions[self.pcursor + 1].input_move(orig, dest, promote_to) {
            Ok(mov) => {
                self.history.push(mov);
                self.pcursor += 1;
                Ok(true)
            }
            Err(err) => Err(err),
        }
    }

    pub fn input_san_move(&mut self, san_move: &str) -> Result<(), LanParseError> {
        let lan_mov = lan(san_move)?;
        self.input_move(lan_mov.from, lan_mov.to, lan_mov.prom)
            .unwrap();
        Ok(())
    }

    pub fn panic_dump(&self) {
        error!("{}", self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::primitives::colors;
    use crate::engine::Search;
    // use crate::primitives::square;
    use std::error::Error;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::path::Path;

    #[test]
    fn it_sets_correct_startpos() {
        let b = Board::startpos();
        let pos = b.current();

        // color boards
        assert_eq!(pos.bb_own(colors::WHITE), 0xffff_u64);
        assert_eq!(pos.bb_opponent(colors::WHITE), 0xffff << (6 * 8));

        // pawn boards
        assert_eq!(pos.bb_pawns(colors::WHITE), 0xff << 8);
        assert_eq!(pos.bb_pawns(colors::BLACK), 0xff << (6 * 8));

        // rook boards
        assert_eq!(pos.bb_rooks(colors::WHITE), 0x81);
        assert_eq!(pos.bb_rooks(colors::BLACK), 0x81 << (7 * 8));

        // bishop boards
        assert_eq!(pos.bb_bishops(colors::WHITE), 0x24);
        assert_eq!(pos.bb_bishops(colors::BLACK), 0x24 << (7 * 8));

        // queen boards
        assert_eq!(pos.bb_queens(colors::WHITE), 0x8);
        assert_eq!(pos.bb_queens(colors::BLACK), 0x8 << (7 * 8));

        // king boards
        assert_eq!(pos.bb_king(colors::WHITE), 0x10);
        assert_eq!(pos.bb_king(colors::BLACK), 0x10 << (7 * 8));

        assert!(b.history().is_empty());
    }

    #[test]
    fn it_makes_correct_fen_strings() {
        let fen_strs = vec!["rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"];

        let board = Board::startpos();
        assert!(board.history().is_empty());

        for fen_str in fen_strs {
            if let Ok(board) = Board::from_fen_str(fen_str) {
                assert_eq!(fen_str, board.to_fen_string());
            } else {
                panic!("Illegal FEN string");
            }
        }
    }

    #[test]
    fn it_parses_fen_strings_correctly() {
        let pospath = Path::new("tests/hyatt-4000-openings.epd");
        let posfile = match File::open(&pospath) {
            Err(why) => panic!(
                "Could not open {}: {}",
                pospath.display(),
                why.description()
            ),
            Ok(file) => file,
        };

        for (line, position) in BufReader::new(posfile)
            .lines()
            .map(|l| l.unwrap())
            .enumerate()
        {
            let b = Board::from_fen_str(&position);
            match b {
                Err(e) => panic!("Error reading {}:{}:{}", pospath.display(), line, e),
                Ok(board) => assert_eq!(position, board.to_fen_string()),
            }
        }
    }

    #[test]
    fn it_rejects_invalid_fen_strings() {
        let fen_strs = vec![
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq abcdefg 0 1",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR y KQkq e3 0 1",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w HFhf e3 0 1",
        ];

        for fen_str in fen_strs {
            let b = Board::from_fen_str(fen_str);
            match b {
                Err(_e) => continue,
                Ok(_board) => panic!(),
            }
        }
    }

    #[test]
    fn it_makes_moves() {
        if let Ok(mut board) =
            Board::from_fen_str("rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 1")
        {
            assert_eq!(0, board.history().len());
            board.input_move(squares::D7, squares::D6, None).unwrap();
            assert_eq!(1, board.history().len());
            assert_eq!(None, board.current().en_passant());
            let last_move = board.history().last().unwrap();
            assert_eq!(last_move.orig(), squares::D7);
            assert_eq!(last_move.dest(), squares::D6);
        }

        let mut board = Board::from_fen_str("8/3p4/8/4P3/8/8/8/8 b - - 0 1").unwrap();
        board.input_move(squares::D7, squares::D5, None).unwrap();
        board.input_move(squares::E5, squares::D6, None).unwrap();
        assert_eq!(0, board.current().occupied()[squares::D5 as usize]);
    }

    // TODO: move to UCI tests
    // #[test]
    fn it_unmakes_moves() {
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
    }

    #[test]
    fn it_unwinds_its_move_stack() {
        {
            let fen = String::from(
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            );
            let mut board = Board::from_fen_str(&fen.clone()).unwrap();
            let _ctx = Board::perft(&mut board, 4);
            assert_eq!(fen, board.to_fen_string());
        }
        {
            let board_orig = Board::startpos();
            let mut board = board_orig.clone();
            let _ctx = Board::perft(&mut board, 4);
        }
    }
}
