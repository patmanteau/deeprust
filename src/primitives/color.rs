pub type Color = u8;

pub mod colors {
    use super::*;
    pub const WHITE: Color = 0;
    pub const BLACK: Color = 1;
}

pub trait ColorPrimitives {
    fn from_char(c: char) -> Self;
    fn to_char(self) -> char;
}

impl ColorPrimitives for Color {
    fn from_char(c: char) -> Self {
        match c {
            'b' => colors::BLACK,
            'w' => colors::WHITE,
            _ => unreachable!("Internal error: unknown color code {}", c),
        }
    }

    fn to_char(self) -> char {
        match self {
            colors::BLACK => 'b',
            colors::WHITE => 'w',
            _ => unreachable!("Internal error: invalid color code {}", self),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_has_correct_color_enum_values() {
        assert_eq!(0, colors::WHITE as usize);
        assert_eq!(1, colors::BLACK as usize);
    }
}
