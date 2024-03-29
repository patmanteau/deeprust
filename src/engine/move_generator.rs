use crate::engine::{bitboards, Bitboard, BitboardPrimitives, Board};
// use crate::board::Board;
use crate::common::*;
use crate::primitives::*;
// use crate::primitives::r#move::flags;
// use crate::primitives::r#move::Move;

// use crate::primitives::color::{self, Color};
// use crate::primitives::square::{self, Square};

pub trait MoveGenerator {
    fn generate_moves(&self) -> Vec<Move>;
    fn is_mate(&mut self, _color: Color) -> bool;
    fn is_in_check(&self, color: Color) -> bool;
    fn is_attacked(&self, color: Color, target: Square) -> bool;

    fn gen_white_pawn_moves(&self, moves: &mut Vec<Move>);
    fn gen_black_pawn_moves(&self, moves: &mut Vec<Move>);
    fn gen_knight_moves(&self, moves: &mut Vec<Move>, color: Color);
    fn gen_king_moves(&self, moves: &mut Vec<Move>, color: Color);
    fn gen_wking_castle(&self, moves: &mut Vec<Move>);
    fn gen_bking_castle(&self, moves: &mut Vec<Move>);
    fn gen_bishop_moves(&self, moves: &mut Vec<Move>, color: Color);
    fn gen_rook_moves(&self, moves: &mut Vec<Move>, color: Color);
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

        // by knights
        if (bitboards::BB_KNIGHT_ATTACKS[target as usize] & pos.bb_knights(1 ^ color)) > 0 {
            return true;
        }

        // by pawns
        if bitboards::BB_PAWN_ATTACKS[color as usize][target as usize] & pos.bb_pawns(1 ^ color) > 0
        {
            return true;
        }

        // by king?!?
        if (bitboards::BB_KING_ATTACKS[target as usize] & pos.bb_king(1 ^ color)) > 0 {
            return true;
        }

        // by bishops or queens
        if bitboards::bishop_attacks(target, occupied)
            & (pos.bb_bishops(1 ^ color) | pos.bb_queens(1 ^ color))
            > 0
        {
            return true;
        }

        // by rooks or queens
        if (bitboards::rank_attacks(target, occupied) | bitboards::file_attacks(target, occupied))
            & (pos.bb_rooks(1 ^ color) | pos.bb_queens(1 ^ color))
            > 0
        {
            return true;
        }

