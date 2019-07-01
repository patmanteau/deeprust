use crate::bitboard::{self, BitboardPrimitives};
use crate::board::Board;
use crate::common::*;
use crate::moves::flags;
use crate::moves::Move;

use crate::color::{self, Color};
use crate::square::{self, Square};

// #[derive(Default)]
// pub struct MoveGenerator {}
pub trait MoveGenerator {
    fn generate_moves(&self) -> Vec<Move>;
    // fn perft(&mut self, depth: u32) -> PerftContext;
    // fn do_perft(&mut self, ctx: &mut PerftContext, depth: u32);
    fn is_mate(&mut self, _color: Color) -> bool;
    fn is_in_check(&self, color: Color) -> bool;
    fn is_attacked(&self, color: Color, target: Square) -> bool;

    fn gen_white_pawn_pushes(&self, moves: &mut Vec<Move>);
    fn gen_black_pawn_pushes(&self, moves: &mut Vec<Move>);
    fn gen_white_pawn_captures(&self, moves: &mut Vec<Move>);
    fn gen_black_pawn_captures(&self, moves: &mut Vec<Move>);
    fn gen_knight_moves(&self, moves: &mut Vec<Move>, color: Color);

    fn gen_knight_captures(&self, moves: &mut Vec<Move>, color: Color);
    fn gen_wking_castle(&self, moves: &mut Vec<Move>);
    fn gen_bking_castle(&self, moves: &mut Vec<Move>);
    fn gen_king_moves(&self, moves: &mut Vec<Move>, color: Color);
    fn gen_king_captures(&self, moves: &mut Vec<Move>, color: Color);
    fn gen_bishop_moves(&self, moves: &mut Vec<Move>, color: Color);
    fn gen_bishop_captures(&self, moves: &mut Vec<Move>, color: Color);
    fn gen_rook_moves(&self, moves: &mut Vec<Move>, color: Color);
    fn gen_rook_captures(&self, moves: &mut Vec<Move>, color: Color);
}

impl MoveGenerator for Board {
    fn is_mate(&mut self, _color: Color) -> bool {
        let moves = self.generate_moves();
        for mov in moves.iter() {
            self.make_move(*mov);
            if !self.is_in_check(1 ^ self.current().to_move()) {
                self.unmake_move();
                return false;
            } else {
                self.unmake_move();
            }
        }
        true
    }

    #[inline]
    fn is_in_check(&self, color: Color) -> bool {
        let kingpos = self.current().bb_king(color).scan();
        self.is_attacked(color, kingpos)
    }

    fn is_attacked(&self, color: Color, target: Square) -> bool {
        let pos = self.current();
        let occupied = pos.bb_own(color) | pos.bb_opponent(color);

        debug_assert!(target < 64);

        if (bitboard::diagonal_attacks(target, occupied)
            | bitboard::anti_diagonal_attacks(target, occupied))
            & (pos.bb_bishops(1 ^ color) | pos.bb_queens(1 ^ color))
            > 0
        {
                return true;
            }

        // by rooks or queens
        if (bitboard::rank_attacks(target, occupied) | bitboard::file_attacks(target, occupied))
            & (pos.bb_rooks(1 ^ color) | pos.bb_queens(1 ^ color))
            > 0
        {
            return true;
        }

        // by knights
        if (bitboard::BB_KNIGHT_ATTACKS[target as usize] & pos.bb_knights(1 ^ color)) > 0 {
            return true;
        }

        // by pawns
        if color == color::WHITE {
            if bitboard::BB_WPAWN_ATTACKS[target as usize] & pos.bb_pawns(color::BLACK) > 0 {
            return true;
        }
        } else if bitboard::BB_BPAWN_ATTACKS[target as usize] & pos.bb_pawns(color::WHITE) > 0 {
            return true;
        }

        // let all_diag_attacks = bitboard::diagonal_attacks(target, occupied)
        //     | bitboard::anti_diagonal_attacks(target, occupied);
        // let rank_file_attacks =
        //     bitboard::rank_attacks(target, occupied) | bitboard::file_attacks(target, occupied);

        // by bishops or queens
        // if all_diag_attacks & (pos.bb_bishops(1 ^ color) | pos.bb_queens(1 ^ color)) > 0 {
        //     return true;
        // }

        // by king?!?
        if (bitboard::BB_KING_ATTACKS[target as usize] & pos.bb_king(1 ^ color)) > 0 {
            return true;
        }

        false
    }

