use crate::bitboard::{self, BitboardPrimitives};
use crate::board::Board;
use crate::common::*;
use crate::moves::flags;
use crate::moves::Move;
use std::fmt;

use crate::color::{self, Color};
use crate::square::{self, Square};

use quanta::Clock;

#[derive(Default)]
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

// #[derive(Default)]
// pub struct MoveGenerator {}
pub trait MoveGenerator {
    fn generate_moves(&self) -> Vec<Move>;
    fn perft(&mut self, depth: u32) -> PerftContext;
    fn do_perft(&mut self, ctx: &mut PerftContext, depth: u32);
    fn is_mate(&mut self, _color: Color) -> bool;
    fn is_in_check(&self, color: Color) -> bool;
    fn is_attacked(&self, color: Color, target: Square) -> bool;

    fn gen_white_pawn_pushes(&self) -> Vec<Move>;
    fn gen_black_pawn_pushes(&self) -> Vec<Move>;
    fn gen_white_pawn_captures(&self) -> Vec<Move>;
    fn gen_black_pawn_captures(&self) -> Vec<Move>;
    fn gen_knight_moves(&self, color: Color) -> Vec<Move>;

    fn gen_knight_captures(&self, color: Color) -> Vec<Move>;
    fn gen_wking_castle(&self) -> Vec<Move>;
    fn gen_bking_castle(&self) -> Vec<Move>;
    fn gen_king_moves(&self, color: Color) -> Vec<Move>;
    fn gen_king_captures(&self, color: Color) -> Vec<Move>;
    fn gen_bishop_moves(&self, color: Color) -> Vec<Move>;
    fn gen_bishop_captures(&self, color: Color) -> Vec<Move>;
    fn gen_rook_moves(&self, color: Color) -> Vec<Move>;
    fn gen_rook_captures(&self, color: Color) -> Vec<Move>;
    fn gen_queen_moves(&self, color: Color) -> Vec<Move>;
    fn gen_queen_captures(&self, color: Color) -> Vec<Move>;
}

impl MoveGenerator for Board {
    fn perft(&mut self, depth: u32) -> PerftContext {
        let mut ctx = PerftContext::new();
        let clock = Clock::new();
        let start = clock.now();
        self.do_perft(&mut ctx, depth);
        let finish = clock.now();
        ctx.elapsed = finish - start;
        ctx
    }

    fn do_perft(&mut self, ctx: &mut PerftContext, depth: u32) {
        if depth == 0 {
            ctx.nodes += 1;
            if !self.history().is_empty() {
                let mov = self.history().last().unwrap();
                if mov.is_capture() {
                    ctx.captures += 1;
                }
                if mov.is_capture_en_passant() {
                    ctx.ep += 1;
                }
                if mov.is_king_castle() || mov.is_queen_castle() {
                    ctx.castles += 1;
                }
                if mov.is_promotion() {
                    ctx.promotions += 1;
                }
            }
            let to_move = self.to_move();
            if self.is_in_check(to_move) {
                ctx.checks += 1;
                if self.is_mate(to_move) {
                    ctx.checkmates += 1;
                }
            }
            return;
        }

        //let mut nodes = 0u64;
        let moves = self.generate_moves();
        for mov in moves.iter() {
            self.make_move(*mov);
            if !self.is_in_check(1 ^ self.to_move()) {
                self.do_perft(ctx, depth - 1);
            }
            self.unmake_move();
        }
    }

    fn is_mate(&mut self, _color: Color) -> bool {
        let moves = self.generate_moves();
        for mov in moves.iter() {
            self.make_move(*mov);
            if !self.is_in_check(1 ^ self.to_move()) {
                self.unmake_move();
                return false;
            } else {
                self.unmake_move();
            }
        }
        true
    }

    fn is_in_check(&self, color: Color) -> bool {
        let kingpos = self.bb_king(color).scan();
        self.is_attacked(color, kingpos)
    }

