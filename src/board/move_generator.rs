use board::Board;
use board::common::*;
use board::bitboard::*;
use board::moves::Move;
use board::util::{bb, piece, squares};
use board::types::Sq;

pub struct MoveGenerator {
    
}

impl MoveGenerator {
    pub fn new() -> MoveGenerator {
        MoveGenerator{
            
        }
    }

    pub fn perft(board: &mut Board, depth: u32) -> u64 {
        if depth == 0 {
            return 1;
        }

        let mut nodes = 0u64;
        let moves = MoveGenerator::from_board(board);
        for mov in moves.iter() {
            board.make_move(*mov);
            if !MoveGenerator::is_in_check(board, 1 ^ board.to_move()) {
                nodes += MoveGenerator::perft(board, depth - 1);
            }
            board.unmake_move();
        }

        nodes
    }

    fn break_helper() {
        let a = 0;
    }

    #[inline]
    pub fn is_in_check(board: &Board, color: u32) -> bool {
        let kingpos = board.bb_king(color).scan();
        MoveGenerator::is_attacked(board, color, kingpos)
    }

    #[inline]
    pub fn is_attacked(board: &Board, color: u32, target: Sq) -> bool {
        let occupied = board.bb_own(color) | board.bb_opponent(color);

        if 63 < target {
            MoveGenerator::break_helper();
        }

        // by black pawns
        if color == piece::WHITE {
            if 0 < 
                ((bb::north_west_one(bb::BB_SQUARES[target as usize]) |
                bb::north_east_one(bb::BB_SQUARES[target as usize])) &
               (board.bb_pawns(piece::BLACK))) {
                   return true
            }
        } else { // by white pawns
            if 0 < 
               ((bb::south_west_one(bb::BB_SQUARES[target as usize]) |
                bb::south_east_one(bb::BB_SQUARES[target as usize])) &
               (board.bb_pawns(piece::WHITE))) {
                   return true
            }
        }

        // by knights
        if 0 < (bb::BB_KNIGHT_ATTACKS[target as usize] & board.bb_knights(1 ^ color)) {
            return true;
        }

        // by king?!?
        if 0 < (bb::BB_KING_ATTACKS[target as usize] & board.bb_king(1 ^ color)) {
            return true
        }

        // by bishops
        if 0 < ((bb::diagonal_attacks(target, occupied) | bb::anti_diagonal_attacks(target, occupied)) 
                & board.bb_bishops(1 ^ color)) {
            return true
        }

        // by rooks
        if 0 < ((
                bb::rank_attacks(target, occupied) | 
                bb::file_attacks(target, occupied)
        ) & board.bb_rooks(1 ^ color)) {
            return true
        }

        // by queens
        if 0 < ((
                bb::diagonal_attacks(target, occupied) | 
                bb::anti_diagonal_attacks(target, occupied) |
                bb::rank_attacks(target, occupied) | 
                bb::file_attacks(target, occupied)
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

        if to_move == piece::WHITE {
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
        let mut pawns = board.bb_pawns(piece::WHITE);
        let mut moves = Vec::with_capacity(16);
        
        while 0 != pawns {
            let from = pawns.scan();
            let single_push = bb::north_one(bb::BB_SQUARES[from as usize]) & board.bb_empty();
            let double_push = bb::north_one(single_push) & board.bb_empty() & bb::BB_RANK_4;

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
                let to = double_push.scan() as u32;
                moves.push(Move::new(from, to, Move::make_flags(false, false, true, false)));
            }            
            pawns.clear_bit(from);
        }
        moves
    }

    #[inline]
    fn gen_black_pawn_pushes(board: &Board) -> Vec<Move> {
        let mut pawns = board.bb_pawns(piece::BLACK);
        let mut moves = Vec::with_capacity(16);
        
        while 0 != pawns {
            let from = pawns.scan();
            let single_push = bb::south_one(bb::BB_SQUARES[from as usize]) & board.bb_empty();
            let double_push = bb::south_one(single_push) & board.bb_empty() & bb::BB_RANK_5;

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
            pawns.clear_bit(from);
        }
        moves
    }

    #[inline]
    fn gen_white_pawn_captures(board: &Board) -> Vec<Move> {
        let mut pawns = board.bb_pawns(piece::WHITE);
        let mut moves = Vec::with_capacity(16);
        let mut ep_bb = bb::BB_EMPTY;
        let mut ep_square = 0;
        
        match board.en_passant() {
            Some(ep_squares) => {
                ep_square = ep_squares[0];
                ep_bb = bb::BB_SQUARES[ep_square as usize];
            },
            None => ()
        }

        while 0 != pawns {
            let from = pawns.scan();
            let mut atk = 
                (bb::north_west_one(bb::BB_SQUARES[from as usize]) |
                 bb::north_east_one(bb::BB_SQUARES[from as usize])) &
                (board.bb_opponent(piece::WHITE) | ep_bb);

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
                atk.clear_bit(to);
            }
            pawns.clear_bit(from);
        }

        moves
    }

    #[inline]
    fn gen_black_pawn_captures(board: &Board) -> Vec<Move> {
        let mut pawns = board.bb_pawns(piece::BLACK);
        let mut moves = Vec::with_capacity(16);
        let mut ep_bb = bb::BB_EMPTY;
        let mut ep_square = 0;
        
        match board.en_passant() {
            Some(ep_squares) => {
                ep_square = ep_squares[0];
                // ep_bb = bb::BB_SQUARES[squares::EP_CAPTURE_SQUARES[ep_square as usize]];
                ep_bb = bb::BB_SQUARES[ep_square as usize];
            },
            None => ()
        }

        while 0 != pawns {
            let from = pawns.scan();
            let mut atk = 
                (bb::north_west_one(bb::BB_SQUARES[from as usize]) |
                 bb::north_east_one(bb::BB_SQUARES[from as usize])) &
                (board.bb_opponent(piece::BLACK) | ep_bb);

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
                atk.clear_bit(to);
            }
            pawns.clear_bit(from);
        }

        moves
    }

