use super::types::{Piece};

use std::collections::HashMap;
use std::str::FromStr;

pub struct SAN {
    pub s: String,
    pub to: u32,
    pub from: Option<u32>,
    pub is_capture: Option<bool>,
    pub dep_piece: Option<Piece>,
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

        let y = match file_lookup.get(&square[0..1]) {
            Some(val) => val,
            None => return Err("Invalid square string"),
        };
        let x = match u32::from_str(&square[1..2]) {
            Ok(val) => val - 1,
            Err(e) => return Err("Invalid square string"),
        };

        Ok(y * 8 + x)
    }

    pub fn from_str(san: &str) -> Result<SAN, &'static str> {
        let file_lookup: HashMap<&str, u8> = [
            ("a", 0),
            ("b", 1),
            ("c", 2),
            ("d", 3),
            ("e", 4),
            ("f", 5),
            ("g", 6),
            ("h", 7),
        ].iter().cloned().collect();

        let s = String::from(san);
        Ok(SAN::new())
    }
    
    pub fn from_square(square: u32) -> SAN {
        let mut san = SAN::new();
        san.to = square;
        san.s.push(FILE[(square >> 3) as usize]);
        san.s.push_str(&(((square & 0x7) + 1).to_string()));
        san
    }
}

