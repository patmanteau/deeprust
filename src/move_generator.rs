use std::fmt;
use board::Board;
use common::*;
use bitboard::{self, Bitboard, BitboardPrimitives};
use moves::Move;
use piece;
use color::{self, Color};
use square::{self, Square, SquarePrimitives};

use quanta::Clock;

pub struct PerftContext {
    nodes: u128,
    captures: u128,
    ep: u128,
    castles: u128,
    promotions: u128,
    checks: u128,
    disco_checks: u128,
    double_checks: u128,
    checkmates: u128,
    elapsed: u64,
}

impl PerftContext {
    pub fn new() -> PerftContext {
        PerftContext {
            nodes: 0,
            captures: 0,
            ep: 0,
            castles: 0,
            promotions: 0,
            checks: 0,
            disco_checks: 0,
            double_checks: 0,
            checkmates: 0,
            elapsed: 0,
        }
    }
}

impl fmt::Display for PerftContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Elapsed: {}s, Nodes: {}, Captures: {}, EP: {}, Castles: {}, Promos: {}, Checks: {}, Discochecks: {}, Double checks: {}, Checkmates: {}", 
                (self.elapsed as f64) / 1_000_000_000f64, self.nodes, self.captures, self.ep, self.castles, self.promotions,
                self.checks, self.disco_checks, self.double_checks, self.checkmates)
    }
}

pub struct MoveGenerator {
    
}

impl MoveGenerator {
    pub fn new() -> MoveGenerator {
        MoveGenerator{
            
        }
    }

    pub fn perft(board: &mut Board, depth: u32) -> PerftContext {
        let mut ctx = PerftContext::new();
        let clock = Clock::new();
        let start = clock.now();
        Self::do_perft(board, &mut ctx, depth);
        let finish = clock.now();
        ctx.elapsed = finish - start;
        ctx
    }

    fn do_perft(board: &mut Board, ctx: &mut PerftContext, depth: u32) {
        if depth == 0 {
            ctx.nodes += 1;
            if board.move_stack().len() > 0 {
                let mov = board.move_stack().peek().mov;
                if mov.is_capture() { ctx.captures += 1; }
                if mov.is_capture_en_passant() { ctx.ep += 1; }
                if mov.is_king_castle() || mov.is_queen_castle() { ctx.castles += 1; }
                if mov.is_promotion() { ctx.promotions += 1; }
            }
            if MoveGenerator::is_in_check(board, board.to_move()) { 
                ctx.checks += 1;
                if MoveGenerator::is_mate(board, board.to_move()) {
                    ctx.checkmates += 1;
                }
            }
            return;
        }

        let mut nodes = 0u64;
        let moves = MoveGenerator::from_board(board);
        for mov in moves.iter() {
            board.make_move(*mov);
            if !MoveGenerator::is_in_check(board, 1 ^ board.to_move()) {
                MoveGenerator::do_perft(board, ctx, depth - 1);
            }
            board.unmake_move();
        }
    }

    fn break_helper() {
        let a = 0;
    }

    #[inline]
    pub fn is_mate(board: &mut Board, color: Color) -> bool {
        let moves = MoveGenerator::from_board(board);
        for mov in moves.iter() {
            board.make_move(*mov);
            if !MoveGenerator::is_in_check(board, 1 ^ board.to_move()) {
                board.unmake_move();
                return false;
            } else {
                board.unmake_move();
            }
        }
        true
    }

    #[inline]
    pub fn is_in_check(board: &Board, color: Color) -> bool {
        let kingpos = board.bb_king(color).scan();
        MoveGenerator::is_attacked(board, color, kingpos)
    }