    fn is_attacked(&self, color: Color, target: Square) -> bool {
        let occupied = self.bb_own(color) | self.bb_opponent(color);

        assert!(target < 64);

        // by black pawns
        if color == color::WHITE {
            if 0 < bitboard::BB_WPAWN_ATTACKS[target as usize] & self.bb_pawns(color::BLACK) {
                return true;
            }
        } else if 0 < bitboard::BB_BPAWN_ATTACKS[target as usize] & self.bb_pawns(color::WHITE) {
            return true;
        }

        // by knights
        if 0 < (bitboard::BB_KNIGHT_ATTACKS[target as usize] & self.bb_knights(1 ^ color)) {
            return true;
        }

        // by king?!?
        if 0 < (bitboard::BB_KING_ATTACKS[target as usize] & self.bb_king(1 ^ color)) {
            return true;
        }

        let all_diag_attacks = bitboard::diagonal_attacks(target, occupied)
            | bitboard::anti_diagonal_attacks(target, occupied);
        let rank_file_attacks =
            bitboard::rank_attacks(target, occupied) | bitboard::file_attacks(target, occupied);

        // by bishops or queens
        if 0 < all_diag_attacks & (self.bb_bishops(1 ^ color) | self.bb_queens(1 ^ color)) {
            return true;
        }

        // by rooks or queens
        if 0 < rank_file_attacks & (self.bb_rooks(1 ^ color) | self.bb_queens(1 ^ color)) {
            return true;
        }

        false
    }

    fn generate_moves(&self) -> Vec<Move> {
        // self.moves = Vec::with_capacity(512);
        let mut moves = Vec::with_capacity(512);
        let to_move = self.to_move();

        if to_move == color::WHITE {
            let w_pawn_pushes = self.gen_white_pawn_pushes();
            moves.extend(w_pawn_pushes);

            let w_pawn_captures = self.gen_white_pawn_captures();
            moves.extend(w_pawn_captures);

            let w_king_castles = self.gen_wking_castle();
            moves.extend(w_king_castles);
        } else {
            let b_pawn_pushes = self.gen_black_pawn_pushes();
            moves.extend(b_pawn_pushes);

            let b_pawn_captures = self.gen_black_pawn_captures();
            moves.extend(b_pawn_captures);

            let b_king_castles = self.gen_bking_castle();
            moves.extend(b_king_castles);
        }

        let knight_captures = self.gen_knight_captures(to_move);
        moves.extend(knight_captures);
        let knight_moves = self.gen_knight_moves(to_move);
        moves.extend(knight_moves);

        let bishop_captures = self.gen_bishop_captures(to_move);
        moves.extend(bishop_captures);
        let bishop_moves = self.gen_bishop_moves(to_move);
        moves.extend(bishop_moves);

        let rook_captures = self.gen_rook_captures(to_move);
        moves.extend(rook_captures);
        let rook_moves = self.gen_rook_moves(to_move);
        moves.extend(rook_moves);

        let queen_captures = self.gen_queen_captures(to_move);
        moves.extend(queen_captures);
        let queen_moves = self.gen_queen_moves(to_move);
        moves.extend(queen_moves);

        let king_captures = self.gen_king_captures(to_move);
        moves.extend(king_captures);
        let king_moves = self.gen_king_moves(to_move);
        moves.extend(king_moves);

        moves
    }

    // fn moves(&self) -> &Vec<Move> {
    //     &self.moves
    // }

    // fn count(&self) -> usize {
    //     self.moves.len()
    // }

    fn gen_white_pawn_pushes(&self) -> Vec<Move> {
        let pawns = self.bb_pawns(color::WHITE);
        let mut moves = Vec::with_capacity(16);

        let mut norm_pawns = pawns & bitboard::BB_NOT_RANK_78;
        let mut prom_pawns = pawns & bitboard::BB_RANK_7;

        for from in prom_pawns.iter() {
            for to in (bitboard::BB_WPAWN_PUSHES[from as usize] & self.bb_empty()).iter() {
                // promotion
                let mov = Move::new(from, to, flags::MOV_QUIET);
                moves.push(mov.with_flags(flags::MOV_PROM_QUEEN));
                moves.push(mov.with_flags(flags::MOV_PROM_ROOK));
                moves.push(mov.with_flags(flags::MOV_PROM_BISHOP));
                moves.push(mov.with_flags(flags::MOV_PROM_KNIGHT));
            }
        }

        for from in norm_pawns.iter() {
            let mut single_push =
                bitboard::north_one(bitboard::BB_SQUARES[from as usize]) & self.bb_empty();
            let mut double_push =
                bitboard::north_one(single_push) & self.bb_empty() & bitboard::BB_RANK_4;
            for to in single_push.iter() {
                moves.push(Move::new(from, to, flags::MOV_QUIET));
            }
            for to in double_push.iter() {
                moves.push(Move::new(from, to, flags::MOV_DPP));
            }
        }

        moves
    }