    #[inline]
    fn gen_knight_moves(board: &Board, color: u32) -> Vec<Move> {
        let mut moves = Vec::with_capacity(16);
        let mut knights = board.bb_knights(color);

        while 0 != knights {
            let from = knights.scan();
            let mut mov = bb::BB_KNIGHT_ATTACKS[from as usize] & board.bb_empty();

            while 0 != mov {
                let to = mov.scan();
                moves.push(Move::new(from, to, Move::make_flags(false, false, false, false)));
                mov.clear_bit(to);
            }
            knights.clear_bit(from);
        }
        moves
    }

    #[inline]
    fn gen_knight_captures(board: &Board, color: u32) -> Vec<Move> {
        let mut moves = Vec::with_capacity(16);
        let mut knights = board.bb_knights(color);

        while 0 != knights {
            let from = knights.scan();
            let mut atk = bb::BB_KNIGHT_ATTACKS[from as usize] & board.bb_opponent(color);

            while 0 != atk {
                let to = atk.scan();
                moves.push(Move::new(from, to, Move::make_flags(true, false, false, false)));
                atk.clear_bit(to);
            }
            knights.clear_bit(from);
        }
        moves
    }

    #[inline]
    fn gen_wking_castle(board: &Board) -> Vec<Move> {
        let occ = board.bb_own(piece::WHITE) | board.bb_opponent(piece::WHITE);
        let mut moves = Vec::with_capacity(16);
        
        if MoveGenerator::is_attacked(board, piece::WHITE, squares::E1) {
            return moves
        }

        let qclear = 
            !occ.test_bit(squares::D1) && !occ.test_bit(squares::C1) &&
            !occ.test_bit(squares::B1) && !MoveGenerator::is_attacked(board, piece::WHITE, squares::D1) &&
            board.castling[piece::WHITE as usize].test_bit(1);

        let kclear = 
            !occ.test_bit(squares::F1) && !occ.test_bit(squares::G1) &&
            !MoveGenerator::is_attacked(board, piece::WHITE, squares::F1) &&
            board.castling[piece::WHITE as usize].test_bit(0);

        if qclear {
            moves.push(Move::new(squares::E1 as u32, squares::C1 as u32, Move::make_flags(false, false, true, true)));
        }
        if kclear {
            moves.push(Move::new(squares::E1 as u32, squares::G1 as u32, Move::make_flags(false, false, false, true)));
        }
        moves
    }

