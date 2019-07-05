use crate::bitboard::*;
use crate::color::{self, Color};
use crate::common::BitTwiddling;
use crate::moves::Move;
use crate::piece::{self, Piece, PiecePrimitives};
use crate::square::{self, Square, SquarePrimitives};
use std::fmt;

pub type PositionStack = Vec<Position>;

/// Represents a chess position
///
/// Uses 16 bitboards ((2 colors + 6 pieces) * (unflipped + flipped)) plus an occupancy array
///
/// 204 Byte
#[derive(Clone, Copy)]
pub struct Position {
    // 8 * 8 * 2 = 128 Byte
    // pub bb: [[Bitboard; 8]; 2],
    // 14 * 8 = 112 Byte
    pub bb: [Bitboard; 14],
    //  1 * 64 = 64 Byte
    pub occupied: [Piece; 64],
    // 8 Byte
    pub to_move: Color,
    // 4 * 2 = 8 Byte
    pub castling: [u32; 2],
    // // 4 * 2 = 8 Byte
    // pub en_passant: Option<[Square; 2]>,
    // 4 Byte
    pub en_passant: Option<Square>,
    // 4 Byte
    pub halfmoves: u32,
    // 4 Byte
    pub fullmoves: u32,
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Position {{ bb[0]: {:#?} }}",
            //self.to_fen(),
            self.bb
        )
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "occupied: ").unwrap();
        for rank in (0..8).rev() {
            writeln!(f, "+---+---+---+---+---+---+---+---+").unwrap();
            for file in 0..8 {
                write!(
                    f,
                    "| {} ",
                    self.occupied[(rank << 3) + file].to_san_string()
                )
                .unwrap();
            }
            writeln!(f, "| {} ", rank + 1).unwrap();
        }
        writeln!(f, "+---+---+---+---+---+---+---+---+").unwrap();
        writeln!(f, "  A   B   C   D   E   F   G   H  ").unwrap();
        writeln!(f).unwrap();
        //writeln!(f, "fen: {}", self.to_fen()).unwrap();
        writeln!(f, "to_move: {}", self.to_move).unwrap();
        writeln!(f).unwrap();

        let bb_titles: [&'static str; 2] = [
            "bb_own      bb_opponent bb_pawns    bb_knights",
            "bb_bishops  bb_rooks    bb_queens   bb_king",
        ];

        for (block, title) in bb_titles.iter().enumerate() {
            writeln!(f, "{}", title).unwrap();
            for rank in (0..8).rev() {
                for cur_bb in 0..4 {
                    write!(
                        f,
                        "{}    ",
                        self.bb[(block * 4) + cur_bb].rank_to_debug_string(rank)
                    )
                    .unwrap();
                }
                writeln!(f).unwrap();
            }
            writeln!(f).unwrap();
        }
        write!(f, "")
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new()
    }
}

impl Position {
    pub fn new() -> Position {
        Position {
            bb: [0; 14],
            occupied: [0; 64],
            to_move: color::WHITE,
            castling: [0, 0],
            en_passant: None,
            halfmoves: 0,
            fullmoves: 1,
        }
    }

    pub fn equals(&self, rhs: &Position) -> bool {
        for bb in 0..14 {
            if self.bb[bb] != rhs.bb[bb] {
                return false;
            }
        }
        for side in 0..2 {
            if self.castling[side] != rhs.castling[side] {
                return false;
            }
        }
        for sq in 0..64 {
            if self.occupied[sq] != rhs.occupied[sq] {
                return false;
            }
        }
        if self.to_move != rhs.to_move {
            return false;
        }
        if self.en_passant != rhs.en_passant {
            return false;
        }
        if self.halfmoves != rhs.halfmoves {
            return false;
        }
        if self.fullmoves != rhs.fullmoves {
            return false;
        }
        true
    }

    fn panic_helper(&self) {
        eprintln!("{}", self);
        panic!();
    }