    #[inline]
    pub fn is_attacked(board: &Board, color: Color, target: Square) -> bool {
        let occupied = board.bb_own(color) | board.bb_opponent(color);

        if 63 < target {
            MoveGenerator::break_helper();
        }

        // by black pawns
        if color == color::WHITE {
            if 0 < 
                ((bitboard::north_west_one(bitboard::BB_SQUARES[target as usize]) |
                bitboard::north_east_one(bitboard::BB_SQUARES[target as usize])) &
               (board.bb_pawns(color::BLACK))) {
                   return true
            }
        } else { // by white pawns
            if 0 < 
               ((bitboard::south_west_one(bitboard::BB_SQUARES[target as usize]) |
                bitboard::south_east_one(bitboard::BB_SQUARES[target as usize])) &
               (board.bb_pawns(color::WHITE))) {
                   return true
            }
        }

        // by knights
        if 0 < (bitboard::BB_KNIGHT_ATTACKS[target as usize] & board.bb_knights(1 ^ color)) {
            return true;
        }

        // by king?!?
        if 0 < (bitboard::BB_KING_ATTACKS[target as usize] & board.bb_king(1 ^ color)) {
            return true
        }

        // by bishops
        if 0 < ((bitboard::diagonal_attacks(target, occupied) | bitboard::anti_diagonal_attacks(target, occupied)) 
                & board.bb_bishops(1 ^ color)) {
            return true
        }

        // by rooks
        if 0 < ((
                bitboard::rank_attacks(target, occupied) | 
                bitboard::file_attacks(target, occupied)
        ) & board.bb_rooks(1 ^ color)) {
            return true
        }

        // by queens
        if 0 < ((
                bitboard::diagonal_attacks(target, occupied) | 
                bitboard::anti_diagonal_attacks(target, occupied) |
                bitboard::rank_attacks(target, occupied) | 
                bitboard::file_attacks(target, occupied)
        ) & board.bb_queens(1 ^ color)) {
            return true
        }

        false
    }
    
    #[inline]
    pub fn from_board(board: &Board) -> Vec<Move> {
        // self.moves = Vec::with_capacity(512);
        let mut moves = Vec::with_capacity(512);
        let to_move = board.to_move();

        if to_move == color::WHITE {
            let w_pawn_pushes = MoveGenerator::gen_white_pawn_pushes(board);
            moves.extend(w_pawn_pushes);

            let w_pawn_captures = MoveGenerator::gen_white_pawn_captures(board);
            moves.extend(w_pawn_captures);

            let w_king_castles = MoveGenerator::gen_wking_castle(board);
            moves.extend(w_king_castles);

        } else {
            let b_pawn_pushes = MoveGenerator::gen_black_pawn_pushes(board);
            moves.extend(b_pawn_pushes);

            let b_pawn_captures = MoveGenerator::gen_black_pawn_captures(board);
            moves.extend(b_pawn_captures);

            let b_king_castles = MoveGenerator::gen_bking_castle(board);
            moves.extend(b_king_castles);
        }

        let knight_captures = MoveGenerator::gen_knight_captures(board, to_move);
        moves.extend(knight_captures);
        let knight_moves = MoveGenerator::gen_knight_moves(board, to_move);
        moves.extend(knight_moves);

        let bishop_captures = MoveGenerator::gen_bishop_captures(board, to_move);
        moves.extend(bishop_captures);
        let bishop_moves = MoveGenerator::gen_bishop_moves(board, to_move);
        moves.extend(bishop_moves);

        let rook_captures = MoveGenerator::gen_rook_captures(board, to_move);
        moves.extend(rook_captures);
        let rook_moves = MoveGenerator::gen_rook_moves(board, to_move);
        moves.extend(rook_moves);

        let queen_captures = MoveGenerator::gen_queen_captures(board, to_move);
        moves.extend(queen_captures);
        let queen_moves = MoveGenerator::gen_queen_moves(board, to_move);
        moves.extend(queen_moves);

        let king_captures = MoveGenerator::gen_king_captures(board, to_move);
        moves.extend(king_captures);
        let king_moves = MoveGenerator::gen_king_moves(board, to_move);
        moves.extend(king_moves);

        moves
    }

    // pub fn moves(&self) -> &Vec<Move> {
    //     &self.moves
    // }

    // pub fn count(&self) -> usize {
    //     self.moves.len()
    // }

    #[inline]
    fn gen_white_pawn_pushes(board: &Board) -> Vec<Move> {
        let mut pawns = board.bb_pawns(color::WHITE);
        let mut moves = Vec::with_capacity(16);
        
        while 0 != pawns {
            let from = pawns.scan();
            let single_push = bitboard::north_one(bitboard::BB_SQUARES[from as usize]) & board.bb_empty();
            let double_push = bitboard::north_one(single_push) & board.bb_empty() & bitboard::BB_RANK_4;

            if 0 != single_push {
                let to = single_push.scan();
                if to >= 56 {
                    // promotion
                    moves.push(Move::new(from, to, Move::make_flags(false, true, true, true))); // queen
                    moves.push(Move::new(from, to, Move::make_flags(false, true, false, true))); // rook
                    moves.push(Move::new(from, to, Move::make_flags(false, true, true, false))); // bishop
                    moves.push(Move::new(from, to, Move::make_flags(false, true, false, false))); // knight
                } else {
                    moves.push(Move::new(from, to, Move::make_flags(false, false, false, false)));
                }
            }

            if 0 != double_push {
                let to = double_push.scan() as u16;
                moves.push(Move::new(from, to, Move::make_flags(false, false, true, false)));
            }            
            pawns.clear(from);
        }
        moves
    }