    #[inline]
    fn gen_bking_castle(board: &Board) -> Vec<Move> {
        let occ = board.bb_own(piece::BLACK) | board.bb_opponent(piece::BLACK);
        let mut moves = Vec::with_capacity(16);

        if MoveGenerator::is_attacked(board, piece::BLACK, squares::E8) {
            return moves
        }

        let qclear = 
            !occ.test_bit(squares::D8) && !occ.test_bit(squares::C8) &&
            !occ.test_bit(squares::B8) && !MoveGenerator::is_attacked(board, piece::BLACK, squares::D8) &&
            board.castling[piece::BLACK as usize].test_bit(1);

        let kclear = 
            !occ.test_bit(squares::F8) && !occ.test_bit(squares::G8) &&
            !MoveGenerator::is_attacked(board, piece::BLACK, squares::F8) &&
            board.castling[piece::BLACK as usize].test_bit(0);

        if qclear {
            moves.push(Move::new(squares::E8 as u32, squares::C8 as u32, Move::make_flags(false, false, true, true)));
        }
        if kclear {
            moves.push(Move::new(squares::E8 as u32, squares::G8 as u32, Move::make_flags(false, false, false, true)));
        }
        moves
    }

    #[inline]
    fn gen_king_moves(board: &Board, color: u32) -> Vec<Move> {
        let mut moves = Vec::with_capacity(16);
        let mut king = board.bb_king(color);

        while 0 != king {
            let from = king.scan();
            let mut mov = bb::BB_KING_ATTACKS[from as usize] & board.bb_empty();

            while 0 != mov {
                let to = mov.scan();
                moves.push(Move::new(from, to, Move::make_flags(false, false, false, false)));
                mov.clear_bit(to);
            }
            king.clear_bit(from);
        }
        moves
    }

    #[inline]
    fn gen_king_captures(board: &Board, color: u32) -> Vec<Move> {
        let mut moves = Vec::with_capacity(16);
        let mut king = board.bb_king(color);

        while 0 != king {
            let from = king.scan();
            let mut atk = bb::BB_KING_ATTACKS[from as usize] & board.bb_opponent(color);

            while 0 != atk {
                let to = atk.scan();
                moves.push(Move::new(from, to, Move::make_flags(true, false, false, false)));
                atk.clear_bit(to);
            }
            king.clear_bit(from);
        }
        moves
    }

    #[inline]
    fn gen_bishop_moves(board: &Board, color: u32) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        let mut bishops = board.bb_bishops(color);
        let occupied = board.bb_own(color) | board.bb_opponent(color);

