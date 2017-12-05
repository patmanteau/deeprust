use engine::board::Board;
use engine::moves::Move;
use engine::util::{bb, piece, squares};
use bits;

pub struct MoveGenerator {
    moves: Vec<Move>,
    cur: usize,
}

impl MoveGenerator {
    pub fn new() -> MoveGenerator {
        MoveGenerator{
            moves: Vec::with_capacity(512), //[Move::new(0, 0, 0); 512],
            cur: 0, 
        }
    }
    
    pub fn from_board(&mut self, board: &Board) {
        self.moves = Vec::with_capacity(512);

        if board.to_move() == piece::WHITE {
            let w_pawn_pushes = self.gen_white_pawn_pushes(board);
            self.moves.extend(w_pawn_pushes);

            let w_pawn_captures = self.gen_white_pawn_captures(board);
            self.moves.extend(w_pawn_captures);

        } else {
            let b_pawn_pushes = self.gen_black_pawn_pushes(board);
            self.moves.extend(b_pawn_pushes);
        }

        let knight_captures = self.gen_knight_captures(board);
        self.moves.extend(knight_captures);
        let knight_moves = self.gen_knight_moves(board);
        self.moves.extend(knight_moves);

        let bishop_captures = self.gen_bishop_captures(board);
        self.moves.extend(bishop_captures);
        let bishop_moves = self.gen_bishop_moves(board);
        self.moves.extend(bishop_moves);

        let rook_captures = self.gen_rook_captures(board);
        self.moves.extend(rook_captures);
        let rook_moves = self.gen_rook_moves(board);
        self.moves.extend(rook_moves);

        let queen_captures = self.gen_queen_captures(board);
        self.moves.extend(queen_captures);
        let queen_moves = self.gen_queen_moves(board);
        self.moves.extend(queen_moves);

        let king_captures = self.gen_king_captures(board);
        self.moves.extend(king_captures);
        let king_moves = self.gen_king_moves(board);
        self.moves.extend(king_moves);
    }

    pub fn moves(&self) -> &Vec<Move> {
        &self.moves
    }

    pub fn count(&self) -> usize {
        self.moves.len()
    }

    fn gen_white_pawn_pushes(&self, board: &Board) -> Vec<Move> {
        let mut pawns = board.bb_pawns();
        let mut moves = Vec::with_capacity(16);
        let single_pushes = bb::north_one(pawns) & board.bb_empty();
        let double_pushes = bb::north_one(single_pushes) & board.bb_empty() & bb::BB_RANK_4;

        while 0 != pawns {
            let from = bits::count_trailing_zeros(pawns) as u32;
            
            // double push, mask by file
            let double_push = double_pushes & bb::BB_FILES[(from & 0x7) as usize];
            if 0 != double_push {
                let to = bits::count_trailing_zeros(double_push) as u32;
                moves.push(Move::new(from, to, Move::make_flags(false, false, true, false)));
            }
            
            // single push, mask by file
            let single_push = single_pushes & bb::BB_FILES[(from & 0x7) as usize];
            if 0 != single_push {
                let to = bits::count_trailing_zeros(single_push) as u32;
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
            pawns = bits::clear_least_significant_one(pawns);
        }
        moves
    }

    fn gen_black_pawn_pushes(&self, board: &Board) -> Vec<Move> {
        let mut pawns = board.bb_pawns();
        let mut moves = Vec::with_capacity(16);
        let single_pushes = bb::south_one(pawns) & board.bb_empty();
        let double_pushes = bb::south_one(single_pushes) & board.bb_empty() & bb::BB_RANK_5;

        while 0 != pawns {
            let from = bits::count_trailing_zeros(pawns) as u32;
            
            // double push, mask by file
            let double_push = double_pushes & bb::BB_FILES[(from & 0x7) as usize];
            if 0 != double_push {
                let to = bits::count_trailing_zeros(double_push) as u32;
                moves.push(Move::new(from, to, Move::make_flags(false, false, true, false)));
            }
            
            // single push, mask by file
            let single_push = single_pushes & bb::BB_FILES[(from & 0x7) as usize];
            if 0 != single_push {
                let to = bits::count_trailing_zeros(single_push) as u32;
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
            pawns = bits::clear_least_significant_one(pawns);
        }
        moves
    }

    fn gen_white_pawn_captures(&self, board: &Board) -> Vec<Move> {
        let mut pawns = board.bb_pawns();
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
            let from = bits::count_trailing_zeros(pawns) as u32;
            let mut atk = 
                (bb::north_west_one(bb::BB_SQUARES[from as usize]) |
                 bb::north_east_one(bb::BB_SQUARES[from as usize])) &
                (board.bb_opponent() | ep_bb);

            while 0 != atk {
                let to = bits::count_trailing_zeros(atk) as u32;
                moves.push(Move::new(from, to, Move::make_flags(true, false, to == ep_square, false)));
                atk = bits::clear_least_significant_one(atk);
            }
            pawns = bits::clear_least_significant_one(pawns);
        }

        moves
    }

    fn gen_knight_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::with_capacity(16);
        let mut knights = board.bb_knights();

        while 0 != knights {
            let from = bits::count_trailing_zeros(knights) as u32;
            let mut mov = bb::BB_KNIGHT_ATTACKS[from as usize] & board.bb_empty();

            while 0 != mov {
                let to = bits::count_trailing_zeros(mov) as u32;
                moves.push(Move::new(from, to, Move::make_flags(false, false, false, false)));
                mov = bits::clear_least_significant_one(mov);
            }
            knights = bits::clear_least_significant_one(knights);
        }
        moves
    }