    #[inline]
    fn gen_black_pawn_pushes(board: &Board) -> Vec<Move> {
        let mut pawns = board.bb_pawns(color::BLACK);
        let mut moves = Vec::with_capacity(16);
        
        while 0 != pawns {
            let from = pawns.scan();
            let single_push = bitboard::south_one(bitboard::BB_SQUARES[from as usize]) & board.bb_empty();
            let double_push = bitboard::south_one(single_push) & board.bb_empty() & bitboard::BB_RANK_5;

            if 0 != single_push {
                let to = single_push.scan();
                if to < 8 {
                    // promotion
                    moves.push(Move::new(from, to, Move::make_flags(false, true, true, true))); // queen
                    moves.push(Move::new(from, to, Move::make_flags(false, true, false, true))); // rook
                    moves.push(Move::new(from, to, Move::make_flags(false, true, true, false))); // bishop
                    moves.push(Move::new(from, to, Move::make_flags(false, true, false, false))); // knight
                } else {
                    moves.push(Move::new(from, to, Move::make_flags(false, false, false, false)));
                }
            }

            if 0 != double_push {
                let to = double_push.scan();
                moves.push(Move::new(from, to, Move::make_flags(false, false, true, false)));
            }            
            pawns.clear(from);
        }
        moves
    }

    #[inline]
    fn gen_white_pawn_captures(board: &Board) -> Vec<Move> {
        let mut pawns = board.bb_pawns(color::WHITE);
        let mut moves = Vec::with_capacity(16);
        let mut ep_bb = bitboard::BB_EMPTY;
        let mut ep_square = 0;
        
        match board.en_passant() {
            Some(ep_squares) => {
                ep_square = ep_squares[0];
                ep_bb = bitboard::BB_SQUARES[ep_square as usize];
            },
            None => ()
        }

        while 0 != pawns {
            let from = pawns.scan();
            let mut atk = 
                (bitboard::north_west_one(bitboard::BB_SQUARES[from as usize]) |
                 bitboard::north_east_one(bitboard::BB_SQUARES[from as usize])) &
                (board.bb_opponent(color::WHITE) | ep_bb);

            while 0 != atk {
                let to = atk.scan();
                if to >= 56 {
                    // promotion
                    moves.push(Move::new(from, to, Move::make_flags(true, true, true, true))); // queen
                    moves.push(Move::new(from, to, Move::make_flags(true, true, false, true))); // rook
                    moves.push(Move::new(from, to, Move::make_flags(true, true, true, false))); // bishop
                    moves.push(Move::new(from, to, Move::make_flags(true, true, false, false))); // knight
                } else {
                    moves.push(Move::new(from, to, Move::make_flags(true, false, to == ep_square, false)));
                }
                atk.clear(to);
            }
            pawns.clear(from);
        }

        moves
    }

    #[inline]
    fn gen_black_pawn_captures(board: &Board) -> Vec<Move> {
        let mut pawns = board.bb_pawns(color::BLACK);
        let mut moves = Vec::with_capacity(16);
        let mut ep_bb = bitboard::BB_EMPTY;
        let mut ep_square = 0;
        
        match board.en_passant() {
            Some(ep_squares) => {
                ep_square = ep_squares[0];
                // ep_bb = bitboard::BB_SQUARES[squares::EP_CAPTURE_SQUARES[ep_square as usize]];
                ep_bb = bitboard::BB_SQUARES[ep_square as usize];
            },
            None => ()
        }

        while 0 != pawns {
            let from = pawns.scan();
            let mut atk = 
                (bitboard::south_west_one(bitboard::BB_SQUARES[from as usize]) |
                 bitboard::south_east_one(bitboard::BB_SQUARES[from as usize])) &
                (board.bb_opponent(color::BLACK) | ep_bb);

            while 0 != atk {
                let to = atk.scan();
                if to < 8 {
                    // promotion
                    moves.push(Move::new(from, to, Move::make_flags(true, true, true, true))); // queen
                    moves.push(Move::new(from, to, Move::make_flags(true, true, false, true))); // rook
                    moves.push(Move::new(from, to, Move::make_flags(true, true, true, false))); // bishop
                    moves.push(Move::new(from, to, Move::make_flags(true, true, false, false))); // knight
                } else {
                    moves.push(Move::new(from, to, Move::make_flags(true, false, to == ep_square, false)));
                }
                atk.clear(to);
            }
            pawns.clear(from);
        }

        moves
    }