        false
    }

    fn generate_moves(&self) -> Vec<Move> {
        let mut moves = Vec::with_capacity(512);
        let to_move = self.current().to_move();

        if to_move == colors::WHITE {
            self.gen_white_pawn_moves(&mut moves);
            self.gen_wking_castle(&mut moves);
        } else {
            self.gen_black_pawn_moves(&mut moves);
            self.gen_bking_castle(&mut moves);
        }

        self.gen_knight_moves(&mut moves, to_move);
        self.gen_bishop_moves(&mut moves, to_move);
        self.gen_rook_moves(&mut moves, to_move);
        self.gen_king_moves(&mut moves, to_move);

        moves
    }

    fn gen_white_pawn_moves(&self, moves: &mut Vec<Move>) {
        let pos = self.current();
        let pawns = pos.bb_pawns(colors::WHITE);

        let (ep_square, ep_bb) = match pos.en_passant() {
            Some(sq) => (sq, bitboards::BB_SQUARES[sq as usize]),
            None => (0, bitboards::BB_EMPTY),
        };

        let mut norm_pawns = pawns & bitboards::BB_NOT_RANK_78;
        let mut prom_pawns = pawns & bitboards::BB_RANK_7;

        for from in prom_pawns.iter() {
            // captures
            for to in
                (bitboards::BB_WPAWN_ATTACKS[from as usize] & pos.bb_opponent(colors::WHITE)).iter()
            {
                // promotion
                let mov = Move::new(from, to, flags::MOV_QUIET);
                moves.push(mov.with_flags(flags::MOV_CAPTURE | flags::MOV_PROM_QUEEN));
                moves.push(mov.with_flags(flags::MOV_CAPTURE | flags::MOV_PROM_ROOK));
                moves.push(mov.with_flags(flags::MOV_CAPTURE | flags::MOV_PROM_BISHOP));
                moves.push(mov.with_flags(flags::MOV_CAPTURE | flags::MOV_PROM_KNIGHT));
            }

            // pushes
            for to in (bitboards::BB_WPAWN_PUSHES[from as usize] & pos.bb_empty()).iter() {
                // promotion
                let mov = Move::new(from, to, flags::MOV_QUIET);
                moves.push(mov.with_flags(flags::MOV_PROM_QUEEN));
                moves.push(mov.with_flags(flags::MOV_PROM_ROOK));
                moves.push(mov.with_flags(flags::MOV_PROM_BISHOP));
                moves.push(mov.with_flags(flags::MOV_PROM_KNIGHT));
            }
        }

        for from in norm_pawns.iter() {
            // captures
            let mut atk = bitboards::BB_WPAWN_ATTACKS[from as usize]
                & (pos.bb_opponent(colors::WHITE) | ep_bb);
            for to in atk.iter() {
                moves.push(Move::new(
                    from,
                    to,
                    Move::make_flags(true, false, to == ep_square, false),
                ));
            }

            // pushes
            let mut single_push =
                bitboards::north_one(bitboards::BB_SQUARES[from as usize]) & pos.bb_empty();
            let mut double_push =
                bitboards::north_one(single_push) & pos.bb_empty() & bitboards::BB_RANK_4;
            for to in single_push.iter() {
                moves.push(Move::new(from, to, flags::MOV_QUIET));
            }
            for to in double_push.iter() {
                moves.push(Move::new(from, to, flags::MOV_DPP));
            }
        }
    }

    fn gen_black_pawn_moves(&self, moves: &mut Vec<Move>) {
        let pos = self.current();
        let pawns = pos.bb_pawns(colors::BLACK);

        let (ep_square, ep_bb) = match pos.en_passant() {
            Some(sq) => (sq, bitboards::BB_SQUARES[sq as usize]),
            None => (0, bitboards::BB_EMPTY),
        };

        let mut norm_pawns = pawns & bitboards::BB_NOT_RANK_12;
        let mut prom_pawns = pawns & bitboards::BB_RANK_2;

        // promotions
        for from in prom_pawns.iter() {
            // captures
            for to in
                (bitboards::BB_BPAWN_ATTACKS[from as usize] & pos.bb_opponent(colors::BLACK)).iter()
            {
                let mov = Move::new(from, to, flags::MOV_QUIET);
                moves.push(mov.with_flags(flags::MOV_CAPTURE | flags::MOV_PROM_QUEEN));
                moves.push(mov.with_flags(flags::MOV_CAPTURE | flags::MOV_PROM_ROOK));
                moves.push(mov.with_flags(flags::MOV_CAPTURE | flags::MOV_PROM_BISHOP));
                moves.push(mov.with_flags(flags::MOV_CAPTURE | flags::MOV_PROM_KNIGHT));
            }

            // pushes
            for to in (bitboards::BB_BPAWN_PUSHES[from as usize] & pos.bb_empty()).iter() {
                let mov = Move::new(from, to, flags::MOV_QUIET);
                moves.push(mov.with_flags(flags::MOV_PROM_QUEEN));
                moves.push(mov.with_flags(flags::MOV_PROM_ROOK));
                moves.push(mov.with_flags(flags::MOV_PROM_BISHOP));
                moves.push(mov.with_flags(flags::MOV_PROM_KNIGHT));
            }
        }

        for from in norm_pawns.iter() {
            // captures
            let mut atk = bitboards::BB_BPAWN_ATTACKS[from as usize]
                & (pos.bb_opponent(colors::BLACK) | ep_bb);
            for to in atk.iter() {
                moves.push(Move::new(
                    from,
                    to,
                    Move::make_flags(true, false, to == ep_square, false),
                ));
            }

            // pushes
            let mut single_push =
                bitboards::south_one(bitboards::BB_SQUARES[from as usize]) & pos.bb_empty();
            let mut double_push =
                bitboards::south_one(single_push) & pos.bb_empty() & bitboards::BB_RANK_5;
            for to in single_push.iter() {
                moves.push(Move::new(from, to, flags::MOV_QUIET));
            }
            for to in double_push.iter() {
                moves.push(Move::new(from, to, flags::MOV_DPP));
            }
        }
    }

    fn gen_knight_moves(&self, moves: &mut Vec<Move>, color: Color) {
        let pos = self.current();
        let mut knights = pos.bb_knights(color);

        for from in knights.iter() {
            // captures
            let mut atk = bitboards::BB_KNIGHT_ATTACKS[from as usize] & pos.bb_opponent(color);

            for to in atk.iter() {
                moves.push(Move::new(from, to, flags::MOV_CAPTURE));
            }

            // quiets
            let mut mov = bitboards::BB_KNIGHT_ATTACKS[from as usize] & pos.bb_empty();

            for to in mov.iter() {
                moves.push(Move::new(from, to, flags::MOV_QUIET));
            }
        }
    }

    fn gen_wking_castle(&self, moves: &mut Vec<Move>) {
        let pos = self.current();
        let occ = pos.bb_own(colors::WHITE) | pos.bb_opponent(colors::WHITE);

        if self.is_attacked(colors::WHITE, squares::E1) {
            return;
        }

        let qlear = pos.castling().get(colors::WHITE, sides::QUEEN_SIDE) //pos.castling()[colors::WHITE as usize].test_bit(1)
            && occ.extract_bits(squares::B1, 3) == 0
            && !self.is_attacked(colors::WHITE, squares::C1)
            && !self.is_attacked(colors::WHITE, squares::D1);

        let klear = pos.castling().get(colors::WHITE, sides::KING_SIDE) //pos.castling()[colors::WHITE as usize].test_bit(0)
            && occ.extract_bits(squares::F1, 2) == 0
            && !self.is_attacked(colors::WHITE, squares::F1);

        if qlear {
            moves.push(Move::new(squares::E1, squares::C1, flags::MOV_Q_CASTLE));
        }
        if klear {
            moves.push(Move::new(squares::E1, squares::G1, flags::MOV_K_CASTLE));
        }
    }

    fn gen_bking_castle(&self, moves: &mut Vec<Move>) {
        let pos = self.current();
        let occ = pos.bb_own(colors::BLACK) | pos.bb_opponent(colors::BLACK);

        if self.is_attacked(colors::BLACK, squares::E8) {
            return;
        }

        let qlear = pos.castling().get(colors::BLACK, sides::QUEEN_SIDE) //pos.castling()[colors::BLACK as usize].test_bit(1)
            && occ.extract_bits(squares::B8, 3) == 0
            && !self.is_attacked(colors::BLACK, squares::C8)
            && !self.is_attacked(colors::BLACK, squares::D8);

        let klear = pos.castling().get(colors::BLACK, sides::KING_SIDE) //pos.castling()[colors::BLACK as usize].test_bit(0)
            && occ.extract_bits(squares::F8, 2) == 0
            && !self.is_attacked(colors::BLACK, squares::F8);

        if qlear {
            moves.push(Move::new(squares::E8, squares::C8, flags::MOV_Q_CASTLE));
        }
        if klear {
            moves.push(Move::new(squares::E8, squares::G8, flags::MOV_K_CASTLE));
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

        // captures
        let mut atk = bitboards::BB_KING_ATTACKS[from as usize] & pos.bb_opponent(color);

        for to in atk.iter() {
            moves.push(Move::new(from, to, flags::MOV_CAPTURE));
        }

        // quiets
        let mut mov = bitboards::BB_KING_ATTACKS[from as usize] & pos.bb_empty();

        for to in mov.iter() {
            moves.push(Move::new(from, to, flags::MOV_QUIET));
        }
    }

    fn gen_bishop_moves(&self, moves: &mut Vec<Move>, color: Color) {
        let pos = self.current();
        let mut bishops = pos.bb_bishops(color) | pos.bb_queens(color);
        let occupied = pos.bb_own(color) | pos.bb_opponent(color);

        for from in bishops.iter() {
            // let rays = bitboards::diagonal_attacks(from, occupied)
            //     | bitboards::anti_diagonal_attacks(from, occupied);
            let rays = bitboards::bishop_attacks(from, occupied);

            // captures
            let mut atk = rays & pos.bb_opponent(color);

            for to in atk.iter() {
                moves.push(Move::new(from, to, flags::MOV_CAPTURE));
            }

            // quiets
            let mut mov = rays & pos.bb_empty();

            for to in mov.iter() {
                moves.push(Move::new(from, to, flags::MOV_QUIET));
            }
        }
    }

    fn gen_rook_moves(&self, moves: &mut Vec<Move>, color: Color) {
        let pos = self.current();
        let mut rooks = pos.bb_rooks(color) | pos.bb_queens(color);
        let occupied = pos.bb_own(color) | pos.bb_opponent(color);

        for from in rooks.iter() {
            let rays =
                bitboards::rank_attacks(from, occupied) | bitboards::file_attacks(from, occupied);

            // captures
            let mut atk = rays & pos.bb_opponent(color);

            for to in atk.iter() {
                moves.push(Move::new(from, to, flags::MOV_CAPTURE));
            }

            // quiets
            let mut mov = rays & pos.bb_empty();

            for to in mov.iter() {
                moves.push(Move::new(from, to, flags::MOV_QUIET));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::{Board, MoveGenerator};
    use crate::interfaces::FenInterface;
    use crate::primitives::{colors, squares};
    // use crate::primitives::square;
    use crate::frontends::UCIFrontend;

    // #[test]
    // fn it_generates_pawn_moves() {
    //     let mut board = Board::startpos();
    //     let mut moves = Vec::with_capacity(512);

    //     MoveGenerator::gen_white_pawn_pushes(&board, &mut moves);
    //     assert_eq!(16, moves.len());
    //     moves.clear();

    //     board.input_move(square::E2, square::E4, None).unwrap();
    //     MoveGenerator::gen_black_pawn_pushes(&board, &mut moves);
    //     assert_eq!(16, moves.len());
    //     moves.clear();

    //     board = Board::from_fen_str("8/PPPPPPPP/8/8/8/8/8/8 w - - 0 1").unwrap();
    //     MoveGenerator::gen_white_pawn_pushes(&board, &mut moves);
    //     assert_eq!(32, moves.len());
    //     moves.clear();

    //     board = Board::from_fen_str("8/8/8/8/8/8/pppppppp/8 b - - 0 1").unwrap();
    //     MoveGenerator::gen_black_pawn_pushes(&board, &mut moves);
    //     assert_eq!(32, moves.len());
    //     moves.clear();
    // }

    // #[test]
    // fn it_generates_pawn_captures() {
    //     //let _gen = MoveGenerator::new();
    //     let mut board = Board::from_fen_str("8/8/8/p1p1p1p1/P1P1P1P1/8/8/8 w - - 0 1").unwrap();
    //     let mut moves = Vec::with_capacity(512);

    //     board = Board::from_fen_str("8/8/8/8/8/8/pppppppp/8 b - - 0 1").unwrap();
    //     MoveGenerator::gen_white_pawn_captures(&board, &mut moves);
    //     assert_eq!(0, moves.len());
    //     moves.clear();

    //     board = Board::from_fen_str("1k6/8/8/p1p1p1p1/1P1P1P1P/8/8/K7 w - - 0 1").unwrap();
    //     MoveGenerator::gen_white_pawn_captures(&board, &mut moves);
    //     assert_eq!(7, moves.len());
    //     moves.clear();

    //     MoveGenerator::gen_white_pawn_pushes(&board, &mut moves);
    //     assert_eq!(4, moves.len());
    //     moves.clear();

    //     board = Board::from_fen_str("1k6/3p4/8/4P3/8/8/8/6K1 b - - 0 1").unwrap();
    //     board.input_move(square::D7, square::D5, None).unwrap();
    //     MoveGenerator::gen_white_pawn_captures(&board, &mut moves);
    //     assert_eq!(1, moves.len());
    //     moves.clear();

    //     MoveGenerator::gen_white_pawn_pushes(&board, &mut moves);
    //     assert_eq!(1, moves.len());
    //     moves.clear();
    // }

    #[test]
    fn it_generates_king_moves() {
        let mut board = Board::startpos();
        let mut moves = Vec::with_capacity(512);
        board.input_move(squares::E2, squares::E4, None).unwrap();
        board.input_move(squares::E7, squares::E5, None).unwrap();
        board.input_move(squares::G1, squares::F3, None).unwrap();
        board.input_move(squares::G8, squares::F6, None).unwrap();
        board.input_move(squares::F1, squares::D3, None).unwrap();
        board.input_move(squares::F8, squares::D6, None).unwrap();

        MoveGenerator::gen_king_moves(&board, &mut moves, colors::WHITE);
        assert_eq!(2, moves.len());
        moves.clear();

        MoveGenerator::gen_wking_castle(&board, &mut moves);
        assert_eq!(1, moves.len());
        moves.clear();
    }

    // TODO: move to UCI tests
    // #[test]
    fn it_generates_castles() {
        let mut c = UCIFrontend::new();
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
