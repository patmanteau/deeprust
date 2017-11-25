use super::super::types::*;
use super::super::types;
use super::super::san::SAN;
use super::{Board, c2s};

use std::str::FromStr;

fn from_fen(fen_string: String) -> Result<Board, &'static str> {
    let mut fen_iter = fen_string.split_whitespace();

    let mut board = Board::new(); // Board { bb: [0; 8] };

    // position
    let mut x = 0;
    let mut y = 7;
    if let Some(position) = fen_iter.next() {
        for chr in position.chars() {
            if let Some(empty) = chr.to_digit(10) {
                x += empty
            } else {
                if chr == '/' {
                    x = 0;
                    y -= 1;
                } else {
                    match chr {
                        'P' => board.set_piece(types::PAWN, types::WHITE, c2s(x, y)),
                        'N' => board.set_piece(types::KNIGHT, types::WHITE, c2s(x, y)),
                        'B' => board.set_piece(types::BISHOP, types::WHITE, c2s(x, y)),
                        'R' => board.set_piece(types::ROOK, types::WHITE, c2s(x, y)),
                        'Q' => board.set_piece(types::QUEEN, types::WHITE, c2s(x, y)),
                        'K' => board.set_piece(types::KING, types::WHITE, c2s(x, y)),
                        'p' => board.set_piece(types::PAWN, types::BLACK, c2s(x, y)),
                        'n' => board.set_piece(types::KNIGHT, types::BLACK, c2s(x, y)),
                        'b' => board.set_piece(types::BISHOP, types::BLACK, c2s(x, y)),
                        'r' => board.set_piece(types::ROOK, types::BLACK, c2s(x, y)),
                        'q' => board.set_piece(types::QUEEN, types::BLACK, c2s(x, y)),
                        'k' => board.set_piece(types::KING, types::BLACK, c2s(x, y)),
                        _ => { return Err("Invalid FEN string") },
                    }
                    x += 1;
                }
            }
        }    
    } else {
        return Err("Invalid FEN string, no position found");
    }

    // to move
    if let Some(to_move) = fen_iter.next() {
        match to_move {
            "w" => board.to_move = types::WHITE,
            "b" => board.to_move = types::BLACK,
            val => return Err("Invalid ToMove char")
        }
    } else {
        return Err("Invalid FEN string, don't know who moves next")
    }

    // Castling rights
    if let Some(castling) = fen_iter.next() {
        for chr in castling.chars() {
            match chr {
                '-' => board.castling = 0,
                'K' => board.castling |= 0x1,
                'Q' => board.castling |= 0x2,
                'k' => board.castling |= 0x4,
                'q' => board.castling |= 0x8,
                val => return Err("Invalid castling char")
            }
        }
    } else {
        return Err("Invalid FEN string, no castling rights found")
    }

    // en passant
    if let Some(en_passant) = fen_iter.next() {
        if en_passant == "-" {
            board.en_passant = None;
        } else {
            match SAN::square_str_to_index(en_passant) {
                Ok(eps) => board.en_passant = Some(eps),
                Err(e) => return Err("Error parsing en passant field"),
            }
        }
    } else {
        return Err("Invalid FEN string, no en passant information")
    }

    // Halfmoves
    if let Some(halfmoves) = fen_iter.next() {
        match u32::from_str(halfmoves) {
            Ok(val) => board.halfmoves = val,
            Err(e) => return Err("Error parsing halfmoves"),
        }
    } else {
        return Err("Invalid FEN string, no halfmoves given")
    }
    
    // Fullmoves
    if let Some(fullmoves) = fen_iter.next() {
        match u32::from_str(fullmoves) {
            Ok(val) => board.fullmoves = val,
            Err(e) => return Err("Error parsing fullmoves"),
        }
    } else {
        return Err("Invalid FEN string, no fullmoves given")
    }

    
    Ok(board)
}

