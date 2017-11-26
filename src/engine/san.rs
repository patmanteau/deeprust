// use super::types::*;

use std::collections::HashMap;
use std::str::FromStr;

pub struct SAN {
    pub s: String,
    pub to: u32,
    pub from: Option<u32>,
    pub is_capture: Option<bool>,
    pub dep_piece: Option<u32>,
    pub is_check: Option<bool>,
    pub is_mate: Option<bool>,
}

const FILE: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

impl SAN {
    pub fn new() -> SAN {
        SAN {
            s: String::from(""),
            to: 0,
            from: None,
            is_capture: None,
            dep_piece: None,
            is_check: None,
            is_mate: None,
        }
    }

    pub fn square_str_to_index(square: &str) -> Result<u32, &'static str> {
        if square.len() != 2 {
            return Err("Invalid square string")
        }

        let file_lookup: HashMap<&str, u32> = [
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

        Ok((y << 3) + x)
    }

    // pub fn from_str(san: &str) -> Result<SAN, &'static str> {
    //     let file_lookup: HashMap<&str, u32> = [
    //         ("a", 0),
    //         ("b", 1),
    //         ("c", 2),
    //         ("d", 3),
    //         ("e", 4),
    //         ("f", 5),
    //         ("g", 6),
    //         ("h", 7),
    //     ].iter().cloned().collect();

    //     let s = String::from(san);
    //     Ok(SAN::new())
    // }
    
    pub fn from_square(square: u32) -> SAN {
        let mut san = SAN::new();
        san.to = square;
        san.s.push(FILE[(square & 0x7) as usize]);
        san.s.push_str(&(((square >> 3) + 1).to_string()));
        san
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_converts_square_strings_to_indices() {
        let mut strs = Vec::new();
        for y in ['1', '2', '3', '4', '5', '6', '7', '8'].iter() {
            for x in ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'].iter() {
                strs.push(format!("{}{}", x, y));
            }
        }

        for (index, st) in strs.into_iter().enumerate() {
            assert_eq!(index, SAN::square_str_to_index(&st).unwrap() as usize);
        }
    }

    #[test]
    fn it_converts_indices_to_square_strings() {
        let mut strs = Vec::new();
        for y in ['1', '2', '3', '4', '5', '6', '7', '8'].iter() {
            for x in ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'].iter() {
                strs.push(format!("{}{}", x, y));
            }
        }

        for (index, st) in strs.into_iter().enumerate() {
            assert_eq!(st, SAN::from_square(index as u32).s);
        }
    }
}