    fn gen_black_pawn_pushes(&self) -> Vec<Move> {
        let pawns = self.bb_pawns(color::BLACK);
        let mut moves = Vec::with_capacity(16);

        let mut norm_pawns = pawns & bitboard::BB_NOT_RANK_12;
        let mut prom_pawns = pawns & bitboard::BB_RANK_2;

        for from in prom_pawns.iter() {
            //let from = prom_pawns.scan();
            for to in (bitboard::BB_BPAWN_PUSHES[from as usize] & self.bb_empty()).iter() {
                // promotion
                let mov = Move::new(from, to, flags::MOV_QUIET);
                moves.push(mov.with_flags(flags::MOV_PROM_QUEEN));
                moves.push(mov.with_flags(flags::MOV_PROM_ROOK));
                moves.push(mov.with_flags(flags::MOV_PROM_BISHOP));
                moves.push(mov.with_flags(flags::MOV_PROM_KNIGHT));
            }
        }

        for from in norm_pawns.iter() {
            let mut single_push =
                bitboard::south_one(bitboard::BB_SQUARES[from as usize]) & self.bb_empty();
            let mut double_push =
                bitboard::south_one(single_push) & self.bb_empty() & bitboard::BB_RANK_5;
            for to in single_push.iter() {
                moves.push(Move::new(from, to, flags::MOV_QUIET));
            }
            for to in double_push.iter() {
                moves.push(Move::new(from, to, flags::MOV_DPP));
            }
        }

        moves
    }

    fn gen_white_pawn_captures(&self) -> Vec<Move> {
        let pawns = self.bb_pawns(color::WHITE);
        let mut moves = Vec::with_capacity(16);
        let mut ep_bb = bitboard::BB_EMPTY;
        let mut ep_square = 0;

        if let Some(ep_squares) = self.en_passant() {
            ep_square = ep_squares[0];
            ep_bb = bitboard::BB_SQUARES[ep_square as usize];
        }

        let mut norm_pawns = pawns & bitboard::BB_NOT_RANK_78;
        let mut prom_pawns = pawns & bitboard::BB_RANK_7;

        for from in prom_pawns.iter() {
            // let from = prom_pawns.scan();
            let mut atk =
                bitboard::BB_WPAWN_ATTACKS[from as usize] & self.bb_opponent(color::WHITE);
            for to in atk.iter() {
                // promotion
                let mov = Move::new(from, to, flags::MOV_QUIET);
                moves.push(mov.with_flags(flags::MOV_CAPTURE | flags::MOV_PROM_QUEEN));
                moves.push(mov.with_flags(flags::MOV_CAPTURE | flags::MOV_PROM_ROOK));
                moves.push(mov.with_flags(flags::MOV_CAPTURE | flags::MOV_PROM_BISHOP));
                moves.push(mov.with_flags(flags::MOV_CAPTURE | flags::MOV_PROM_KNIGHT));
            }
        }

        for from in norm_pawns.iter() {
            let mut atk = bitboard::BB_WPAWN_ATTACKS[from as usize]
                & (self.bb_opponent(color::WHITE) | ep_bb);
            for to in atk.iter() {
                moves.push(Move::new(
                    from,
                    to,
                    Move::make_flags(true, false, to == ep_square, false),
                ));
            }
        }
        moves
    }