    fn generate_moves(&self) -> Vec<Move> {
        let mut moves = Vec::with_capacity(512);
        let to_move = self.current().to_move();

        if to_move == color::WHITE {
            self.gen_white_pawn_pushes(&mut moves);
            self.gen_white_pawn_captures(&mut moves);
            self.gen_wking_castle(&mut moves);
        } else {
            self.gen_black_pawn_pushes(&mut moves);
            self.gen_black_pawn_captures(&mut moves);
            self.gen_bking_castle(&mut moves);
        }

        self.gen_knight_captures(&mut moves, to_move);
        self.gen_knight_moves(&mut moves, to_move);

        self.gen_bishop_captures(&mut moves, to_move);
        self.gen_bishop_moves(&mut moves, to_move);

        self.gen_rook_captures(&mut moves, to_move);
        self.gen_rook_moves(&mut moves, to_move);

        self.gen_king_captures(&mut moves, to_move);
        self.gen_king_moves(&mut moves, to_move);

        moves
    }

    fn gen_white_pawn_pushes(&self, moves: &mut Vec<Move>) {
        let pos = self.current();
        let pawns = pos.bb_pawns(color::WHITE);

        let mut norm_pawns = pawns & bitboard::BB_NOT_RANK_78;
        let mut prom_pawns = pawns & bitboard::BB_RANK_7;

        for from in prom_pawns.iter() {
            for to in (bitboard::BB_WPAWN_PUSHES[from as usize] & pos.bb_empty()).iter() {
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
                bitboard::north_one(bitboard::BB_SQUARES[from as usize]) & pos.bb_empty();
            let mut double_push =
                bitboard::north_one(single_push) & pos.bb_empty() & bitboard::BB_RANK_4;
            for to in single_push.iter() {
                moves.push(Move::new(from, to, flags::MOV_QUIET));
            }
            for to in double_push.iter() {
                moves.push(Move::new(from, to, flags::MOV_DPP));
            }
        }
    }

    fn gen_black_pawn_pushes(&self, moves: &mut Vec<Move>) {
        let pos = self.current();
        let pawns = pos.bb_pawns(color::BLACK);

        let mut norm_pawns = pawns & bitboard::BB_NOT_RANK_12;
        let mut prom_pawns = pawns & bitboard::BB_RANK_2;

        for from in prom_pawns.iter() {
            //let from = prom_pawns.scan();
            for to in (bitboard::BB_BPAWN_PUSHES[from as usize] & pos.bb_empty()).iter() {
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
                bitboard::south_one(bitboard::BB_SQUARES[from as usize]) & pos.bb_empty();
            let mut double_push =
                bitboard::south_one(single_push) & pos.bb_empty() & bitboard::BB_RANK_5;
            for to in single_push.iter() {
                moves.push(Move::new(from, to, flags::MOV_QUIET));
            }
            for to in double_push.iter() {
                moves.push(Move::new(from, to, flags::MOV_DPP));
            }
        }
    }

    fn gen_white_pawn_captures(&self, moves: &mut Vec<Move>) {
        let pos = self.current();
        let pawns = pos.bb_pawns(color::WHITE);

        // let mut ep_bb = bitboard::BB_EMPTY;
        // let mut ep_square = 0;
        // if let Some(ep_squares) = pos.en_passant() {
        //     ep_square = ep_squares[0];
        //     ep_bb = bitboard::BB_SQUARES[ep_square as usize];
        // }
        let (ep_square, ep_bb) = match pos.en_passant() {
            Some(sq) => (sq[0], bitboard::BB_SQUARES[sq[0] as usize]),
            None => (0, bitboard::BB_EMPTY),
        };

        let mut norm_pawns = pawns & bitboard::BB_NOT_RANK_78;
        let mut prom_pawns = pawns & bitboard::BB_RANK_7;

        for from in prom_pawns.iter() {
            // let from = prom_pawns.scan();
            let mut atk = bitboard::BB_WPAWN_ATTACKS[from as usize] & pos.bb_opponent(color::WHITE);
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
            let mut atk =
                bitboard::BB_WPAWN_ATTACKS[from as usize] & (pos.bb_opponent(color::WHITE) | ep_bb);
            for to in atk.iter() {
                moves.push(Move::new(
                    from,
                    to,
                    Move::make_flags(true, false, to == ep_square, false),
                ));
            }
        }
    }

    fn gen_black_pawn_captures(&self, moves: &mut Vec<Move>) {
        let pos = self.current();
        let pawns = pos.bb_pawns(color::BLACK);
        // let mut ep_bb = bitboard::BB_EMPTY;
        // let mut ep_square = 0;
        // if let Some(ep_squares) = pos.en_passant() {
        //     ep_square = ep_squares[0];
        //     ep_bb = bitboard::BB_SQUARES[ep_square as usize];
        // }
        let (ep_square, ep_bb) = match pos.en_passant() {
            Some(sq) => (sq[0], bitboard::BB_SQUARES[sq[0] as usize]),
            None => (0, bitboard::BB_EMPTY),
        };

        let mut norm_pawns = pawns & bitboard::BB_NOT_RANK_12;
        let mut prom_pawns = pawns & bitboard::BB_RANK_2;

        for from in prom_pawns.iter() {
            let mut atk = bitboard::BB_BPAWN_ATTACKS[from as usize] & pos.bb_opponent(color::BLACK);
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
            let mut atk =
                bitboard::BB_BPAWN_ATTACKS[from as usize] & (pos.bb_opponent(color::BLACK) | ep_bb);
            for to in atk.iter() {
                moves.push(Move::new(
                    from,
                    to,
                    Move::make_flags(true, false, to == ep_square, false),
                ));
            }
        }
    }

    fn gen_knight_moves(&self, moves: &mut Vec<Move>, color: Color) {
        let pos = self.current();
        let mut knights = pos.bb_knights(color);

        for from in knights.iter() {
            let mut atk = bitboard::BB_KNIGHT_ATTACKS[from as usize] & pos.bb_empty();

            for to in atk.iter() {
                moves.push(Move::new(from, to, flags::MOV_QUIET));
            }
        }
    }

    fn gen_knight_captures(&self, moves: &mut Vec<Move>, color: Color) {
        let pos = self.current();
        let mut knights = pos.bb_knights(color);

        for from in knights.iter() {
            let mut atk = bitboard::BB_KNIGHT_ATTACKS[from as usize] & pos.bb_opponent(color);

            for to in atk.iter() {
                moves.push(Move::new(from, to, flags::MOV_CAPTURE));
            }
        }
    }

    fn gen_wking_castle(&self, moves: &mut Vec<Move>) {
        let pos = self.current();
        let occ = pos.bb_own(color::WHITE) | pos.bb_opponent(color::WHITE);

        if self.is_attacked(color::WHITE, square::E1) {
            return;
        }

        let qlear = occ.extract_bits(u32::from(square::B1), 3) == 0
            && !self.is_attacked(color::WHITE, square::C1)
            && !self.is_attacked(color::WHITE, square::D1)
            && pos.castling()[color::WHITE as usize].test_bit(1);

        let klear = occ.extract_bits(u32::from(square::F1), 2) == 0
            && !self.is_attacked(color::WHITE, square::F1)
            && pos.castling()[color::WHITE as usize].test_bit(0);

        if qlear {
            moves.push(Move::new(square::E1, square::C1, flags::MOV_Q_CASTLE));
        }
        if klear {
            moves.push(Move::new(square::E1, square::G1, flags::MOV_K_CASTLE));
        }
    }

    fn gen_bking_castle(&self, moves: &mut Vec<Move>) {
        let pos = self.current();
        let occ = pos.bb_own(color::BLACK) | pos.bb_opponent(color::BLACK);

        if self.is_attacked(color::BLACK, square::E8) {
            return;
        }

        let qlear = occ.extract_bits(u32::from(square::B8), 3) == 0
            && !self.is_attacked(color::BLACK, square::C8)
            && !self.is_attacked(color::BLACK, square::D8)
            && pos.castling()[color::BLACK as usize].test_bit(1);

        let klear = occ.extract_bits(u32::from(square::F8), 2) == 0
            && !self.is_attacked(color::BLACK, square::F8)
            && pos.castling()[color::BLACK as usize].test_bit(0);

        if qlear {
            moves.push(Move::new(square::E8, square::C8, flags::MOV_Q_CASTLE));
        }
        if klear {
            moves.push(Move::new(square::E8, square::G8, flags::MOV_K_CASTLE));
        }
    }

    fn gen_king_moves(&self, moves: &mut Vec<Move>, color: Color) {
        let pos = self.current();

        let from = pos.bb_king(color).scan();
        
        #[cfg(feature = "sanity_checks")]
        {
            if from > 63 {
                self.panic_dump();
            }
        }

        let mut atk = bitboard::BB_KING_ATTACKS[from as usize] & pos.bb_empty();

        for to in atk.iter() {
            moves.push(Move::new(from, to, flags::MOV_QUIET));
        }
    }

    fn gen_king_captures(&self, moves: &mut Vec<Move>, color: Color) {
        let pos = self.current();

        let from = pos.bb_king(color).scan();

        #[cfg(feature = "sanity_checks")]
        {
            if from > 63 {
                self.panic_dump();
            }
        }

        //assert!(from < 64);
        let mut atk = bitboard::BB_KING_ATTACKS[from as usize] & pos.bb_opponent(color);

        for to in atk.iter() {
            moves.push(Move::new(from, to, flags::MOV_CAPTURE));
        }
    }

    fn gen_bishop_moves(&self, moves: &mut Vec<Move>, color: Color) {
        let pos = self.current();
        let mut bishops = pos.bb_bishops(color) | pos.bb_queens(color);
        let occupied = pos.bb_own(color) | pos.bb_opponent(color);

        for from in bishops.iter() {
            let mut atk = (bitboard::diagonal_attacks(from, occupied)
                | bitboard::anti_diagonal_attacks(from, occupied))
                & pos.bb_empty();

            for to in atk.iter() {
                moves.push(Move::new(from, to, flags::MOV_QUIET));
            }
        }
    }

    fn gen_bishop_captures(&self, moves: &mut Vec<Move>, color: Color) {
        let pos = self.current();
        let mut bishops = pos.bb_bishops(color) | pos.bb_queens(color);
        let occupied = pos.bb_own(color) | pos.bb_opponent(color);

        for from in bishops.iter() {
            let mut atk = (bitboard::diagonal_attacks(from, occupied)
                | bitboard::anti_diagonal_attacks(from, occupied))
                & pos.bb_opponent(color);

            for to in atk.iter() {
                moves.push(Move::new(from, to, flags::MOV_CAPTURE));
            }
        }
    }

    fn gen_rook_moves(&self, moves: &mut Vec<Move>, color: Color) {
        let pos = self.current();
        let mut rooks = pos.bb_rooks(color) | pos.bb_queens(color);
        let occupied = pos.bb_own(color) | pos.bb_opponent(color);

        for from in rooks.iter() {
            let mut atk = (bitboard::rank_attacks(from, occupied)
                | bitboard::file_attacks(from, occupied))
                & pos.bb_empty();

            for to in atk.iter() {
                moves.push(Move::new(from, to, flags::MOV_QUIET));
            }
        }
    }

    fn gen_rook_captures(&self, moves: &mut Vec<Move>, color: Color) {
        let pos = self.current();
        let mut rooks = pos.bb_rooks(color) | pos.bb_queens(color);
        let occupied = pos.bb_own(color) | pos.bb_opponent(color);

        for from in rooks.iter() {
            let mut atk = (bitboard::rank_attacks(from, occupied)
                | bitboard::file_attacks(from, occupied))
                & pos.bb_opponent(color);

            for to in atk.iter() {
                moves.push(Move::new(from, to, flags::MOV_CAPTURE));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::board::Board;
    use crate::color;
    use crate::interfaces::FenInterface;
    use crate::move_generator::MoveGenerator;
    use crate::square;
    use crate::uci::UCIInterface;

    #[test]
    fn it_generates_pawn_moves() {
        let mut board = Board::startpos();
        let mut moves = Vec::with_capacity(512);

        MoveGenerator::gen_white_pawn_pushes(&board, &mut moves);
        assert_eq!(16, moves.len());
        moves.clear();

        board.input_move(square::E2, square::E4, None).unwrap();
        MoveGenerator::gen_black_pawn_pushes(&board, &mut moves);
        assert_eq!(16, moves.len());
        moves.clear();

        board = Board::from_fen_str("8/PPPPPPPP/8/8/8/8/8/8 w - - 0 1").unwrap();
        MoveGenerator::gen_white_pawn_pushes(&board, &mut moves);
        assert_eq!(32, moves.len());
        moves.clear();

        board = Board::from_fen_str("8/8/8/8/8/8/pppppppp/8 b - - 0 1").unwrap();
        MoveGenerator::gen_black_pawn_pushes(&board, &mut moves);
        assert_eq!(32, moves.len());
        moves.clear();
    }

    #[test]
    fn it_generates_pawn_captures() {
        //let _gen = MoveGenerator::new();
        let mut board = Board::from_fen_str("8/8/8/p1p1p1p1/P1P1P1P1/8/8/8 w - - 0 1").unwrap();
        let mut moves = Vec::with_capacity(512);

        board = Board::from_fen_str("8/8/8/8/8/8/pppppppp/8 b - - 0 1").unwrap();
        MoveGenerator::gen_white_pawn_captures(&board, &mut moves);
        assert_eq!(0, moves.len());
        moves.clear();

        board = Board::from_fen_str("1k6/8/8/p1p1p1p1/1P1P1P1P/8/8/K7 w - - 0 1").unwrap();
        MoveGenerator::gen_white_pawn_captures(&board, &mut moves);
        assert_eq!(7, moves.len());
        moves.clear();

        MoveGenerator::gen_white_pawn_pushes(&board, &mut moves);
        assert_eq!(4, moves.len());
        moves.clear();

        board = Board::from_fen_str("1k6/3p4/8/4P3/8/8/8/6K1 b - - 0 1").unwrap();
        board.input_move(square::D7, square::D5, None).unwrap();
        MoveGenerator::gen_white_pawn_captures(&board, &mut moves);
        assert_eq!(1, moves.len());
        moves.clear();

        MoveGenerator::gen_white_pawn_pushes(&board, &mut moves);
        assert_eq!(1, moves.len());
        moves.clear();
    }

    #[test]
    fn it_generates_king_moves() {
        let mut board = Board::startpos();
        let mut moves = Vec::with_capacity(512);
        board.input_move(square::E2, square::E4, None).unwrap();
        board.input_move(square::E7, square::E5, None).unwrap();
        board.input_move(square::G1, square::F3, None).unwrap();
        board.input_move(square::G8, square::F6, None).unwrap();
        board.input_move(square::F1, square::D3, None).unwrap();
        board.input_move(square::F8, square::D6, None).unwrap();

        MoveGenerator::gen_king_moves(&board, &mut moves, color::WHITE);
        assert_eq!(2, moves.len());
        moves.clear();

        MoveGenerator::gen_wking_castle(&board, &mut moves);
        assert_eq!(1, moves.len());
        moves.clear();
    }

    // TODO: move to UCI tests
    // #[test]
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