fn to_fen(board: &Board) -> String {
    let mut fen_string = String::new();

    // Position
    for y in (0..8).rev() {
        let mut emptycount = 0;
        for x in 0..8 {
            if 0 == ( (board.bb[types::WHITE as usize] | board.bb[types::BLACK as usize]) & (1 << ((y << 3) + x))) {
                emptycount += 1;
            } else {
                if emptycount > 0 { 
                    fen_string.push_str(&emptycount.to_string());
                    emptycount = 0;
                };

                if 0 != (board.bb[types::PAWN as usize] & board.bb[types::WHITE as usize] & (1 << c2s(x, y))) { fen_string.push('P')}
                else if 0 != board.bb[types::PAWN as usize] & board.bb[types::BLACK as usize] & (1 << c2s(x, y)) { fen_string.push('p')}
                else if 0 != board.bb[types::KNIGHT as usize] & board.bb[types::WHITE as usize] & (1 << c2s(x, y)) { fen_string.push('N')}
                else if 0 != board.bb[types::KNIGHT as usize] & board.bb[types::BLACK as usize] & (1 << c2s(x, y)) { fen_string.push('n')}
                else if 0 != board.bb[types::BISHOP as usize] & board.bb[types::WHITE as usize] & (1 << c2s(x, y)) { fen_string.push('B')}
                else if 0 != board.bb[types::BISHOP as usize] & board.bb[types::BLACK as usize] & (1 << c2s(x, y)) { fen_string.push('b')}
                else if 0 != board.bb[types::ROOK as usize] & board.bb[types::WHITE as usize] & (1 << c2s(x, y)) { fen_string.push('R')}
                else if 0 != board.bb[types::ROOK as usize] & board.bb[types::BLACK as usize] & (1 << c2s(x, y)) { fen_string.push('r')}
                else if 0 != board.bb[types::QUEEN as usize] & board.bb[types::WHITE as usize] & (1 << c2s(x, y)) { fen_string.push('Q')}
                else if 0 != board.bb[types::QUEEN as usize] & board.bb[types::BLACK as usize] & (1 << c2s(x, y)) { fen_string.push('q')}
                else if 0 != board.bb[types::KING as usize] & board.bb[types::WHITE as usize] & (1 << c2s(x, y)) { fen_string.push('K')}
                else if 0 != board.bb[types::KING as usize] & board.bb[types::BLACK as usize] & (1 << c2s(x, y)) { fen_string.push('k')};
            }
        }
        if emptycount > 0 {
            fen_string.push_str(&emptycount.to_string());
            emptycount = 0;
        };
        if y > 0 { fen_string.push('/'); }
    }

    // To move
    fen_string.push(' ');
    let to_move = match board.to_move {
        types::WHITE => 'w',
        types::BLACK => 'b',
        _ => 'w',
    };
    fen_string.push(to_move);

    // Castling rights
    fen_string.push(' ');
    if board.castling == 0 {
        fen_string.push('-');
    } else {
        if 0 != board.castling & 0x1 { fen_string.push('K'); }
        if 0 != board.castling & 0x2 { fen_string.push('Q'); }
        if 0 != board.castling & 0x4 { fen_string.push('k'); }
        if 0 != board.castling & 0x8 { fen_string.push('q'); }
    }

    // en passant
    fen_string.push(' ');
    if let Some(eps) = board.en_passant {
        let san = SAN::from_square(eps);
        fen_string.push_str(&san.s.to_string())
    } else {
        fen_string.push('-')
    }

    // Halfmoves
    fen_string.push(' ');
    fen_string.push_str(&board.halfmoves.to_string());
    
    // Fullmoves
    fen_string.push(' ');
    fen_string.push_str(&board.fullmoves.to_string());
    
    fen_string
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_makes_correct_fen_strings() {
        let b = Board::startpos();
        assert_eq!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", to_fen(&b));
    }

    #[test]
    fn it_parses_fen_strings_correctly() {
        let b = from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"));
        match b {
            Err(e) => assert!(false, e),
            Ok(board) => assert_eq!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", to_fen(&board)),
        }

        let b = from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - e3 0 1"));
        match b {
            Err(e) => assert!(false, e),
            Ok(board) => assert_eq!("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - e3 0 1", to_fen(&board)),
        }
    }

    #[test]
    fn it_rejects_invalid_fen_strings() {
        let b = from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR"));
        match b {
            Err(e) => assert!(true),
            Ok(board) => assert!(false),
        }

        let b = from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq abcdefg 0 1"));
        match b {
            Err(e) => assert!(true),
            Ok(board) => assert!(false),
        }

        let b = from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR y KQkq e3 0 1"));
        match b {
            Err(e) => assert!(true),
            Ok(board) => assert!(false),
        }

        let b = from_fen(String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w HFhf e3 0 1"));
        match b {
            Err(e) => assert!(true),
            Ok(board) => assert!(false),
        }
    }
}