    fn gen_black_pawn_captures(&self) -> Vec<Move> {
        let pawns = self.bb_pawns(color::BLACK);
        let mut moves = Vec::with_capacity(16);
        let mut ep_bb = bitboard::BB_EMPTY;
        let mut ep_square = 0;

        if let Some(ep_squares) = self.en_passant() {
            ep_square = ep_squares[0];
            ep_bb = bitboard::BB_SQUARES[ep_square as usize];
        }

        let mut norm_pawns = pawns & bitboard::BB_NOT_RANK_12;
        let mut prom_pawns = pawns & bitboard::BB_RANK_2;

        for from in prom_pawns.iter() {
            let mut atk =
                bitboard::BB_BPAWN_ATTACKS[from as usize] & self.bb_opponent(color::BLACK);
            for to in atk.iter() {
                // promotion
                let mov = Move::new(from, to, flags::MOV_QUIET);
                moves.push(mov.with_flags(flags::MOV_CAPTURE | flags::MOV_PROM_QUEEN));
                moves.push(mov.with_flags(flags::MOV_CAPTURE | flags::MOV_PROM_ROOK));
                moves.push(mov.with_flags(flags::MOV_CAPTURE | flags::MOV_PROM_BISHOP));
                moves.push(mov.with_flags(flags::MOV_CAPTURE | flags::MOV_PROM_KNIGHT));
            }
        }

        for from in norm_pawns.iter() {
            let mut atk = bitboard::BB_BPAWN_ATTACKS[from as usize]
                & (self.bb_opponent(color::BLACK) | ep_bb);
            for to in atk.iter() {
                moves.push(Move::new(
                    from,
                    to,
                    Move::make_flags(true, false, to == ep_square, false),
                ));
            }
        }
        moves
    }

    fn gen_knight_moves(&self, color: Color) -> Vec<Move> {
        let mut moves = Vec::with_capacity(16);
        let mut knights = self.bb_knights(color);

        for from in knights.iter() {
            let mut atk = bitboard::BB_KNIGHT_ATTACKS[from as usize] & self.bb_empty();

            for to in atk.iter() {
                moves.push(Move::new(from, to, flags::MOV_QUIET));
            }
        }
        moves
    }

    fn gen_knight_captures(&self, color: Color) -> Vec<Move> {
        let mut moves = Vec::with_capacity(16);
        let mut knights = self.bb_knights(color);

        for from in knights.iter() {
            let mut atk = bitboard::BB_KNIGHT_ATTACKS[from as usize] & self.bb_opponent(color);

            for to in atk.iter() {
                moves.push(Move::new(from, to, flags::MOV_CAPTURE));
            }
        }
        moves
    }

    fn gen_wking_castle(&self) -> Vec<Move> {
        let occ = self.bb_own(color::WHITE) | self.bb_opponent(color::WHITE);
        let mut moves = Vec::with_capacity(16);

        if self.is_attacked(color::WHITE, square::E1) {
            return moves;
        }

        let qlear = 0 == occ.extract_bits(u32::from(square::B1), 3)
            && !self.is_attacked(color::WHITE, square::C1)
            && !self.is_attacked(color::WHITE, square::D1)
            && self.castling()[color::WHITE as usize].test_bit(1);

        let klear = 0 == occ.extract_bits(u32::from(square::F1), 2)
            && !self.is_attacked(color::WHITE, square::F1)
            && self.castling()[color::WHITE as usize].test_bit(0);

        if qlear {
            moves.push(Move::new(square::E1, square::C1, flags::MOV_Q_CASTLE));
        }
        if klear {
            moves.push(Move::new(square::E1, square::G1, flags::MOV_K_CASTLE));
        }
        moves
    }

    fn gen_bking_castle(&self) -> Vec<Move> {
        let occ = self.bb_own(color::BLACK) | self.bb_opponent(color::BLACK);
        let mut moves = Vec::with_capacity(16);

        if self.is_attacked(color::BLACK, square::E8) {
            return moves;
        }

        let qlear = 0 == occ.extract_bits(u32::from(square::B8), 3)
            && !self.is_attacked(color::BLACK, square::C8)
            && !self.is_attacked(color::BLACK, square::D8)
            && self.castling()[color::BLACK as usize].test_bit(1);

        let klear = 0 == occ.extract_bits(u32::from(square::F8), 2)
            && !self.is_attacked(color::BLACK, square::F8)
            && self.castling()[color::BLACK as usize].test_bit(0);

        if qlear {
            moves.push(Move::new(square::E8, square::C8, flags::MOV_Q_CASTLE));
        }
        if klear {
            moves.push(Move::new(square::E8, square::G8, flags::MOV_K_CASTLE));
        }
        moves
    }