    fn gen_knight_captures(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::with_capacity(16);
        let mut knights = board.bb_knights();

        while 0 != knights {
            let from = bits::count_trailing_zeros(knights) as u32;
            let mut atk = bb::BB_KNIGHT_ATTACKS[from as usize] & board.bb_opponent();

            while 0 != atk {
                let to = bits::count_trailing_zeros(atk) as u32;
                moves.push(Move::new(from, to, Move::make_flags(true, false, false, false)));
                atk = bits::clear_least_significant_one(atk);
            }
            knights = bits::clear_least_significant_one(knights);
        }
        moves
    }

    fn gen_king_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::with_capacity(16);
        let mut king = board.bb_king();

        while 0 != king {
            let from = bits::count_trailing_zeros(king) as u32;
            let mut mov = bb::BB_KING_ATTACKS[from as usize] & board.bb_empty();

            // castling
            //
            // this code REALLY assumes that board.castling is set correctly,
            // it doesn't check at all where the king really sits as long as
            // castling is allowed
            if 0 != board.castling()[board.to_move() as usize] & 0x1 {
                let castle_from = [squares::E1 as u32, squares::E8 as u32][board.to_move() as usize];
                let castle_to = [squares::G1 as u32, squares::G8 as u32][board.to_move() as usize];
                let castle = bb::east_one(bb::east_one(bb::BB_SQUARES[castle_from as usize]) & board.bb_empty()) & board.bb_empty();
                if 0 != castle {
                    moves.push(Move::new(castle_from as u32, castle_to as u32, Move::make_flags(false, false, false, true)));
                }
            }
            if 0 != board.castling()[board.to_move() as usize] & 0x2 {
                let castle_from: u64 = [squares::E1 as u64, squares::E8 as u64][board.to_move() as usize];
                let castle_to: u64 = [squares::C1 as u64, squares::C8 as u64][board.to_move() as usize];
                let castle = bb::west_one(bb::west_one(bb::west_one(bb::BB_SQUARES[castle_from as usize]) & board.bb_empty()) & board.bb_empty()) & board.bb_empty();
                if 0 != castle {
                    moves.push(Move::new(castle_from as u32, castle_to as u32, Move::make_flags(false, false, true, true)));
                }
            }

            while 0 != mov {
                let to = bits::count_trailing_zeros(mov) as u32;
                moves.push(Move::new(from, to, Move::make_flags(false, false, false, false)));
                mov = bits::clear_least_significant_one(mov);
            }
            king = bits::clear_least_significant_one(king);
        }
        moves
    }

    fn gen_king_captures(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::with_capacity(16);
        let mut king = board.bb_king();

        while 0 != king {
            let from = bits::count_trailing_zeros(king) as u32;
            let mut atk = bb::BB_KING_ATTACKS[from as usize] & board.bb_opponent();

            while 0 != atk {
                let to = bits::count_trailing_zeros(atk) as u32;
                moves.push(Move::new(from, to, Move::make_flags(true, false, false, false)));
                atk = bits::clear_least_significant_one(atk);
            }
            king = bits::clear_least_significant_one(king);
        }
        moves
    }

    fn gen_bishop_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        let mut bishops = board.bb_bishops();
        let occupied = board.bb_own() | board.bb_opponent();

        while 0 != bishops {
            let from = bits::count_trailing_zeros(bishops) as u32;
            let mut mov = (bb::diagonal_attacks(from, occupied) | bb::anti_diagonal_attacks(from, occupied)) & board.bb_empty();

            while 0 != mov {
                let to = bits::count_trailing_zeros(mov) as u32;
                moves.push(Move::new(from, to, Move::make_flags(false, false, false, false)));
                mov = bits::clear_least_significant_one(mov);
            }
            bishops = bits::clear_least_significant_one(bishops);
        }
        moves
    }

    fn gen_bishop_captures(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        let mut bishops = board.bb_bishops();
        let occupied = board.bb_own() | board.bb_opponent();

        while 0 != bishops {
            let from = bits::count_trailing_zeros(bishops) as u32;
            let mut atk = (bb::diagonal_attacks(from, occupied) | bb::anti_diagonal_attacks(from, occupied)) & board.bb_opponent();

            while 0 != atk {
                let to = bits::count_trailing_zeros(atk) as u32;
                moves.push(Move::new(from, to, Move::make_flags(true, false, false, false)));
                atk = bits::clear_least_significant_one(atk);
            }
            bishops = bits::clear_least_significant_one(bishops);
        }
        moves
    }

    fn gen_rook_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        let mut rooks = board.bb_rooks();
        let occupied = board.bb_own() | board.bb_opponent();

        while 0 != rooks {
            let from = bits::count_trailing_zeros(rooks) as u32;
            let mut mov = (bb::rank_attacks(from, occupied) | bb::file_attacks(from, occupied)) & board.bb_empty();

            while 0 != mov {
                let to = bits::count_trailing_zeros(mov) as u32;
                moves.push(Move::new(from, to, Move::make_flags(false, false, false, false)));
                mov = bits::clear_least_significant_one(mov);
            }
            rooks = bits::clear_least_significant_one(rooks);
        }
        moves
    }

    fn gen_rook_captures(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        let mut rooks = board.bb_rooks();
        let occupied = board.bb_own() | board.bb_opponent();

        while 0 != rooks {
            let from = bits::count_trailing_zeros(rooks) as u32;
            let mut atk = (bb::rank_attacks(from, occupied) | bb::file_attacks(from, occupied)) & board.bb_opponent();

            while 0 != atk {
                let to = bits::count_trailing_zeros(atk) as u32;
                moves.push(Move::new(from, to, Move::make_flags(true, false, false, false)));
                atk = bits::clear_least_significant_one(atk);
            }
            rooks = bits::clear_least_significant_one(rooks);
        }
        moves
    }

    fn gen_queen_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        let mut queens = board.bb_queens();
        let occupied = board.bb_own() | board.bb_opponent();

        while 0 != queens {
            let from = bits::count_trailing_zeros(queens) as u32;
            let mut mov = board.bb_empty() & (
                bb::rank_attacks(from, occupied) |
                bb::file_attacks(from, occupied) |
                bb::diagonal_attacks(from, occupied) |
                bb::anti_diagonal_attacks(from, occupied)
            );
            while 0 != mov {
                let to = bits::count_trailing_zeros(mov) as u32;
                moves.push(Move::new(from, to, Move::make_flags(false, false, false, false)));
                mov = bits::clear_least_significant_one(mov);
            }
            queens = bits::clear_least_significant_one(queens);
        }
        moves
    }

    fn gen_queen_captures(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);
        let mut queens = board.bb_queens();
        let occupied = board.bb_own() | board.bb_opponent();

        while 0 != queens {
            let from = bits::count_trailing_zeros(queens) as u32;
            let mut atk = board.bb_opponent() & (
                bb::rank_attacks(from, occupied) |
                bb::file_attacks(from, occupied) |
                bb::diagonal_attacks(from, occupied) |
                bb::anti_diagonal_attacks(from, occupied)
            );

            while 0 != atk {
                let to = bits::count_trailing_zeros(atk) as u32;
                moves.push(Move::new(from, to, Move::make_flags(true, false, false, false)));
                atk = bits::clear_least_significant_one(atk);
            }
            queens = bits::clear_least_significant_one(queens);
        }
        moves
    }
}

