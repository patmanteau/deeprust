pub type Color = u8;

pub const WHITE: Color = 0;
pub const BLACK: Color = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_has_correct_color_enum_values() {
        assert_eq!(0, WHITE as usize);
        assert_eq!(1, BLACK as usize);
    }
}
