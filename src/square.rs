use std::collections::HashMap;
use std::str::FromStr;
use std::string::String;

const FILE: [&str; 8] = ["a", "b", "c", "d", "e", "f", "g", "h"];

pub type Square = u32;

pub trait SquarePrimitives {
    fn from_coords(x: u32, y: u32) -> Square;
    fn from_san_string(square: &str) -> Result<Square, &'static str>;
    fn to_san_string(self) -> String;

    fn flipped(self) -> Square;
}

impl SquarePrimitives for Square {
    fn from_coords(x: u32, y: u32) -> Square {
        ((y << 3) + x) as Square
    }

    fn from_san_string(square: &str) -> Result<Square, &'static str> {
        if square.len() != 2 {
            return Err("Invalid square string")
        }

        let file_lookup: HashMap<&'static str, u32> = [
            ("a", 0),
            ("b", 1),
            ("c", 2),
            ("d", 3),
            ("e", 4),
            ("f", 5),
            ("g", 6),
            ("h", 7),
        ].iter().cloned().collect();

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
        san.push_str(FILE[(self & 0x7) as usize]);
        san.push_str(&(((self >> 3) + 1).to_string()));
        san
    }

    #[inline]
    fn flipped(self) -> Square {
        self ^ 56
    }
}