    fn gen_king_moves(&self, color: Color) -> Vec<Move> {
        let mut moves = Vec::with_capacity(16);
        
        let from = self.bb_king(color).scan();
        let mut atk = bitboard::BB_KING_ATTACKS[from as usize] & self.bb_empty();

        for to in atk.iter() {
            moves.push(Move::new(from, to, flags::MOV_QUIET));
        }
        moves
    }

    fn gen_king_captures(&self, color: Color) -> Vec<Move> {
        let mut moves = Vec::with_capacity(16);
        
        let from = self.bb_king(color).scan();
        let mut atk = bitboard::BB_KING_ATTACKS[from as usize] & self.bb_opponent(color);

        for to in atk.iter() {
            moves.push(Move::new(from, to, flags::MOV_CAPTURE));
        }
        moves
    }

    fn gen_bishop_moves(&self, color: Color) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        let mut bishops = self.bb_bishops(color);
        let occupied = self.bb_own(color) | self.bb_opponent(color);

        for from in bishops.iter() {
            let mut atk = (bitboard::diagonal_attacks(from, occupied)
                | bitboard::anti_diagonal_attacks(from, occupied))
                & self.bb_empty();

            for to in atk.iter() {
                moves.push(Move::new(from, to, flags::MOV_QUIET));
            }
        }
        moves
    }

    fn gen_bishop_captures(&self, color: Color) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        let mut bishops = self.bb_bishops(color);
        let occupied = self.bb_own(color) | self.bb_opponent(color);

        for from in bishops.iter() {
            let mut atk = (bitboard::diagonal_attacks(from, occupied)
                | bitboard::anti_diagonal_attacks(from, occupied))
                & self.bb_opponent(color);

            for to in atk.iter() {
                moves.push(Move::new(from, to, flags::MOV_CAPTURE));
            }
        }
        moves
    }

    fn gen_rook_moves(&self, color: Color) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        let mut rooks = self.bb_rooks(color);
        let occupied = self.bb_own(color) | self.bb_opponent(color);

        for from in rooks.iter() {
            let mut atk = (bitboard::rank_attacks(from, occupied)
                | bitboard::file_attacks(from, occupied))
                & self.bb_empty();
            
            for to in atk.iter() {
                moves.push(Move::new(from, to, flags::MOV_QUIET));
            }
        }
        moves
    }

    fn gen_rook_captures(&self, color: Color) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        let mut rooks = self.bb_rooks(color);
        let occupied = self.bb_own(color) | self.bb_opponent(color);

        for from in rooks.iter() {
            let mut atk = (bitboard::rank_attacks(from, occupied)
                | bitboard::file_attacks(from, occupied))
                & self.bb_opponent(color);
            
            for to in atk.iter() {
                moves.push(Move::new(from, to, flags::MOV_CAPTURE));
            }
        }
        moves
    }

    fn gen_queen_moves(&self, color: Color) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        let mut queens = self.bb_queens(color);
        let occupied = self.bb_own(color) | self.bb_opponent(color);

        for from in queens.iter() {
            let mut atk = self.bb_empty()
                & (bitboard::rank_attacks(from, occupied)
                    | bitboard::file_attacks(from, occupied)
                    | bitboard::diagonal_attacks(from, occupied)
                    | bitboard::anti_diagonal_attacks(from, occupied));
            for to in atk.iter() {
                moves.push(Move::new(from, to, flags::MOV_QUIET));
            }
        }
        moves
    }

    fn gen_queen_captures(&self, color: Color) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        let mut queens = self.bb_queens(color);
        let occupied = self.bb_own(color) | self.bb_opponent(color);

        for from in queens.iter() {
            let mut atk = self.bb_opponent(color)
                & (bitboard::rank_attacks(from, occupied)
                    | bitboard::file_attacks(from, occupied)
                    | bitboard::diagonal_attacks(from, occupied)
                    | bitboard::anti_diagonal_attacks(from, occupied));
            for to in atk.iter() {
                moves.push(Move::new(from, to, flags::MOV_CAPTURE));
            }
        }
        moves
    }
}

#[cfg(test)]
mod tests {
    use crate::board::Board;
    use crate::color;
    use crate::fen::BoardFen;
    use crate::move_generator::MoveGenerator;
    use crate::square;
    use crate::uci::UCIInterface;