    #[inline]
    fn gen_knight_moves(board: &Board, color: Color) -> Vec<Move> {
        let mut moves = Vec::with_capacity(16);
        let mut knights = board.bb_knights(color);

        while 0 != knights {
            let from = knights.scan();
            let mut mov = bitboard::BB_KNIGHT_ATTACKS[from as usize] & board.bb_empty();

            while 0 != mov {
                let to = mov.scan();
                moves.push(Move::new(from, to, Move::make_flags(false, false, false, false)));
                mov.clear(to);
            }
            knights.clear(from);
        }
        moves
    }

    #[inline]
    fn gen_knight_captures(board: &Board, color: Color) -> Vec<Move> {
        let mut moves = Vec::with_capacity(16);
        let mut knights = board.bb_knights(color);

        while 0 != knights {
            let from = knights.scan();
            let mut atk = bitboard::BB_KNIGHT_ATTACKS[from as usize] & board.bb_opponent(color);

            while 0 != atk {
                let to = atk.scan();
                moves.push(Move::new(from, to, Move::make_flags(true, false, false, false)));
                atk.clear(to);
            }
            knights.clear(from);
        }
        moves
    }

    #[inline]
    fn gen_wking_castle(board: &Board) -> Vec<Move> {
        let occ = board.bb_own(color::WHITE) | board.bb_opponent(color::WHITE);
        let mut moves = Vec::with_capacity(16);
        
        if MoveGenerator::is_attacked(board, color::WHITE, square::E1) {
            return moves
        }

        let qclear = 
            !occ.test(square::D1) && !occ.test(square::C1) &&
            !occ.test(square::B1) && !MoveGenerator::is_attacked(board, color::WHITE, square::D1) &&
            board.castling()[color::WHITE as usize].test_bit(1);

        let kclear = 
            !occ.test(square::F1) && !occ.test(square::G1) &&
            !MoveGenerator::is_attacked(board, color::WHITE, square::F1) &&
            board.castling()[color::WHITE as usize].test_bit(0);

        if qclear {
            moves.push(Move::new(square::E1, square::C1, Move::make_flags(false, false, true, true)));
        }
        if kclear {
            moves.push(Move::new(square::E1, square::G1, Move::make_flags(false, false, false, true)));
        }
        moves
    }

    #[inline]
    fn gen_bking_castle(board: &Board) -> Vec<Move> {
        let occ = board.bb_own(color::BLACK) | board.bb_opponent(color::BLACK);
        let mut moves = Vec::with_capacity(16);

        if MoveGenerator::is_attacked(board, color::BLACK, square::E8) {
            return moves
        }

        let qclear = 
            !occ.test(square::D8) && !occ.test(square::C8) &&
            !occ.test(square::B8) && !MoveGenerator::is_attacked(board, color::BLACK, square::D8) &&
            board.castling()[color::BLACK as usize].test_bit(1);

        let kclear = 
            !occ.test(square::F8) && !occ.test(square::G8) &&
            !MoveGenerator::is_attacked(board, color::BLACK, square::F8) &&
            board.castling()[color::BLACK as usize].test_bit(0);

        if qclear {
            moves.push(Move::new(square::E8, square::C8, Move::make_flags(false, false, true, true)));
        }
        if kclear {
            moves.push(Move::new(square::E8, square::G8, Move::make_flags(false, false, false, true)));
        }
        moves
    }

    #[inline]
    fn gen_king_moves(board: &Board, color: Color) -> Vec<Move> {
        let mut moves = Vec::with_capacity(16);
        let mut king = board.bb_king(color);

        while 0 != king {
            let from = king.scan();
            let mut mov = bitboard::BB_KING_ATTACKS[from as usize] & board.bb_empty();

            while 0 != mov {
                let to = mov.scan();
                moves.push(Move::new(from, to, Move::make_flags(false, false, false, false)));
                mov.clear(to);
            }
            king.clear(from);
        }
        moves
    }

