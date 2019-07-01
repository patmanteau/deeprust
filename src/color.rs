pub type Color = u8;

pub const WHITE: Color = 0;
pub const BLACK: Color = 1;

pub trait ColorPrimitives {
    fn from_char(c: char) -> Self;
    fn to_char(self) -> char;
}

impl ColorPrimitives for Color {
    fn from_char(c: char) -> Self {
        match c {
            'b' => BLACK,
            'w' => WHITE,
            _ => unreachable!("Internal error: unknown color code {}", c),
        }
    }

    fn to_char(self) -> char {
        match self {
            BLACK => 'b',
            WHITE => 'w',
            _ => unreachable!("Internal error: invalid color code {}", self),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_has_correct_color_enum_values() {
        assert_eq!(0, WHITE as usize);
        assert_eq!(1, BLACK as usize);
    }
}