    #[test]
    fn it_generates_pawn_moves() {
        let mut board = Board::startpos();

        assert_eq!(16, MoveGenerator::gen_white_pawn_pushes(&board).len());
        board.input_move(square::E2, square::E4, None).unwrap();
        assert_eq!(16, MoveGenerator::gen_black_pawn_pushes(&board).len());

        board = Board::from_fen_str("8/PPPPPPPP/8/8/8/8/8/8 w - - 0 1").unwrap();
        assert_eq!(32, MoveGenerator::gen_white_pawn_pushes(&board).len());
        board = Board::from_fen_str("8/8/8/8/8/8/pppppppp/8 b - - 0 1").unwrap();
        assert_eq!(32, MoveGenerator::gen_black_pawn_pushes(&board).len());
    }

    #[test]
    fn it_generates_pawn_captures() {
        //let _gen = MoveGenerator::new();
        let mut board = Board::from_fen_str("8/8/8/p1p1p1p1/P1P1P1P1/8/8/8 w - - 0 1").unwrap();
        assert_eq!(0, MoveGenerator::gen_white_pawn_captures(&board).len());

        board = Board::from_fen_str("1k6/8/8/p1p1p1p1/1P1P1P1P/8/8/K7 w - - 0 1").unwrap();
        assert_eq!(7, MoveGenerator::gen_white_pawn_captures(&board).len());
        assert_eq!(4, MoveGenerator::gen_white_pawn_pushes(&board).len());
        // assert_eq!(11, MoveGenerator::from_board(&board).len());

        board = Board::from_fen_str("1k6/3p4/8/4P/8/8/8/6K1 b - - 0 1").unwrap();
        board.input_move(square::D7, square::D5, None).unwrap();
        assert_eq!(1, MoveGenerator::gen_white_pawn_captures(&board).len());
        assert_eq!(1, MoveGenerator::gen_white_pawn_pushes(&board).len());
        // assert_eq!(2, MoveGenerator::from_board(&board).len());
    }

    #[test]
    fn it_generates_king_moves() {
        // position startpos moves e2e4 e7e5 g1f3 g8f6 f1d3 f8d6
        let mut board = Board::startpos();
        board.input_move(square::E2, square::E4, None).unwrap();
        board.input_move(square::E7, square::E5, None).unwrap();
        board.input_move(square::G1, square::F3, None).unwrap();
        board.input_move(square::G8, square::F6, None).unwrap();
        board.input_move(square::F1, square::D3, None).unwrap();
        board.input_move(square::F8, square::D6, None).unwrap();
        assert_eq!(2, MoveGenerator::gen_king_moves(&board, color::WHITE).len());
        assert_eq!(1, MoveGenerator::gen_wking_castle(&board).len());
    }

    #[test]
    fn it_generates_castles() {
        let mut c = UCIInterface::new();
        c.parse(String::from(
            "position startpos moves e2e4 d7d5 g1f3 b8c6 f1e2 c8e6 e1g1 d8d6 d2d4",
        ));
        assert_eq!(39, c.board.generate_moves().len());
    }

    #[test]
    fn it_generates_all_moves() {
        // see https://chessprogramming.wikispaces.com/Perft+Results
        // Position 1(startpos) perft(1)
        let mut board = Board::startpos();
        assert_eq!(20, board.generate_moves().len());

        // Position 2(kiwipete) perft(1)
        board = Board::from_fen_str(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap();
        assert_eq!(48, board.generate_moves().len());

        // // Position 3 perft(1)
        // board = Board::from_fen_str("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
        // assert_eq!(14, MoveGenerator::from_board(&board).len());

        // // Position 4 perft(1)
        // board = Board::from_fen_str("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1").unwrap();
        // assert_eq!(6, MoveGenerator::from_board(&board).len());

        // // Position 5 perft(1)
        // board = Board::from_fen_str("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();
        // assert_eq!(44, MoveGenerator::from_board(&board).len());

        // Position 6 perft(1)
        board = Board::from_fen_str(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        )
        .unwrap();
        board.generate_moves();
        assert_eq!(46, board.generate_moves().len());
    }
}