#[cfg(test)]
mod tests {
    use engine::move_generator::MoveGenerator;
    use engine::board::Board;
    use engine::util::squares;

    #[test]
    fn it_generates_pawn_moves() {
        let mut gen = MoveGenerator::new();
        let mut board = Board::startpos();
        
        assert_eq!(16, gen.gen_white_pawn_pushes(&board).len());
        board.input_move(squares::E2, squares::E4, None);
        assert_eq!(16, gen.gen_black_pawn_pushes(&board).len());

        board = Board::from_fen(String::from("8/PPPPPPPP/8/8/8/8/8/8 w - - 0 1")).unwrap();
        assert_eq!(32, gen.gen_white_pawn_pushes(&board).len());
        board = Board::from_fen(String::from("8/8/8/8/8/8/pppppppp/8 b - - 0 1")).unwrap();
        assert_eq!(32, gen.gen_black_pawn_pushes(&board).len());
    }

    #[test]
    fn it_generates_pawn_captures() {
        let mut gen = MoveGenerator::new();
        let mut board = Board::from_fen(String::from("8/8/8/p1p1p1p1/P1P1P1P1/8/8/8 w - - 0 1")).unwrap();
        assert_eq!(0, gen.gen_white_pawn_captures(&board).len());

        board = Board::from_fen(String::from("8/8/8/p1p1p1p1/1P1P1P1P/8/8/8 w - - 0 1")).unwrap();
        assert_eq!(7, gen.gen_white_pawn_captures(&board).len());
        assert_eq!(4, gen.gen_white_pawn_pushes(&board).len());
        gen.from_board(&board);
        assert_eq!(11, gen.moves().len());

        board = Board::from_fen(String::from("8/3p4/8/4P/8/8/8/8 b - - 0 1 moves d7d5")).unwrap();
        // assert_eq!(1, gen.gen_white_pawn_captures(&board).len());
        // assert_eq!(4, gen.gen_white_pawn_pushes(&board).len());
        gen.from_board(&board);
        assert_eq!(2, gen.moves().len());
    }

    #[test]
    fn it_generates_king_moves() {
        let mut gen = MoveGenerator::new();

        // position startpos moves e2e4 e7e5 g1f3 g8f6 f1d3 f8d6
        let mut board = Board::startpos();
        board.input_move(squares::E2, squares::E4, None);
        board.input_move(squares::E7, squares::E5, None);
        board.input_move(squares::G1, squares::F3, None);
        board.input_move(squares::G8, squares::F6, None);
        board.input_move(squares::F1, squares::D3, None);
        board.input_move(squares::F8, squares::D6, None);
        assert_eq!(3, gen.gen_king_moves(&board).len());
    }


    
    
}