    #[inline]
    fn gen_king_captures(board: &Board, color: Color) -> Vec<Move> {
        let mut moves = Vec::with_capacity(16);
        let mut king = board.bb_king(color);

        while 0 != king {
            let from = king.scan();
            let mut atk = bitboard::BB_KING_ATTACKS[from as usize] & board.bb_opponent(color);

            while 0 != atk {
                let to = atk.scan();
                moves.push(Move::new(from, to, Move::make_flags(true, false, false, false)));
                atk.clear(to);
            }
            king.clear(from);
        }
        moves
    }

    #[inline]
    fn gen_bishop_moves(board: &Board, color: Color) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        let mut bishops = board.bb_bishops(color);
        let occupied = board.bb_own(color) | board.bb_opponent(color);

        while 0 != bishops {
            let from = bishops.scan();
            let mut mov = (bitboard::diagonal_attacks(from, occupied) | bitboard::anti_diagonal_attacks(from, occupied)) & board.bb_empty();

            while 0 != mov {
                let to = mov.scan();
                moves.push(Move::new(from, to, Move::make_flags(false, false, false, false)));
                mov.clear(to);
            }
            bishops.clear(from);
        }
        moves
    }

    #[inline]
    fn gen_bishop_captures(board: &Board, color: Color) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        let mut bishops = board.bb_bishops(color);
        let occupied = board.bb_own(color) | board.bb_opponent(color);

        while 0 != bishops {
            let from = bishops.scan();
            let mut atk = (bitboard::diagonal_attacks(from, occupied) | bitboard::anti_diagonal_attacks(from, occupied)) & board.bb_opponent(color);

            while 0 != atk {
                let to = atk.scan();
                moves.push(Move::new(from, to, Move::make_flags(true, false, false, false)));
                atk.clear(to);
            }
            bishops.clear(from);
        }
        moves
    }

    #[inline]
    fn gen_rook_moves(board: &Board, color: Color) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        let mut rooks = board.bb_rooks(color);
        let occupied = board.bb_own(color) | board.bb_opponent(color);

        while 0 != rooks {
            let from = rooks.scan() ;
            let mut mov = (bitboard::rank_attacks(from, occupied) | bitboard::file_attacks(from, occupied)) & board.bb_empty();
            // let mut mov = (bitboard::rank_attacks(from, occupied)) & board.bb_empty();

            while 0 != mov {
                let to = mov.scan();
                moves.push(Move::new(from, to, Move::make_flags(false, false, false, false)));
                mov.clear(to);
            }
            rooks.clear(from);
        }
        moves
    }

    #[inline]
    fn gen_rook_captures(board: &Board, color: Color) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        let mut rooks = board.bb_rooks(color);
        let occupied = board.bb_own(color) | board.bb_opponent(color);

        while 0 != rooks {
            let from = rooks.scan();
            let mut atk = (bitboard::rank_attacks(from, occupied) | bitboard::file_attacks(from, occupied)) & board.bb_opponent(color);
            // let mut atk = (bitboard::rank_attacks(from, occupied)) & board.bb_opponent(color);

            while 0 != atk {
                let to = atk.scan();
                moves.push(Move::new(from, to, Move::make_flags(true, false, false, false)));
                atk.clear(to);
            }
            rooks.clear(from);
        }
        moves
    }

    #[inline]
    fn gen_queen_moves(board: &Board, color: Color) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        let mut queens = board.bb_queens(color);
        let occupied = board.bb_own(color) | board.bb_opponent(color);

        while 0 != queens {
            let from = queens.scan();
            let mut mov = board.bb_empty() & (
                bitboard::rank_attacks(from, occupied) |
                bitboard::file_attacks(from, occupied) |
                bitboard::diagonal_attacks(from, occupied) |
                bitboard::anti_diagonal_attacks(from, occupied)
            );
            while 0 != mov {
                let to = mov.scan();
                moves.push(Move::new(from, to, Move::make_flags(false, false, false, false)));
                mov.clear(to);
            }
            queens.clear(from);
        }
        moves
    }

    #[inline]
    fn gen_queen_captures(board: &Board, color: Color) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        let mut queens = board.bb_queens(color);
        let occupied = board.bb_own(color) | board.bb_opponent(color);

        while 0 != queens {
            let from = queens.scan();
            let mut atk = board.bb_opponent(color) & (
                bitboard::rank_attacks(from, occupied) |
                bitboard::file_attacks(from, occupied) |
                bitboard::diagonal_attacks(from, occupied) |
                bitboard::anti_diagonal_attacks(from, occupied)
            );

            while 0 != atk {
                let to = atk.scan();
                moves.push(Move::new(from, to, Move::make_flags(true, false, false, false)));
                atk.clear(to);
            }
            queens.clear(from);
        }
        moves
    }
}

