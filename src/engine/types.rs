use serde::{Deserialize, Serialize};

/// Color representation in RGB
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255 };
    pub const GREEN_PHOSPHOR: Self = Self { r: 51, g: 255, b: 51 };
    pub const AMBER_CRT: Self = Self { r: 255, g: 176, b: 0 };
    pub const CYAN: Self = Self { r: 0, g: 255, b: 255 };
    pub const MAGENTA: Self = Self { r: 255, g: 0, b: 128 };

    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

/// Dithering algorithm
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DitherMode {
    #[default]
    None,
    FloydSteinberg,
    Atkinson,
    Bayer4x4,
}

impl DitherMode {
    pub fn all() -> &'static [Self] {
        &[
            Self::None,
            Self::FloydSteinberg,
            Self::Atkinson,
            Self::Bayer4x4,
        ]
    }

    pub fn label_ru(&self) -> &'static str {
        match self {
            Self::None => "Без дизеринга (Прямой порог)",
            Self::FloydSteinberg => "Floyd-Steinberg (Плавный)",
            Self::Atkinson => "Atkinson (Ретро Macintosh)",
            Self::Bayer4x4 => "Bayer 4x4 (Сетка)",
        }
    }
}

/// Character palette type
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PaletteType {
    #[default]
    Standard70,
    Standard10,
    Blocks,
    Minimalist,
    Binary,
    Matrix,
    SobelOutline,
    Braille,
    Custom(String),
}

impl PaletteType {
    pub fn label_ru(&self) -> &'static str {
        match self {
            Self::Standard70 => "Детальная (70 символов)",
            Self::Standard10 => "Простая (10 символов)",
            Self::Blocks => "Блочные символы ( ░▒▓█)",
            Self::Minimalist => "Минимализм ( .oO@)",
            Self::Binary => "Двоичный код (01)",
            Self::Matrix => "Матрица (Matrix Code)",
            Self::SobelOutline => "Контурный скетч (Sobel)",
            Self::Braille => "Ультра-детализация Брайля (2x4)",
            Self::Custom(_) => "Пользовательский набор",
        }
    }
}

/// Color mode for characters
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ColorMode {
    #[default]
    Monochrome,
    GreenPhosphor,
    AmberCrt,
    Cyberpunk,
    TrueColor,
}

impl ColorMode {
    pub fn all() -> &'static [Self] {
        &[
            Self::Monochrome,
            Self::GreenPhosphor,
            Self::AmberCrt,
            Self::Cyberpunk,
            Self::TrueColor,
        ]
    }

    pub fn label_ru(&self) -> &'static str {
        match self {
            Self::Monochrome => "Монохром (Чёрно-белый)",
            Self::GreenPhosphor => "Зелёный терминал (CRT)",
            Self::AmberCrt => "Янтарный экран (Amber CRT)",
            Self::Cyberpunk => "Киберпанк (Неон)",
            Self::TrueColor => "Полноцветный (TrueColor RGB)",
        }
    }
}

/// Parameters for conversion
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConversionConfig {
    /// Width of the output in characters
    pub width: usize,
    /// Character aspect ratio (character width / character height)
    /// Monospace fonts are typically ~0.50
    pub aspect_ratio: f32,
    /// Brightness adjustment (-1.0 to 1.0)
    pub brightness: f32,
    /// Contrast adjustment (-1.0 to 1.0)
    pub contrast: f32,
    /// Gamma exponent (0.2 to 3.0, 1.0 is linear)
    pub gamma: f32,
    /// Invert brightness
    pub invert: bool,
    /// Dithering algorithm
    pub dither: DitherMode,
    /// Character set
    pub palette: PaletteType,
    /// Edge detection threshold (0.0 to 1.0) for Sobel
    pub edge_threshold: f32,
    /// Color mode
    pub color_mode: ColorMode,
}

impl Default for ConversionConfig {
    fn default() -> Self {
        Self {
            width: 100,
            aspect_ratio: 0.50,
            brightness: 0.0,
            contrast: 0.0,
            gamma: 1.0,
            invert: false,
            dither: DitherMode::None,
            palette: PaletteType::Standard70,
            edge_threshold: 0.25,
            color_mode: ColorMode::Monochrome,
        }
    }
}
