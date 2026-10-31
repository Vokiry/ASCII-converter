use super::types::Rgb;

/// Calculates perceived luminance according to Rec. 709 / sRGB:
/// Y = 0.2126 * R + 0.7152 * G + 0.0722 * B
#[inline]
pub fn rgb_to_luma(rgb: Rgb) -> f32 {
    (0.2126 * rgb.r as f32 + 0.7152 * rgb.g as f32 + 0.0722 * rgb.b as f32) / 255.0
}

/// Applies brightness, contrast, gamma, and invert adjustments to a normalized luminance [0.0, 1.0].
#[inline]
pub fn adjust_luminance(
    mut luma: f32,
    brightness: f32,
    contrast: f32,
    gamma: f32,
    invert: bool,
) -> f32 {
    // Brightness offset
    luma += brightness;

    // Contrast adjustment centered at 0.5:
    // contrast factor: when contrast = 0 -> 1.0, contrast > 0 -> >1.0, contrast < 0 -> <1.0
    if contrast != 0.0 {
        let factor = (1.015686 * (contrast + 1.0)) / (1.0 * (1.015686 - contrast.clamp(-0.99, 0.99)));
        luma = (luma - 0.5) * factor + 0.5;
    }

    // Clamp before gamma
    luma = luma.clamp(0.0, 1.0);

    // Gamma correction
    if (gamma - 1.0).abs() > 0.001 && gamma > 0.0 {
        luma = luma.powf(1.0 / gamma);
    }

    // Inversion (e.g. for dark mode / white background differences)
    if invert {
        luma = 1.0 - luma;
    }

    luma.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_luma() {
        assert_eq!(rgb_to_luma(Rgb::new(0, 0, 0)), 0.0);
        let white_luma = rgb_to_luma(Rgb::new(255, 255, 255));
        assert!((white_luma - 1.0).abs() < 1e-4);

        // Green has highest perceived weight
        let green = rgb_to_luma(Rgb::new(0, 255, 0));
        let red = rgb_to_luma(Rgb::new(255, 0, 0));
        let blue = rgb_to_luma(Rgb::new(0, 0, 255));
        assert!(green > red);
        assert!(red > blue);
    }

    #[test]
    fn test_adjust_luminance_invert() {
        let luma = 0.2;
        let adjusted = adjust_luminance(luma, 0.0, 0.0, 1.0, true);
        assert!((adjusted - 0.8).abs() < 1e-4);
    }

    #[test]
    fn test_adjust_luminance_brightness() {
        let luma = 0.5;
        let brighter = adjust_luminance(luma, 0.2, 0.0, 1.0, false);
        assert!((brighter - 0.7).abs() < 1e-4);

        let clamped = adjust_luminance(luma, 0.8, 0.0, 1.0, false);
        assert_eq!(clamped, 1.0);
    }
}
