use std::fmt;

use crate::bitboard::*;
use crate::common::*;

use crate::color::{self, Color};
use crate::moves::{Move, MoveStack};
use crate::piece::{self, Piece, PiecePrimitives};
use crate::position::{Position, PositionStack};
use crate::square::{self, Square, SquarePrimitives};

use std::str::FromStr;

pub const PSTACK_SIZE: usize = 64;

#[derive(Clone)]
pub struct Board {
    pstack: PositionStack,
    pcursor: usize,
    move_stack: MoveStack,
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Board {{ Position: {:?}, pcursor: {}, move_stack: {:?} }}",
            self.current(),
            self.pcursor,
            self.move_stack
        )
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.current()).unwrap();
        writeln!(f).unwrap();
        writeln!(f, "fen: {}", self.to_fen()).unwrap();
        //writeln!(f, "move_stack: {}", self.move_stack).unwrap();
        writeln!(f, "to_move: {}", self.to_move()).unwrap();
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
            pstack: vec![Position::new(); PSTACK_SIZE],
            pcursor: 0,
            move_stack: MoveStack::with_capacity(1024)
        }
    }

    #[inline]
    pub fn current(&self) -> &Position {
        &self.pstack[self.pcursor]
    }

    pub fn startpos() -> Board {
        let mut board = Board::new();
        let mut position = &mut board.pstack[board.pcursor];

        // pawns
        for x in 0..8 {
            position.set_piece(piece::PAWN, color::WHITE, Square::from_coords(x, 1));
            position.set_piece(piece::PAWN, color::BLACK, Square::from_coords(x, 6));
        }

        // knights
        position.set_piece(piece::KNIGHT, color::WHITE, square::B1);
        position.set_piece(piece::KNIGHT, color::WHITE, square::G1);
        position.set_piece(piece::KNIGHT, color::BLACK, square::B8);
        position.set_piece(piece::KNIGHT, color::BLACK, square::G8);

        // bishops
        position.set_piece(piece::BISHOP, color::WHITE, square::C1);
        position.set_piece(piece::BISHOP, color::WHITE, square::F1);
        position.set_piece(piece::BISHOP, color::BLACK, square::C8);
        position.set_piece(piece::BISHOP, color::BLACK, square::F8);

        // rooks
        position.set_piece(piece::ROOK, color::WHITE, square::A1);
        position.set_piece(piece::ROOK, color::WHITE, square::H1);
        position.set_piece(piece::ROOK, color::BLACK, square::A8);
        position.set_piece(piece::ROOK, color::BLACK, square::H8);

        // queens
        position.set_piece(piece::QUEEN, color::WHITE, square::D1);
        position.set_piece(piece::QUEEN, color::BLACK, square::D8);

        // kings
        position.set_piece(piece::KING, color::WHITE, square::E1);
        position.set_piece(piece::KING, color::BLACK, square::E8);

        position.castling = [3, 3];
        board
    }

    pub fn from_fen(fen_string: String) -> Result<Board, &'static str> {
        let mut board = Self::new();
        let mut position = Position::new();

        // position
        let mut x = 0;
        let mut y = 7;
        let mut fen_iter = fen_string.split_whitespace();
        if let Some(piece_list) = fen_iter.next() {
            for chr in piece_list.chars() {
                if let Some(empty) = chr.to_digit(10) {
                    x += empty
                } else if chr == '/' {
                    x = 0;
                    y -= 1;
                } else {
                    match chr {
                        'P' => {
                            position.set_piece(piece::PAWN, color::WHITE, Square::from_coords(x, y))
                        }
                        'N' => {
                            position.set_piece(piece::KNIGHT, color::WHITE, Square::from_coords(x, y))
                        }
                        'B' => {
                            position.set_piece(piece::BISHOP, color::WHITE, Square::from_coords(x, y))
                        }
                        'R' => {
                            position.set_piece(piece::ROOK, color::WHITE, Square::from_coords(x, y))
                        }
                        'Q' => {
                            position.set_piece(piece::QUEEN, color::WHITE, Square::from_coords(x, y))
                        }
                        'K' => {
                            position.set_piece(piece::KING, color::WHITE, Square::from_coords(x, y))
                        }
                        'p' => {
                            position.set_piece(piece::PAWN, color::BLACK, Square::from_coords(x, y))
                        }
                        'n' => {
                            position.set_piece(piece::KNIGHT, color::BLACK, Square::from_coords(x, y))
                        }
                        'b' => {
                            position.set_piece(piece::BISHOP, color::BLACK, Square::from_coords(x, y))
                        }
                        'r' => {
                            position.set_piece(piece::ROOK, color::BLACK, Square::from_coords(x, y))
                        }
                        'q' => {
                            position.set_piece(piece::QUEEN, color::BLACK, Square::from_coords(x, y))
                        }
                        'k' => {
                            position.set_piece(piece::KING, color::BLACK, Square::from_coords(x, y))
                        }
                        _ => return Err("Invalid FEN string"),
                    }
                    x += 1;
                }
            }
        } else {
            return Err("Invalid FEN string, no piece list found");
        }

        // to move
        if let Some(to_move) = fen_iter.next() {
            match to_move {
                "w" => position.to_move = color::WHITE,
                "b" => position.to_move = color::BLACK,
                _ => return Err("Invalid ToMove char"),
            }
        } else {
            return Err("Invalid FEN string, don't know who moves next");
        }

        // Castling rights
        if let Some(castling) = fen_iter.next() {
            for chr in castling.chars() {
                match chr {
                    '-' => position.castling = [0, 0],
                    'K' => position.castling[color::WHITE as usize] |= 0x1,
                    'Q' => position.castling[color::WHITE as usize] |= 0x2,
                    'k' => position.castling[color::BLACK as usize] |= 0x1,
                    'q' => position.castling[color::BLACK as usize] |= 0x2,
                    _ => return Err("Invalid castling char"),
                }
            }
        } else {
            return Err("Invalid FEN string, no castling rights found");
        }

        // en passant
        if let Some(en_passant) = fen_iter.next() {
            if en_passant == "-" {
                position.en_passant = None;
            } else {
                //match SAN::square_str_to_index(en_passant) {
                match Square::from_san_string(en_passant) {
                    Ok(eps) => position.en_passant = Some([eps, eps.flipped()]),
                    Err(_) => return Err("Error parsing en passant field"),
                }
            }
        } else {
            return Err("Invalid FEN string, no en passant information");
        }

        // Halfmoves
        if let Some(halfmoves) = fen_iter.next() {
            match u32::from_str(halfmoves) {
                Ok(val) => position.halfmoves = val,
                Err(_) => return Err("Error parsing halfmoves"),
            }
        } else {
            return Err("Invalid FEN string, no halfmoves given");
        }

        // Fullmoves
        if let Some(fullmoves) = fen_iter.next() {
            match u32::from_str(fullmoves) {
                Ok(val) => position.fullmoves = val,
                Err(_) => return Err("Error parsing fullmoves"),
            }
        } else {
            return Err("Invalid FEN string, no fullmoves given");
        }

        board.pstack[0] = position;

        if let Some(move_token) = fen_iter.next() {
            if move_token == "moves" {
                for mov in fen_iter {
                    match board.input_san_move(mov) {
                        Ok(_) => continue,
                        Err(err) => return Err(err)
                    }
                }
            }
        }
        Ok(board)
    }

    pub fn to_fen(&self) -> String {
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
            color::WHITE => 'w',
            color::BLACK => 'b',
            _ => 'w',
        };
        fen_string.push(to_move);

        // Castling rights
        fen_string.push(' ');
        if self.castling() == [0, 0] {
            fen_string.push('-');
        } else {
            if 0 != self.castling()[color::WHITE as usize].extract_bits(0, 1) {
                fen_string.push('K');
            }
            if 0 != self.castling()[color::WHITE as usize].extract_bits(1, 1) {
                fen_string.push('Q');
            }
            if 0 != self.castling()[color::BLACK as usize].extract_bits(0, 1) {
                fen_string.push('k');
            }
            if 0 != self.castling()[color::BLACK as usize].extract_bits(1, 1) {
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

        // if self.has_moves() {
        //     fen_string.push_str(" moves");
        //     for mov in &self.move_stack {
        //         fen_string.push_str(&format!(" {}{}", mov.orig().to_san_string(), mov.dest().to_san_string()));
        //     }
        // }
        fen_string
    }

    pub fn bb(&self) -> &[[Bitboard; 8]; 2] {
        &self.current().bb()
    }

    // don't actually return flipped boards for now
    pub fn bb_own(&self, color: Color) -> Bitboard {
        self.current().bb_own(color)
    }

    pub fn bb_opponent(&self, color: Color) -> Bitboard {
        self.current().bb_opponent(color)
    }

    pub fn bb_pawns(&self, color: Color) -> Bitboard {
        self.current().bb_pawns(color)
    }

    pub fn bb_knights(&self, color: Color) -> Bitboard {
        self.current().bb_knights(color)
    }

    pub fn bb_bishops(&self, color: Color) -> Bitboard {
        self.current().bb_bishops(color)
    }

    pub fn bb_rooks(&self, color: Color) -> Bitboard {
        self.current().bb_rooks(color)
    }

    pub fn bb_queens(&self, color: Color) -> Bitboard {
        self.current().bb_queens(color)
    }

    pub fn bb_king(&self, color: Color) -> Bitboard {
        self.current().bb_king(color)
    }

    pub fn bb_empty(&self) -> Bitboard {
        !(self.bb_own(color::WHITE) | self.bb_opponent(color::WHITE))
    }

    pub fn to_move(&self) -> Color {
        self.current().to_move()
    }

    pub fn castling(&self) -> [u32; 2] {
        self.current().castling()
    }

    pub fn en_passant(&self) -> Option<[Square; 2]> {
        self.current().en_passant()
    }

    pub fn occupied(&self) -> &[Piece; 64] {
        &self.current().occupied()
    }

    pub fn halfmoves(&self) -> u32 {
        self.current().halfmoves()
    }

    pub fn fullmoves(&self) -> u32 {
        self.current().fullmoves()
    }


    pub fn make_move(&mut self, mov: Move) {
        assert!(self.pcursor + 1 < PSTACK_SIZE);
        //self.pstack[self.pcursor + 1] = self.current().clone();
        self.pstack[self.pcursor + 1] = *self.current();
        self.pstack[self.pcursor + 1].make_move(mov);
        self.move_stack.push(mov);
        self.pcursor += 1;
    }

    pub fn unmake_move(&mut self) {
        assert!(self.pcursor > 0);
        self.move_stack.pop();
        self.pcursor -= 1;
    }

    pub fn input_move(&mut self,
        orig: Square,
        dest: Square,
        promote_to: Option<Piece>,
    ) -> Result<bool, &'static str> {
        assert!(self.pcursor + 1 < PSTACK_SIZE);
        //self.pstack[self.pcursor + 1] = self.current().clone();
        self.pstack[self.pcursor + 1] = *self.current();
        return match self.pstack[self.pcursor + 1].input_move(orig, dest, promote_to) {
            Ok(mov) => {
                self.move_stack.push(mov);
                self.pcursor += 1;
                Ok(true)
            }
            Err(err) => Err(err)
        }
    }

    pub fn input_san_move(&mut self, san_move: &str) -> Result<bool, &'static str> {
        if san_move.len() < 4 {
            return Err("error: incomplete move")
        }
        if let (Ok(from), Ok(to)) = (
            Square::from_san_string(&san_move[0..2]),
            Square::from_san_string(&san_move[2..4]),
        ) {
            return self.input_move(from, to, None)
        } else {
            return Err("error: invalid move")
        }
    }

    pub fn has_moves(&self) -> bool {
        !self.move_stack.is_empty()
    }

    pub fn last_move(&self) -> Option<Move> {
        if self.move_stack.is_empty() {
            None
        } else {
            Some(self.move_stack[self.move_stack.len() - 1])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use crate::color::*;
    use crate::move_generator::MoveGenerator;
    use std::error::Error;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::path::Path;

    #[test]
    fn it_sets_correct_startpos() {
        let b = Board::startpos();
        
        // color boards
        assert_eq!(0xffff, b.bb_own(WHITE));
        assert_eq!(0xffff << 6*8, b.bb_opponent(WHITE));

        // pawn boards
        assert_eq!(0xff << 8, b.bb_pawns(WHITE));
        assert_eq!(0xff << 6*8, b.bb_pawns(BLACK));
        
        // rook boards
        assert_eq!(0x81, b.bb_rooks(WHITE));
        assert_eq!(0x81 << 7*8, b.bb_rooks(BLACK));
        
        // bishop boards
        assert_eq!(0x24, b.bb_bishops(WHITE));
        assert_eq!(0x24 << 7*8, b.bb_bishops(BLACK));

        // queen boards
        assert_eq!(0x8, b.bb_queens(WHITE));
        assert_eq!(0x8 << 7*8, b.bb_queens(BLACK));

        // king boards
        assert_eq!(0x10, b.bb_king(WHITE));
        assert_eq!(0x10 << 7*8, b.bb_king(BLACK));

        assert!(!b.has_moves());
    }

    #[test]
    fn it_makes_correct_fen_strings() {
        let fen_strs = vec![
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        ];
        
        let board = Board::startpos();
        assert!(!board.has_moves());
        
        for fen_str in fen_strs {
            if let Ok(board) = Board::from_fen(String::from(fen_str)) {
                assert_eq!(fen_str, board.to_fen());
            } else {
                assert!(false);
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

        for position in BufReader::new(posfile).lines().map(|l| l.unwrap()) {
            let b = Board::from_fen(position.clone());
            match b {
                Err(e) => assert!(false, e),
                Ok(board) => assert_eq!(position, board.to_fen()),
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
            let b = Board::from_fen(String::from(fen_str));
            match b {
                Err(_e) => assert!(true),
                Ok(_board) => assert!(false),
            }
        }
    }

    #[test]
    fn it_makes_moves() {
        if let Ok(mut board) = Board::from_fen(String::from(
            "rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 1",
        )) {
            assert_eq!(0, board.move_stack.len());
            board.input_move(square::D7, square::D6, None).unwrap();
            assert_eq!(1, board.move_stack.len());
            assert_eq!(None, board.en_passant());
            let last_move = board.last_move().unwrap();
            assert_eq!(last_move.orig(), square::D7);
            assert_eq!(last_move.dest(), square::D6);
        }

        let mut board = Board::from_fen(String::from("8/3p4/8/4P/8/8/8/8 b - - 0 1")).unwrap();
        board.input_move(square::D7, square::D5, None).unwrap();
        board.input_move(square::E5, square::D6, None).unwrap();
        assert_eq!(0, board.occupied()[square::D5 as usize]);
    }

    #[test]
    fn it_unmakes_moves() {
        let one_move_strs = vec![
            ("rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 1", " moves d7d5"),
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
            if let Ok(mut board) = Board::from_fen(
                String::from(one_mover_fen) + &String::from(one_mover_moves)
            ) {
                board.unmake_move();
                assert_eq!(one_mover_fen, board.to_fen());
            } else {
                assert!(false);
            }
            
        }

        for (two_mover_fen, two_mover_moves) in two_move_strs {
            if let Ok(mut board) = Board::from_fen(
                String::from(two_mover_fen) + &String::from(two_mover_moves)
            ) {
                board.unmake_move();
                board.unmake_move();
                assert_eq!(two_mover_fen, board.to_fen());
            } else {
                assert!(false);
            }
            
        }
    }

    #[test]
    fn it_unwinds_its_move_stack() {
        {
            let fen = String::from(
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            );
            let mut board = Board::from_fen(fen.clone()).unwrap();
            let _ctx = MoveGenerator::perft(&mut board, 4);
            assert_eq!(fen, board.to_fen());
        }
        {
            let board_orig = Board::startpos();
            let mut board = board_orig.clone();
            let _ctx = MoveGenerator::perft(&mut board, 4);
        }
    }
}