    #[cfg(feature = "sanity_checks")]
    fn sanity_check(&self) -> bool {
        // disjoint color bbs?
        if self.bb_own(color::WHITE) & self.bb_opponent(color::WHITE) > 0 {
            self.panic_helper()
        }

        for i in 2..8 {
            // piece bbs not in color bbs?
            if !(self.bb_own(color::WHITE) | self.bb_opponent(color::WHITE)) & self.bb[0][i] > 0 {
                self.panic_helper()
            }
            // same bit set in multiple color bbs?
            for j in i + 1..8 {
                if self.bb[0][i] & self.bb[0][j] > 0 {
                    self.panic_helper()
                }
            }
        }

        // wrong number of kings?
        if self.bb_king(color::WHITE).count() != 1 || self.bb_king(color::BLACK).count() != 1 {
            self.panic_helper()
        }

        // too many pawns?
        if self.bb_pawns(color::WHITE).count() > 8 || self.bb_pawns(color::BLACK).count() > 8 {
            self.panic_helper()
        }

        true
    }

    #[inline]
    fn bb_idx(color: Color, piece: Piece) -> usize {
        usize::from((color * 6) + piece)
    }

    #[inline]
    pub fn bb(&self) -> &[Bitboard; 14] {
        &self.bb
    }

    // don't actually return flipped boards for now
    #[inline]
    pub fn bb_own(&self, color: Color) -> Bitboard {
        self.bb[color as usize]
    }

    #[inline]
    pub fn bb_opponent(&self, color: Color) -> Bitboard {
        self.bb[(1 ^ color) as usize]
    }

    #[inline]
    pub fn bb_pawns(&self, color: Color) -> Bitboard {
        self.bb[Self::bb_idx(color, piece::PAWN)]
        // self.bb[0][piece::PAWN as usize] & self.bb_own(color)
    }

    #[inline]
    pub fn bb_knights(&self, color: Color) -> Bitboard {
        self.bb[Self::bb_idx(color, piece::KNIGHT)]
        // self.bb[0][piece::KNIGHT as usize] & self.bb_own(color)
    }

    #[inline]
    pub fn bb_bishops(&self, color: Color) -> Bitboard {
        self.bb[Self::bb_idx(color, piece::BISHOP)]
        // self.bb[0][piece::BISHOP as usize] & self.bb_own(color)
    }

    #[inline]
    pub fn bb_rooks(&self, color: Color) -> Bitboard {
        self.bb[Self::bb_idx(color, piece::ROOK)]
        // self.bb[0][piece::ROOK as usize] & self.bb_own(color)
    }

    #[inline]
    pub fn bb_queens(&self, color: Color) -> Bitboard {
        self.bb[Self::bb_idx(color, piece::QUEEN)]
        // self.bb[0][piece::QUEEN as usize] & self.bb_own(color)
    }

    #[inline]
    pub fn bb_king(&self, color: Color) -> Bitboard {
        self.bb[Self::bb_idx(color, piece::KING)]
        // self.bb[0][piece::KING as usize] & self.bb_own(color)
    }

    #[inline]
    pub fn bb_empty(&self) -> Bitboard {
        !(self.bb_own(color::WHITE) | self.bb_opponent(color::WHITE))
    }

    #[inline]
    pub fn to_move(&self) -> Color {
        self.to_move
    }

    #[inline]
    pub fn castling(&self) -> [u32; 2] {
        self.castling
    }

    #[inline]
    pub fn en_passant(&self) -> Option<Square> {
        self.en_passant
    }

    #[inline]
    pub fn occupied(&self) -> &[Piece; 64] {
        &self.occupied
    }

    #[inline]
    pub fn halfmoves(&self) -> u32 {
        self.halfmoves
    }

    #[inline]
    pub fn fullmoves(&self) -> u32 {
        self.fullmoves
    }

    #[inline]
    pub fn get_piece_and_color(&self, square: Square) -> (Piece, Color) {
        (
            self.occupied[square as usize].code(),
            self.occupied[square as usize].color(),
        )
    }

    #[inline]
    pub fn check_piece(&self, piece: Piece, color: Color, square: Square) -> bool {
        (piece, color) == self.get_piece_and_color(square)
    }

    #[inline]
    pub fn set_piece(&mut self, piece: Piece, color: Color, to: Square) {
        self.bb[color as usize].set(to);
        self.bb[Self::bb_idx(color, piece)].set(to);
        // self.bb[piece as usize].set(to);

        self.occupied[to as usize] = Piece::new(piece, color);
    }