        while 0 != bishops {
            let from = bishops.scan();
            let mut mov = (bb::diagonal_attacks(from, occupied) | bb::anti_diagonal_attacks(from, occupied)) & board.bb_empty();

            while 0 != mov {
                let to = mov.scan();
                moves.push(Move::new(from, to, Move::make_flags(false, false, false, false)));
                mov.clear_bit(to);
            }
            bishops.clear_bit(from);
        }
        moves
    }

    #[inline]
    fn gen_bishop_captures(board: &Board, color: u32) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        let mut bishops = board.bb_bishops(color);
        let occupied = board.bb_own(color) | board.bb_opponent(color);

        while 0 != bishops {
            let from = bishops.scan();
            let mut atk = (bb::diagonal_attacks(from, occupied) | bb::anti_diagonal_attacks(from, occupied)) & board.bb_opponent(color);

            while 0 != atk {
                let to = atk.scan();
                moves.push(Move::new(from, to, Move::make_flags(true, false, false, false)));
                atk.clear_bit(to);
            }
            bishops.clear_bit(from);
        }
        moves
    }

    #[inline]
    fn gen_rook_moves(board: &Board, color: u32) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        let mut rooks = board.bb_rooks(color);
        let occupied = board.bb_own(color) | board.bb_opponent(color);

        while 0 != rooks {
            let from = rooks.scan() ;
            let mut mov = (bb::rank_attacks(from, occupied) | bb::file_attacks(from, occupied)) & board.bb_empty();
            // let mut mov = (bb::rank_attacks(from, occupied)) & board.bb_empty();

            while 0 != mov {
                let to = mov.scan();
                moves.push(Move::new(from, to, Move::make_flags(false, false, false, false)));
                mov.clear_bit(to);
            }
            rooks.clear_bit(from);
        }
        moves
    }

    #[inline]
    fn gen_rook_captures(board: &Board, color: u32) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        let mut rooks = board.bb_rooks(color);
        let occupied = board.bb_own(color) | board.bb_opponent(color);

        while 0 != rooks {
            let from = rooks.scan();
            let mut atk = (bb::rank_attacks(from, occupied) | bb::file_attacks(from, occupied)) & board.bb_opponent(color);
            // let mut atk = (bb::rank_attacks(from, occupied)) & board.bb_opponent(color);

            while 0 != atk {
                let to = atk.scan();
                moves.push(Move::new(from, to, Move::make_flags(true, false, false, false)));
                atk.clear_bit(to);
            }
            rooks.clear_bit(from);
        }
        moves
    }

    #[inline]
    fn gen_queen_moves(board: &Board, color: u32) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        let mut queens = board.bb_queens(color);
        let occupied = board.bb_own(color) | board.bb_opponent(color);

        while 0 != queens {
            let from = queens.scan();
            let mut mov = board.bb_empty() & (
                bb::rank_attacks(from, occupied) |
                bb::file_attacks(from, occupied) |
                bb::diagonal_attacks(from, occupied) |
                bb::anti_diagonal_attacks(from, occupied)
            );
            while 0 != mov {
                let to = mov.scan();
                moves.push(Move::new(from, to, Move::make_flags(false, false, false, false)));
                mov.clear_bit(to);
            }
            queens.clear_bit(from);
        }
        moves
    }

    #[inline]
    fn gen_queen_captures(board: &Board, color: u32) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        let mut queens = board.bb_queens(color);
        let occupied = board.bb_own(color) | board.bb_opponent(color);

        while 0 != queens {
            let from = queens.scan();
            let mut atk = board.bb_opponent(color) & (
                bb::rank_attacks(from, occupied) |
                bb::file_attacks(from, occupied) |
                bb::diagonal_attacks(from, occupied) |
                bb::anti_diagonal_attacks(from, occupied)
            );

            while 0 != atk {
                let to = atk.scan();
                moves.push(Move::new(from, to, Move::make_flags(true, false, false, false)));
                atk.clear_bit(to);
            }
            queens.clear_bit(from);
        }
        moves
    }
}

#[cfg(test)]
mod tests {
    use uci::UCIInterface;
    use board::move_generator::MoveGenerator;
    use board::Board;
    use board::util::{squares, piece};

    #[test]
    fn it_generates_pawn_moves() {
        let mut board = Board::startpos();
        
        assert_eq!(16, MoveGenerator::gen_white_pawn_pushes(&board).len());
        board.input_move(squares::E2, squares::E4, None);
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

        board = Board::from_fen(String::from("1k6/3p4/8/4P/8/8/8/6K1 b - - 0 1 moves d7d5")).unwrap();
        assert_eq!(1, MoveGenerator::gen_white_pawn_captures(&board).len());
        assert_eq!(1, MoveGenerator::gen_white_pawn_pushes(&board).len());
        // assert_eq!(2, MoveGenerator::from_board(&board).len());
    }

    #[test]
    fn it_generates_king_moves() {
        // position startpos moves e2e4 e7e5 g1f3 g8f6 f1d3 f8d6
        let mut board = Board::startpos();
        board.input_move(squares::E2, squares::E4, None);
        board.input_move(squares::E7, squares::E5, None);
        board.input_move(squares::G1, squares::F3, None);
        board.input_move(squares::G8, squares::F6, None);
        board.input_move(squares::F1, squares::D3, None);
        board.input_move(squares::F8, squares::D6, None);
        assert_eq!(2, MoveGenerator::gen_king_moves(&board, piece::WHITE).len());
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