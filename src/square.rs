use std::collections::HashMap;
use std::str::FromStr;
use std::string::String;

pub type Square = u32;

pub trait SquarePrimitives<T> {
    fn from_coords(x: u32, y: u32) -> Square;
    fn from_san_string(square: &str) -> Result<Square, &'static str>;
    fn to_san_string(self) -> String;

    fn flipped(self) -> Square;
}

impl SquarePrimitives<Square> for Square {
    #[inline]
    fn from_coords(x: u32, y: u32) -> Square {
        ((y << 3) + x) as Square
    }

    fn from_san_string(square: &str) -> Result<Square, &'static str> {
        if square.len() != 2 {
            return Err("Invalid square string");
        }

        let file_lookup: HashMap<&'static str, Square> = [
            ("a", 0),
            ("b", 1),
            ("c", 2),
            ("d", 3),
            ("e", 4),
            ("f", 5),
            ("g", 6),
            ("h", 7),
        ]
        .iter()
        .cloned()
        .collect();

        let x = match file_lookup.get(&square[0..1]) {
            Some(val) => val,
            None => return Err("Invalid square string"),
        };
        let y = match u32::from_str(&square[1..2]) {
            Ok(val) => val - 1,
            Err(_) => return Err("Invalid square string"),
        };

        Ok((y << 3) + *x)
    }

    fn to_san_string(self) -> String {
        let mut san = String::new();
        san.push_str(FILE_NAMES[(self & 0x7) as usize]);
        san.push_str(&(((self >> 3) + 1).to_string()));
        san
    }

    #[inline]
    fn flipped(self) -> Square {
        self ^ 56
    }
}

macro_rules! msq {
    ($($id:ident,$val:expr),*) => {
        $(pub const $id: Square = $val;)*
    };
}

#[rustfmt::skip::macros(msq)]
msq!(A1, 0, B1, 1, C1, 2, D1, 3, E1, 4, F1, 5, G1, 6, H1, 7,
     A2, 8, B2, 9, C2,10, D2,11, E2,12, F2,13, G2,14, H2,15,
     A3,16, B3,17, C3,18, D3,19, E3,20, F3,21, G3,22, H3,23,
     A4,24, B4,25, C4,26, D4,27, E4,28, F4,29, G4,30, H4,31,
     A5,32, B5,33, C5,34, D5,35, E5,36, F5,37, G5,38, H5,39,
     A6,40, B6,41, C6,42, D6,43, E6,44, F6,45, G6,46, H6,47,
     A7,48, B7,49, C7,50, D7,51, E7,52, F7,53, G7,54, H7,55,
     A8,56, B8,57, C8,58, D8,59, E8,60, F8,61, G8,62, H8,63);

#[rustfmt::skip]
pub const SQUARE_NAMES: [&str; 64] = [
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1",
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
    "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
];

pub const FILE_NAMES: [&str; 8] = ["a", "b", "c", "d", "e", "f", "g", "h"];

pub const RANK_NAMES: [&str; 8] = ["1", "2", "3", "4", "5", "6", "7", "8"];

#[rustfmt::skip]
pub fn ep_capture_square(ep_square: Square) -> Square {
    let table = [
         0,  0,  0,  0,  0,  0,  0,  0,
         0,  0,  0,  0,  0,  0,  0,  0,
        24, 25, 26, 27, 28, 29, 30, 31,
         0,  0,  0,  0,  0,  0,  0,  0,
         0,  0,  0,  0,  0,  0,  0,  0,
        32, 33, 34, 35, 36, 37, 38, 39,
         0,  0,  0,  0,  0,  0,  0,  0,
         0,  0,  0,  0,  0,  0,  0,  0,
    ];
    table[ep_square as usize]
}

// // pub fn flip_square(square: Square) -> Square {
//     square ^ 56
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_turns_strings_to_squares() {
        let mut strs = Vec::new();
        for y in ['1', '2', '3', '4', '5', '6', '7', '8'].iter() {
            for x in ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'].iter() {
                strs.push(format!("{}{}", x, y));
            }
        }

        for (index, st) in strs.into_iter().enumerate() {
            assert_eq!(index, Square::from_san_string(&st).unwrap() as usize);
        }
    }

    #[test]
    fn it_turns_squares_to_strings() {
        let mut strs = Vec::new();
        for y in ['1', '2', '3', '4', '5', '6', '7', '8'].iter() {
            for x in ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'].iter() {
                strs.push(format!("{}{}", x, y));
            }
        }

        for (index, st) in strs.into_iter().enumerate() {
            assert_eq!(st, (index as Square).to_san_string());
        }
    }
}