    #[inline]
    pub fn remove_piece(&mut self, piece: Piece, color: Color, from: Square) {
        self.bb[color as usize].clear(from);
        self.bb[Self::bb_idx(color, piece)].clear(from);

        self.occupied[from as usize] = 0;
    }

    #[inline]
    pub fn quiet_move_piece(&mut self, piece: Piece, color: Color, from: Square, to: Square) {
        // see https://www.chessprogramming.org/General_Setwise_Operations
        let from_bb = u64::bit_at(u32::from(from));
        let to_bb = u64::bit_at(u32::from(to));
        let from_to_bb = from_bb ^ to_bb;

        self.bb[color as usize] ^= from_to_bb;
        self.bb[Self::bb_idx(color, piece)] ^= from_to_bb;

        self.occupied[to as usize] = self.occupied[from as usize];
        self.occupied[from as usize] = 0;
    }

    #[inline]
    pub fn capture_move_piece(
        &mut self,
        piece: Piece,
        color: Color,
        captured_piece: Piece,
        from: Square,
        to: Square,
    ) {
        // see https://www.chessprogramming.org/General_Setwise_Operations
        let captured_color = 1 ^ color;
        let from_bb = u64::bit_at(u32::from(from));
        let to_bb = u64::bit_at(u32::from(to));
        let from_to_bb = from_bb ^ to_bb;

        self.bb[color as usize] ^= from_to_bb;
        self.bb[Self::bb_idx(color, piece)] ^= from_to_bb;

        self.bb[captured_color as usize] ^= to_bb;
        self.bb[Self::bb_idx(captured_color, captured_piece)] ^= to_bb;

        self.occupied[to as usize] = self.occupied[from as usize];
        self.occupied[from as usize] = 0;
    }

    #[inline]
    pub fn replace_piece(
        &mut self,
        old_piece: Piece,
        old_color: Color,
        new_piece: Piece,
        new_color: Color,
        square: Square,
    ) {
        self.bb[old_color as usize].clear(square);
        self.bb[Self::bb_idx(old_color, old_piece)].clear(square);

        self.set_piece(new_piece, new_color, square);
    }

