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
#[derive(Clone, Copy)]
pub struct Position {
    pub bb: [[Bitboard; 8]; 2],
    pub occupied: [Piece; 64],
    pub to_move: Color,
    pub castling: [u32; 2],
    pub en_passant: Option<[Square; 2]>,
    pub halfmoves: u32,
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
                        self.bb[0][(block * 4) + cur_bb].rank_to_debug_string(rank)
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
            bb: [[0; 8]; 2],
            occupied: [0; 64],
            to_move: color::WHITE,
            castling: [0, 0],
            en_passant: None,
            halfmoves: 0,
            fullmoves: 1,
        }
    }

    pub fn equals(&self, rhs: &Position) -> bool {
        for side in 0..2 {
            for bb in 0..8 {
                if self.bb[side][bb] != rhs.bb[side][bb] {
                    return false;
                }
            }
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
        if 0 < self.bb_own(color::WHITE) & self.bb_opponent(color::WHITE) {
            self.panic_helper()
        }

        for i in 2..8 {
            if 0 < !(self.bb_own(color::WHITE) | self.bb_opponent(color::WHITE)) & self.bb[0][i] {
                self.panic_helper()
            }
            for j in i + 1..8 {
                if 0 < self.bb[0][i] & self.bb[0][j] {
                    self.panic_helper()
                }
            }
        }
        true
    }

    pub fn bb(&self) -> &[[Bitboard; 8]; 2] {
        &self.bb
    }

    // don't actually return flipped boards for now
    pub fn bb_own(&self, color: Color) -> Bitboard {
        self.bb[0][color as usize]
    }

    pub fn bb_opponent(&self, color: Color) -> Bitboard {
        self.bb[0][(1 ^ color) as usize]
    }

    pub fn bb_pawns(&self, color: Color) -> Bitboard {
        self.bb[0][piece::PAWN as usize] & self.bb_own(color)
    }

    pub fn bb_knights(&self, color: Color) -> Bitboard {
        self.bb[0][piece::KNIGHT as usize] & self.bb_own(color)
    }

    pub fn bb_bishops(&self, color: Color) -> Bitboard {
        self.bb[0][piece::BISHOP as usize] & self.bb_own(color)
    }

    pub fn bb_rooks(&self, color: Color) -> Bitboard {
        self.bb[0][piece::ROOK as usize] & self.bb_own(color)
    }

    pub fn bb_queens(&self, color: Color) -> Bitboard {
        self.bb[0][piece::QUEEN as usize] & self.bb_own(color)
    }

    pub fn bb_king(&self, color: Color) -> Bitboard {
        self.bb[0][piece::KING as usize] & self.bb_own(color)
    }

    pub fn bb_empty(&self) -> Bitboard {
        !(self.bb_own(color::WHITE) | self.bb_opponent(color::WHITE))
    }

    pub fn to_move(&self) -> Color {
        self.to_move
    }

    pub fn castling(&self) -> [u32; 2] {
        self.castling
    }

    pub fn en_passant(&self) -> Option<[Square; 2]> {
        self.en_passant
    }

    pub fn occupied(&self) -> &[Piece; 64] {
        &self.occupied
    }

    pub fn halfmoves(&self) -> u32 {
        self.halfmoves
    }

    pub fn fullmoves(&self) -> u32 {
        self.fullmoves
    }

    pub fn get_piece_and_color(&self, square: Square) -> (Piece, Color) {
        (
            self.occupied[square as usize].code(),
            self.occupied[square as usize].color(),
        )
    }

    pub fn check_piece(&self, piece: Piece, color: Color, square: Square) -> bool {
        (piece, color) == self.get_piece_and_color(square)
    }

    pub fn set_piece(&mut self, piece: Piece, color: Color, to: Square) {
        // update unflipped bb
        self.bb[0][color as usize].set(to);
        self.bb[0][piece as usize].set(to);

        // update flipped bb
        self.bb[1][color as usize].set(to ^ 56);
        self.bb[1][piece as usize].set(to ^ 56);

        // update occupancy array
        self.occupied[to as usize] = Piece::new(piece, color);
    }

    pub fn remove_piece(&mut self, piece: Piece, color: Color, from: Square) {
        // update unflipped bb
        self.bb[0][color as usize].clear(from);
        self.bb[0][piece as usize].clear(from);

        // update flipped bb
        self.bb[1][color as usize].clear(from ^ 56);
        self.bb[1][piece as usize].clear(from ^ 56);

        // update occupancy array
        self.occupied[from as usize] = 0;
    }

    pub fn replace_piece(
        &mut self,
        old_piece: Piece,
        old_color: Color,
        new_piece: Piece,
        new_color: Color,
        square: Square,
    ) {
        // remove from unflipped bb
        self.bb[0][old_color as usize].clear(square);
        self.bb[0][old_piece as usize].clear(square);

        // remove from flipped bb
        self.bb[1][old_color as usize].clear(square ^ 56);
        self.bb[1][old_piece as usize].clear(square ^ 56);

        self.set_piece(new_piece, new_color, square);
    }

    pub fn make_move(&mut self, mov: Move) {
        // fail if no piece at origin square
        // debug_assert!(self.check_piece(mov.piece(), mov.color(), mov.from()));

        let orig_square = mov.orig();
        let dest_square = mov.dest();
        let (orig_piece, orig_color) = self.get_piece_and_color(orig_square);
        let (mut dest_piece, mut dest_color) = self.get_piece_and_color(dest_square);
        // let is_capture = 0 != dest_piece;
        let is_capture = mov.is_capture();

        // let _ep_allowed = self.en_passant != None;
        // let _ep_square = match self.en_passant {
        //     Some(sq) => sq[0],
        //     None => 64,
        // };

        // debug_assert_eq!(orig_color, self.to_move);
        if orig_color != self.to_move {
            eprintln!("orig_color != self.to_move");
            eprintln!("offending move: {:?}", mov);
            self.panic_helper();
        }

        // reset en passant
        self.en_passant = None;

        // promotions change pieces
        if mov.is_promotion() {
            let prom_piece = mov.special() as Piece + 3;
            self.remove_piece(orig_piece, orig_color, orig_square);
            if is_capture {
                self.replace_piece(dest_piece, dest_color, prom_piece, orig_color, dest_square);
            } else {
                self.set_piece(prom_piece, orig_color, dest_square);
            }
        } else if mov.is_quiet() {
            self.remove_piece(orig_piece, orig_color, orig_square);
            self.set_piece(orig_piece, orig_color, dest_square);
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
            self.remove_piece(orig_piece, orig_color, orig_square);
            self.replace_piece(dest_piece, dest_color, orig_piece, orig_color, dest_square);
        } else if mov.is_double_pawn_push() {
            self.remove_piece(orig_piece, orig_color, orig_square);
            let new_ep_square =
                (i32::from(dest_square) - [8i32, -8i32][orig_color as usize]) as Square;
            self.en_passant = Some([new_ep_square, new_ep_square.flipped()]);
            self.set_piece(orig_piece, orig_color, dest_square);
        } else if mov.is_king_castle() {
            self.remove_piece(orig_piece, orig_color, orig_square);
            self.set_piece(orig_piece, orig_color, dest_square);
            // move rook
            self.remove_piece(piece::ROOK, orig_color, dest_square + 1);
            self.set_piece(piece::ROOK, orig_color, dest_square - 1);
        } else if mov.is_queen_castle() {
            self.remove_piece(orig_piece, orig_color, orig_square);
            self.set_piece(orig_piece, orig_color, dest_square);
            // move rook
            self.remove_piece(piece::ROOK, orig_color, dest_square - 2);
            self.set_piece(piece::ROOK, orig_color, dest_square + 1);
        } else {
            panic!("shouldn't come here")
        }

        // clear castling rights on king move
        if piece::KING == orig_piece {
            self.castling[self.to_move as usize].clear_bit(0);
            self.castling[self.to_move as usize].clear_bit(1);
        }

        // clear castling rights on rook move
        if piece::ROOK == orig_piece {
            if BB_SQUARES[orig_square as usize] & BB_CORNERS & BB_FILE_A > 0 {
                self.castling[self.to_move as usize].clear_bit(1);
            } else if BB_SQUARES[orig_square as usize] & BB_CORNERS & BB_FILE_H > 0 {
                self.castling[self.to_move as usize].clear_bit(0);
            }
        }

        // clear castling rights on rook capture at home square
        if dest_piece == piece::ROOK {
            if BB_SQUARES[dest_square as usize] & BB_CORNERS & BB_FILE_A > 0 {
                self.castling[(1 ^ self.to_move) as usize].clear_bit(1);
            } else if BB_SQUARES[dest_square as usize] & BB_CORNERS & BB_FILE_H > 0 {
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
        if piece == piece::PAWN && Some([dest, dest.flipped()]) == self.en_passant {
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
                    assert!(position.bb[0][color as usize] & bb::BB_SQUARES[square as usize] != 0);
                    assert!(position.bb[0][piece as usize] & bb::BB_SQUARES[square as usize] != 0);
                    assert!(
                        position.bb[1][color as usize] & bb::BB_SQUARES[(square ^ 56) as usize]
                            != 0
                    );
                    assert!(
                        position.bb[1][piece as usize] & bb::BB_SQUARES[(square ^ 56) as usize]
                            != 0
                    );
                    assert_eq!(piece, position.occupied[square as usize].code());
                    assert_eq!(color, position.occupied[square as usize].color());

                    //assert_eq!(position.bb[1][color as usize], bits::swap_bytes(position.bb[0][color as usize]));
                    //assert_eq!(position.bb[1][piece as usize], bits::swap_bytes(position.bb[0][piece as usize]));

                    assert_eq!(
                        position.bb[1][color as usize],
                        position.bb[0][color as usize].swap_bytes()
                    );
                    assert_eq!(
                        position.bb[1][piece as usize],
                        position.bb[0][piece as usize].swap_bytes()
                    );
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
                    assert!(position.bb[0][color as usize] & bb::BB_SQUARES[square as usize] != 0);
                    assert!(position.bb[0][piece as usize] & bb::BB_SQUARES[square as usize] != 0);
                    assert!(
                        position.bb[1][color as usize] & bb::BB_SQUARES[(square ^ 56) as usize]
                            != 0
                    );
                    assert!(
                        position.bb[1][piece as usize] & bb::BB_SQUARES[(square ^ 56) as usize]
                            != 0
                    );
                    assert_eq!(piece, position.occupied[square as usize].code());
                    assert_eq!(color, position.occupied[square as usize].color());

                    assert_eq!(
                        position.bb[1][color as usize],
                        position.bb[0][color as usize].swap_bytes()
                    );
                    assert_eq!(
                        position.bb[1][piece as usize],
                        position.bb[0][piece as usize].swap_bytes()
                    );
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
