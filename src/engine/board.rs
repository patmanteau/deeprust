pub enum Piece {
    White,
    Black,
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

pub enum Color {
    White,
    Black,
}

struct Board {
    bb: [u64; 8],
}

impl Board {
    // fn get_white_pawns(&self) -> u64 {
    //     self.bb[Piece::Pawn as usize] & self.bb[Piece::White as usize]
    // }

    // fn get_pawns(&self, color: Color) -> u64 {
    //     self.bb[Piece::Pawn as usize] & self.bb[color as usize]
    // }

    fn get_pieces(&self, piece: Piece, color: Color) -> u64 {
        self.bb[piece as usize] & self.bb[color as usize]
    }

    fn set_piece(&mut self, piece: Piece, color: Color, x: u32, y: u32) {
        self.bb[piece as usize] |= 1 << ((y * 8) + x);
        self.bb[color as usize] |= 1 << ((y * 8) + x);
    }

    fn set_startpos(&mut self) {
        // pawns
        for x in 0..8 {
            self.set_piece(Piece::Pawn, Color::White, x, 1);
            self.set_piece(Piece::Pawn, Color::Black, x, 6);
        }

        // knights
        self.set_piece(Piece::Knight, Color::White, 1, 0);
        self.set_piece(Piece::Knight, Color::White, 6, 0);
        self.set_piece(Piece::Knight, Color::Black, 1, 7);
        self.set_piece(Piece::Knight, Color::Black, 6, 7);

        // bishops
        self.set_piece(Piece::Bishop, Color::White, 2, 0);
        self.set_piece(Piece::Bishop, Color::White, 5, 0);
        self.set_piece(Piece::Bishop, Color::Black, 2, 7);
        self.set_piece(Piece::Bishop, Color::Black, 5, 7);

        // rooks
        self.set_piece(Piece::Rook, Color::White, 0, 0);
        self.set_piece(Piece::Rook, Color::White, 7, 0);
        self.set_piece(Piece::Rook, Color::Black, 0, 7);
        self.set_piece(Piece::Rook, Color::Black, 7, 7);

        // queens
        self.set_piece(Piece::Queen, Color::White, 3, 0);
        self.set_piece(Piece::Queen, Color::Black, 3, 7);

        // kings
        self.set_piece(Piece::King, Color::White, 4, 0);
        self.set_piece(Piece::King, Color::Black, 4, 7);   
    }

    fn to_fen(&self) -> String {
        let mut fen_string = String::new();

        for y in (0..8).rev() {
            let mut emptycount = 0;
            for x in 0..8 {
                if 0 == ( (self.bb[Color::White as usize] | self.bb[Color::Black as usize]) & (1 << ((y * 8) + x))) {
                    emptycount += 1;
                } else {
                    if emptycount > 0 { 
                        fen_string.push_str(&emptycount.to_string());
                        emptycount = 0;
                    };

                    if 0 != (self.bb[Piece::Pawn as usize] & self.bb[Color::White as usize] & (1 << ((y * 8) + x))) { fen_string.push('P')}
                    else if 0 != self.bb[Piece::Pawn as usize] & self.bb[Color::Black as usize] & (1 << ((y * 8) + x)) { fen_string.push('p')}
                    else if 0 != self.bb[Piece::Knight as usize] & self.bb[Color::White as usize] & (1 << ((y * 8) + x)) { fen_string.push('N')}
                    else if 0 != self.bb[Piece::Knight as usize] & self.bb[Color::Black as usize] & (1 << ((y * 8) + x)) { fen_string.push('n')}
                    else if 0 != self.bb[Piece::Bishop as usize] & self.bb[Color::White as usize] & (1 << ((y * 8) + x)) { fen_string.push('B')}
                    else if 0 != self.bb[Piece::Bishop as usize] & self.bb[Color::Black as usize] & (1 << ((y * 8) + x)) { fen_string.push('b')}
                    else if 0 != self.bb[Piece::Rook as usize] & self.bb[Color::White as usize] & (1 << ((y * 8) + x)) { fen_string.push('R')}
                    else if 0 != self.bb[Piece::Rook as usize] & self.bb[Color::Black as usize] & (1 << ((y * 8) + x)) { fen_string.push('r')}
                    else if 0 != self.bb[Piece::Queen as usize] & self.bb[Color::White as usize] & (1 << ((y * 8) + x)) { fen_string.push('Q')}
                    else if 0 != self.bb[Piece::Queen as usize] & self.bb[Color::Black as usize] & (1 << ((y * 8) + x)) { fen_string.push('q')}
                    else if 0 != self.bb[Piece::King as usize] & self.bb[Color::White as usize] & (1 << ((y * 8) + x)) { fen_string.push('K')}
                    else if 0 != self.bb[Piece::King as usize] & self.bb[Color::Black as usize] & (1 << ((y * 8) + x)) { fen_string.push('k')};
                }
            }
            if emptycount > 0 {
                fen_string.push_str(&emptycount.to_string());
                emptycount = 0;
            };
            if y > 0 { fen_string.push('/'); }
        }
        fen_string
    }

    fn from_fen(fen_string: String) -> Result<Board, &'static str> {
        let mut board = Board { bb: [0; 8] };
        let mut x = 0;
        let mut y = 7;
        for chr in fen_string.chars() {
            if let Some(empty) = chr.to_digit(10) {
                x += empty
            } else {
                if chr == '/' {
                    x = 0;
                    y -= 1;
                } else {
                    match chr {
                        'P' => board.set_piece(Piece::Pawn, Color::White, x, y),
                        'N' => board.set_piece(Piece::Knight, Color::White, x, y),
                        'B' => board.set_piece(Piece::Bishop, Color::White, x, y),
                        'R' => board.set_piece(Piece::Rook, Color::White, x, y),
                        'Q' => board.set_piece(Piece::Queen, Color::White, x, y),
                        'K' => board.set_piece(Piece::King, Color::White, x, y),
                        'p' => board.set_piece(Piece::Pawn, Color::Black, x, y),
                        'n' => board.set_piece(Piece::Knight, Color::Black, x, y),
                        'b' => board.set_piece(Piece::Bishop, Color::Black, x, y),
                        'r' => board.set_piece(Piece::Rook, Color::Black, x, y),
                        'q' => board.set_piece(Piece::Queen, Color::Black, x, y),
                        'k' => board.set_piece(Piece::King, Color::Black, x, y),
                        _ => { return Err("Invalid FEN string") },
                    }
                    x += 1;
                }
            }
        }
        Ok(board)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_has_correct_piece_enum_values() {
        assert_eq!(0, Piece::White as usize);
        assert_eq!(1, Piece::Black as usize);
        assert_eq!(2, Piece::Pawn as usize);
        assert_eq!(3, Piece::Knight as usize);
        assert_eq!(4, Piece::Bishop as usize);
        assert_eq!(5, Piece::Rook as usize);
        assert_eq!(6, Piece::Queen as usize);
        assert_eq!(7, Piece::King as usize);
    }

    #[test]
    fn it_has_correct_color_enum_values() {
        assert_eq!(0, Piece::White as usize);
        assert_eq!(1, Piece::Black as usize);
    }
    
    #[test]
    fn it_sets_correct_startpos() {
        let mut b = Board { bb: [0; 8] };
        b.set_startpos();

        // color boards
        assert_eq!(0xffff, b.bb[Color::White as usize]);
        assert_eq!(0xffff << 6*8, b.bb[Color::Black as usize]);

        // pawn boards
        assert_eq!(0xff << 8, b.bb[Piece::Pawn as usize] & b.bb[Color::White as usize]);
        assert_eq!(0xff << 8, b.get_pieces(Piece::Pawn, Color::White));
        assert_eq!(0xff << 6*8, b.bb[Piece::Pawn as usize] & b.bb[Color::Black as usize]);
        assert_eq!(0xff << 6*8, b.get_pieces(Piece::Pawn, Color::Black));

        // rook boards
        assert_eq!(0x81, b.bb[Piece::Rook as usize] & b.bb[Color::White as usize]);
        assert_eq!(0x81, b.get_pieces(Piece::Rook, Color::White));
        assert_eq!(0x81 << 7*8, b.bb[Piece::Rook as usize] & b.bb[Color::Black as usize]);
        assert_eq!(0x81 << 7*8, b.get_pieces(Piece::Rook, Color::Black));
        

        // bishop boards
        assert_eq!(0x24, b.bb[Piece::Bishop as usize] & b.bb[Color::White as usize]);
        assert_eq!(0x24, b.get_pieces(Piece::Bishop, Color::White));
        assert_eq!(0x24 << 7*8, b.bb[Piece::Bishop as usize] & b.bb[Color::Black as usize]);
        assert_eq!(0x24 << 7*8, b.get_pieces(Piece::Bishop, Color::Black));

        // queen boards
        assert_eq!(0x8, b.bb[Piece::Queen as usize] & b.bb[Color::White as usize]);
        assert_eq!(0x8, b.get_pieces(Piece::Queen, Color::White));
        assert_eq!(0x8 << 7*8, b.bb[Piece::Queen as usize] & b.bb[Color::Black as usize]);
        assert_eq!(0x8 << 7*8, b.get_pieces(Piece::Queen, Color::Black));

        // king boards
        assert_eq!(0x10, b.bb[Piece::King as usize] & b.bb[Color::White as usize]);
        assert_eq!(0x10, b.get_pieces(Piece::King, Color::White));
        assert_eq!(0x10 << 7*8, b.bb[Piece::King as usize] & b.bb[Color::Black as usize]);
        assert_eq!(0x10 << 7*8, b.get_pieces(Piece::King, Color::Black));
    }

    #[test]
    fn it_makes_correct_fen_strings() {
        let mut b = Board { bb: [0; 8] };
        b.set_startpos();

        assert_eq!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", b.to_fen());
    }

    #[test]
    fn it_parses_fen_strings_correctly() {
        let b = Board::from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR"));
        match b {
            Err(e) => assert!(false, e),
            Ok(board) => {
                assert_eq!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", board.to_fen());
            }
        }
    }
}