    pub fn make_move(&mut self, mov: Move) {
        let orig_square = mov.orig();
        let dest_square = mov.dest();
        let (orig_piece, orig_color) = self.get_piece_and_color(orig_square);
        let (mut dest_piece, mut dest_color) = self.get_piece_and_color(dest_square);
        let is_capture = mov.is_capture();

        #[cfg(feature = "sanity_checks")]
        {
            if orig_color != self.to_move {
                eprintln!("orig_color != self.to_move");
                eprintln!("offending move: {:?}", mov);
                self.panic_helper();
            }
        }

        // reset en passant
        self.en_passant = None;

        // promotions change pieces
        if mov.is_promotion() {
            let prom_piece = mov.prom_piece_code();
            self.remove_piece(orig_piece, orig_color, orig_square);
            if is_capture {
                self.replace_piece(dest_piece, dest_color, prom_piece, orig_color, dest_square);
            } else {
                self.set_piece(prom_piece, orig_color, dest_square);
            }
        } else if mov.is_quiet() {
            self.quiet_move_piece(orig_piece, orig_color, orig_square, dest_square);
        } else if mov.is_capture_en_passant() {
            self.remove_piece(orig_piece, orig_color, orig_square);
            dest_piece = piece::PAWN;
            dest_color = 1 ^ orig_color;
            self.remove_piece(
                dest_piece,
                dest_color,
                square::ep_capture_square(dest_square),
            );
            self.set_piece(orig_piece, orig_color, dest_square);
        } else if is_capture {
            self.capture_move_piece(orig_piece, orig_color, dest_piece, orig_square, dest_square);
        // self.remove_piece(orig_piece, orig_color, orig_square);
        // self.replace_piece(dest_piece, dest_color, orig_piece, orig_color, dest_square);
        } else if mov.is_double_pawn_push() {
            let new_ep_square =
                (i64::from(dest_square) - [8i64, -8i64][orig_color as usize]) as Square;
            self.en_passant = Some(new_ep_square);
            self.quiet_move_piece(orig_piece, orig_color, orig_square, dest_square);
        } else if mov.is_king_castle() {
            self.quiet_move_piece(orig_piece, orig_color, orig_square, dest_square);
            self.quiet_move_piece(piece::ROOK, orig_color, dest_square + 1, dest_square - 1);
        } else if mov.is_queen_castle() {
            self.quiet_move_piece(orig_piece, orig_color, orig_square, dest_square);
            self.quiet_move_piece(piece::ROOK, orig_color, dest_square - 2, dest_square + 1);
        } else {
            panic!("shouldn't come here")
        }

        // clear castling rights on king or rook move
        // let orig_bb = BB_SQUARES[orig_square as usize];
        let orig_bb = Bitboard::bit_at(u32::from(orig_square));
        if piece::KING == orig_piece {
            self.castling[self.to_move as usize].clear_bit(0);
            self.castling[self.to_move as usize].clear_bit(1);
        } else if orig_piece == piece::ROOK && (orig_bb & BB_ROOK_HOMES[self.to_move as usize] > 0)
        {
            if orig_bb & BB_FILE_A > 0 {
                self.castling[self.to_move as usize].clear_bit(1);
            } else {
                self.castling[self.to_move as usize].clear_bit(0);
            }
        }

        // clear castling rights on rook capture at home square
        // let dest_bb = BB_SQUARES[dest_square as usize];
        let dest_bb = Bitboard::bit_at(u32::from(dest_square));
        if dest_piece == piece::ROOK && (dest_bb & BB_ROOK_HOMES[1 ^ self.to_move as usize] > 0) {
            if dest_bb & BB_FILE_A > 0 {
                self.castling[(1 ^ self.to_move) as usize].clear_bit(1);
            } else {
                self.castling[(1 ^ self.to_move) as usize].clear_bit(0);
            }
        }

        // Full move clock needs to be incremented after black moves
        // piece::WHITE == 0 and piece::BLACK == 1, so we use that to save an if :-)
        self.fullmoves += u32::from(self.to_move);

        // set half move clock
        if orig_piece == piece::PAWN || is_capture {
            self.halfmoves = 0; // reset half move clock on pawn moves and captures
        } else {
            self.halfmoves += 1;
        }

        // flip to move
        self.to_move ^= 1;

        #[cfg(feature = "sanity_checks")]
        self.sanity_check();
    }

    pub fn input_move(
        &mut self,
        orig: Square,
        dest: Square,
        promote_to: Option<Piece>,
    ) -> Result<Move, &'static str> {
        let (mut is_capture, mut is_promotion, mut is_special_0, mut is_special_1) =
            (false, false, false, false);
        let (piece, _color) = self.get_piece_and_color(orig);
        let (dest_piece, _dest_color) = self.get_piece_and_color(dest);
        if 0 == piece {
            return Err("No piece at given square");
        };

        // let (cap_piece, _) = self.get_piece_and_color(dest);
        // is_capture = 0 != cap_piece;

        // set special flags for double pawn push
        if piece == piece::PAWN && ((orig + 16 == dest) || (dest + 16 == orig)) {
            is_special_0 = true;
            is_special_1 = false;
        }

        // set flags for en passant capture
        if piece == piece::PAWN && Some(dest) == self.en_passant {
            is_capture = true;
            is_special_0 = true;
            is_special_1 = false;
        }

        // set flags for capture
        if dest_piece >= 2 {
            is_capture = true;
        }

        // set flags for promotion
        if let Some(promoted_piece) = promote_to {
            is_special_0 = false;
            is_special_1 = false;
            is_promotion = true;
            if (piece::BISHOP == promoted_piece) || (piece::QUEEN == promoted_piece) {
                is_special_0 = true;
            }
            if (piece::ROOK == promoted_piece) || (piece::QUEEN == promoted_piece) {
                is_special_1 = true;
            }
        }