#[cfg(test)]
mod tests {
    use uci::UCIInterface;
    use move_generator::MoveGenerator;
    use board::Board;
    use color;
    use square;

    #[test]
    fn it_generates_pawn_moves() {
        let mut board = Board::startpos();
        
        assert_eq!(16, MoveGenerator::gen_white_pawn_pushes(&board).len());
        board.input_move(square::E2, square::E4, None);
        assert_eq!(16, MoveGenerator::gen_black_pawn_pushes(&board).len());

        board = Board::from_fen(String::from("8/PPPPPPPP/8/8/8/8/8/8 w - - 0 1")).unwrap();
        assert_eq!(32, MoveGenerator::gen_white_pawn_pushes(&board).len());
        board = Board::from_fen(String::from("8/8/8/8/8/8/pppppppp/8 b - - 0 1")).unwrap();
        assert_eq!(32, MoveGenerator::gen_black_pawn_pushes(&board).len());
    }

    #[test]
    fn it_generates_pawn_captures() {
        let mut gen = MoveGenerator::new();
        let mut board = Board::from_fen(String::from("8/8/8/p1p1p1p1/P1P1P1P1/8/8/8 w - - 0 1")).unwrap();
        assert_eq!(0, MoveGenerator::gen_white_pawn_captures(&board).len());

        board = Board::from_fen(String::from("1k6/8/8/p1p1p1p1/1P1P1P1P/8/8/K7 w - - 0 1")).unwrap();
        assert_eq!(7, MoveGenerator::gen_white_pawn_captures(&board).len());
        assert_eq!(4, MoveGenerator::gen_white_pawn_pushes(&board).len());
        // assert_eq!(11, MoveGenerator::from_board(&board).len());

        board = Board::from_fen(String::from("1k6/3p4/8/4P/8/8/8/6K1 b - - 0 1")).unwrap();
        board.input_move(square::D7, square::D5, None);
        assert_eq!(1, MoveGenerator::gen_white_pawn_captures(&board).len());
        assert_eq!(1, MoveGenerator::gen_white_pawn_pushes(&board).len());
        // assert_eq!(2, MoveGenerator::from_board(&board).len());
    }

    #[test]
    fn it_generates_king_moves() {
        // position startpos moves e2e4 e7e5 g1f3 g8f6 f1d3 f8d6
        let mut board = Board::startpos();
        board.input_move(square::E2, square::E4, None);
        board.input_move(square::E7, square::E5, None);
        board.input_move(square::G1, square::F3, None);
        board.input_move(square::G8, square::F6, None);
        board.input_move(square::F1, square::D3, None);
        board.input_move(square::F8, square::D6, None);
        assert_eq!(2, MoveGenerator::gen_king_moves(&board, color::WHITE).len());
        assert_eq!(1, MoveGenerator::gen_wking_castle(&board).len());
    }

    #[test]
    fn it_generates_castles() {
        let mut c = UCIInterface::new();
        c.parse(String::from("position startpos moves e2e4 d7d5 g1f3 b8c6 f1e2 c8e6 e1g1 d8d6 d2d4"));
        assert_eq!(38, MoveGenerator::from_board(&c.board).len());
    }

    #[test]
    fn it_generates_all_moves() {
        // see https://chessprogramming.wikispaces.com/Perft+Results
        // Position 1(startpos) perft(1)
        let mut board = Board::startpos();
        assert_eq!(20, MoveGenerator::from_board(&board).len());

        // Position 2(kiwipete) perft(1)
        board = Board::from_fen(String::from("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")).unwrap();
        assert_eq!(48, MoveGenerator::from_board(&board).len());

        // // Position 3 perft(1)
        // board = Board::from_fen(String::from("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1")).unwrap();
        // assert_eq!(14, MoveGenerator::from_board(&board).len());

        // // Position 4 perft(1)
        // board = Board::from_fen(String::from("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")).unwrap();
        // assert_eq!(6, MoveGenerator::from_board(&board).len());

        // // Position 5 perft(1)
        // board = Board::from_fen(String::from("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")).unwrap();
        // assert_eq!(44, MoveGenerator::from_board(&board).len());

        // Position 6 perft(1)
        board = Board::from_fen(String::from("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10")).unwrap();
        MoveGenerator::from_board(&board);
        assert_eq!(46, MoveGenerator::from_board(&board).len());
    }
}