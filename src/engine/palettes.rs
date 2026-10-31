use super::types::PaletteType;

pub const STANDARD_70: &str = "$@B%8&WM#*oahkbdpqwmZO0QLCJUYXzcvunxrjft/\\|()1{}[]?-_+~<>i!lI;:,\"^`'. ";
pub const STANDARD_10: &str = "@%#*+=-:. ";
pub const BLOCKS: &str = "█▓▒░ ";
pub const MINIMALIST: &str = "@Oo. ";
pub const BINARY: &str = "10 ";
pub const MATRIX: &str = "$#@&0123456789ABCDEF!?:;=+-_~ ";

pub struct CharPalette {
    /// Characters ordered from darkest/densest to lightest (or vice versa)
    pub chars: Vec<char>,
}

impl CharPalette {
    pub fn new(chars: &[char]) -> Self {
        if chars.is_empty() {
            Self { chars: vec![' '] }
        } else {
            Self { chars: chars.to_vec() }
        }
    }

    pub fn from_str(s: &str) -> Self {
        let chars: Vec<char> = s.chars().collect();
        Self::new(&chars)
    }

    pub fn from_type(palette_type: &PaletteType) -> Self {
        match palette_type {
            PaletteType::Standard70 => Self::from_str(STANDARD_70),
            PaletteType::Standard10 => Self::from_str(STANDARD_10),
            PaletteType::Blocks => Self::from_str(BLOCKS),
            PaletteType::Minimalist => Self::from_str(MINIMALIST),
            PaletteType::Binary => Self::from_str(BINARY),
            PaletteType::Matrix => Self::from_str(MATRIX),
            PaletteType::SobelOutline | PaletteType::Braille => Self::from_str(" "),
            PaletteType::Custom(s) => {
                if s.trim().is_empty() {
                    Self::from_str(STANDARD_10)
                } else {
                    Self::from_str(s)
                }
            }
        }
    }

    /// Maps normalized luminance [0.0, 1.0] to a character.
    /// Notice: In standard ASCII art on a dark background (terminal/modern apps):
    /// 0.0 (black background) -> ' ' (empty space, index len - 1)
    /// 1.0 (brightest highlight) -> '@' or '$' (index 0).
    /// If user inverts, adjust_luminance handles the swap.
    #[inline]
    pub fn get_char(&self, luma: f32) -> char {
        let len = self.chars.len();
        if len == 0 {
            return ' ';
        }
        if len == 1 {
            return self.chars[0];
        }

        // luma 0.0 -> darkest (space, end of palette)
        // luma 1.0 -> brightest (dense, start of palette)
        let inverted = 1.0 - luma.clamp(0.0, 1.0);
        let index = ((inverted * (len - 1) as f32).round() as usize).min(len - 1);
        self.chars[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_mapping() {
        let p = CharPalette::from_str("@.");
        assert_eq!(p.get_char(1.0), '@');
        assert_eq!(p.get_char(0.0), '.');
    }

    #[test]
    fn test_standard_10() {
        let p = CharPalette::from_type(&PaletteType::Standard10);
        assert_eq!(p.get_char(1.0), '@');
        assert_eq!(p.get_char(0.0), ' ');
    }
}