        // set flags for castling
        if piece == piece::KING {
            if 2 == dest.wrapping_sub(orig) {
                // King castle
                is_special_0 = false;
                is_special_1 = true;
            } else if 2 == orig.wrapping_sub(dest) {
                // Queen castle
                is_special_0 = true;
                is_special_1 = true;
            }
        }

        let mov = Move::new(
            orig,
            dest,
            Move::make_flags(is_capture, is_promotion, is_special_0, is_special_1),
        );
        self.make_move(mov);
        Ok(mov)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboard as bb;

    #[test]
    fn it_sets_pieces() {
        // full board
        let mut position = Position::new();
        for square in 0..64 {
            for color in 0..2 {
                for piece in 2..8 {
                    position.set_piece(piece, color, square);
                    assert!(position.check_piece(piece, color, square));
                    assert!(position.bb[color as usize] & bb::BB_SQUARES[square as usize] != 0);
                    assert!(position.bb[Position::bb_idx(color, piece)] & bb::BB_SQUARES[square as usize] != 0);
                    // assert!(
                    //     position.bb[1][color as usize] & bb::BB_SQUARES[(square ^ 56) as usize]
                    //         != 0
                    // );
                    // assert!(
                    //     position.bb[1][piece as usize] & bb::BB_SQUARES[(square ^ 56) as usize]
                    //         != 0
                    // );
                    assert_eq!(piece, position.occupied[square as usize].code());
                    assert_eq!(color, position.occupied[square as usize].color());

                    //assert_eq!(position.bb[1][color as usize], bits::swap_bytes(position.bb[0][color as usize]));
                    //assert_eq!(position.bb[1][piece as usize], bits::swap_bytes(position.bb[0][piece as usize]));

                    // assert_eq!(
                    //     position.bb[1][color as usize],
                    //     position.bb[0][color as usize].swap_bytes()
                    // );
                    // assert_eq!(
                    //     position.bb[1][piece as usize],
                    //     position.bb[0][piece as usize].swap_bytes()
                    // );
                }
            }
        }

        // single pieces
        for square in 0..64 {
            for color in 0..2 {
                for piece in 2..8 {
                    let mut position = Position::new();
                    position.set_piece(piece, color, square);
                    assert!(position.check_piece(piece, color, square));
                    assert!(position.bb[color as usize] & bb::BB_SQUARES[square as usize] != 0);
                    assert!(position.bb[Position::bb_idx(color, piece)] & bb::BB_SQUARES[square as usize] != 0);
                    // assert!(
                    //     position.bb[1][color as usize] & bb::BB_SQUARES[(square ^ 56) as usize]
                    //         != 0
                    // );
                    // assert!(
                    //     position.bb[1][piece as usize] & bb::BB_SQUARES[(square ^ 56) as usize]
                    //         != 0
                    // );
                    assert_eq!(piece, position.occupied[square as usize].code());
                    assert_eq!(color, position.occupied[square as usize].color());

                    // assert_eq!(
                    //     position.bb[1][color as usize],
                    //     position.bb[0][color as usize].swap_bytes()
                    // );
                    // assert_eq!(
                    //     position.bb[1][piece as usize],
                    //     position.bb[0][piece as usize].swap_bytes()
                    // );
                }
            }
        }
    }

    #[test]
    fn it_calculates_ep_squares_correctly() {
        for x in 0..8 {
            // white
            assert_eq!(
                Square::from_coords(x, 3),
                square::ep_capture_square(Square::from_coords(x, 2))
            );
            // black
            assert_eq!(
                Square::from_coords(x, 4),
                square::ep_capture_square(Square::from_coords(x, 5))
            );
        }
    }

    // #[test]
    // fn it_makes_moves() {
    //     if let Ok(mut position) = Position::from_fen(String::from(
    //         "rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 1",
    //     )) {
    //         position.input_move(square::D7, square::D6, None).unwrap();
    //         assert_eq!(None, position.en_passant);
    //     }

    //     let mut position = Position::from_fen(String::from("8/3p4/8/4P/8/8/8/8 b - - 0 1")).unwrap();
    //     position.input_move(square::D7, square::D5, None).unwrap();
    //     position.input_move(square::E5, square::D6, None).unwrap();
    //     assert_eq!(0, position.occupied[square::D5 as usize]);
    // }
}
