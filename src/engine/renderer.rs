use super::frame::AsciiFrame;
use super::types::{ColorMode, Rgb};
use fontdue::{Font, FontSettings};
use image::{Rgba, RgbaImage};
use std::io::Cursor;

/// Options for rendering ASCII to an image file
#[derive(Clone, Debug)]
pub struct RenderOptions {
    pub font_size: f32,
    pub padding: u32,
    pub background: Rgb,
    pub default_text_color: Rgb,
    pub color_mode: ColorMode,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            padding: 16,
            background: Rgb::new(18, 18, 18),
            default_text_color: Rgb::WHITE,
            color_mode: ColorMode::Monochrome,
        }
    }
}

pub struct ImageRenderer;

impl ImageRenderer {
    /// Renders an AsciiFrame to an RgbaImage using the provided font bytes
    pub fn render_frame(
        frame: &AsciiFrame,
        font_bytes: &[u8],
        options: &RenderOptions,
    ) -> Result<RgbaImage, String> {
        if frame.width == 0 || frame.height == 0 {
            return Err("Frame is empty".to_string());
        }

        let font = Font::from_bytes(font_bytes, FontSettings::default())
            .map_err(|e| format!("Failed to parse font: {}", e))?;

        // Monospace metrics: measure 'M' or '0'
        let test_metrics = font.metrics('0', options.font_size);
        let cell_w = (test_metrics.advance_width.round() as u32).max(8);
        let line_h = (options.font_size * 1.25).round() as u32;

        let total_w = frame.width as u32 * cell_w + options.padding * 2;
        let total_h = frame.height as u32 * line_h + options.padding * 2;

        let bg = options.background;
        let bg_pixel = Rgba([bg.r, bg.g, bg.b, 255]);
        let mut img = RgbaImage::from_pixel(total_w, total_h, bg_pixel);

        let baseline_offset = (options.font_size * 0.95).round() as i32;

        for y in 0..frame.height {
            let row_start = y * frame.width;
            let cell_y = options.padding + (y as u32 * line_h);

            for x in 0..frame.width {
                let idx = row_start + x;
                if idx >= frame.characters.len() {
                    break;
                }
                let ch = frame.characters[idx];
                if ch == ' ' {
                    continue;
                }

                let color = match options.color_mode {
                    ColorMode::TrueColor => frame.colors.get(idx).copied().unwrap_or(options.default_text_color),
                    ColorMode::GreenPhosphor => Rgb::GREEN_PHOSPHOR,
                    ColorMode::AmberCrt => Rgb::AMBER_CRT,
                    ColorMode::Cyberpunk => {
                        if x % 2 == 0 { Rgb::CYAN } else { Rgb::MAGENTA }
                    }
                    ColorMode::Monochrome => options.default_text_color,
                };

                let cell_x = options.padding + (x as u32 * cell_w);
                let (metrics, bitmap) = font.rasterize(ch, options.font_size);

                let glyph_x = cell_x as i32 + metrics.xmin;
                let glyph_y = cell_y as i32 + baseline_offset - metrics.ymin - metrics.height as i32;

                for gy in 0..metrics.height {
                    for gx in 0..metrics.width {
                        let alpha = bitmap[gy * metrics.width + gx];
                        if alpha == 0 {
                            continue;
                        }

                        let px = glyph_x + gx as i32;
                        let py = glyph_y + gy as i32;

                        if px >= 0 && px < total_w as i32 && py >= 0 && py < total_h as i32 {
                            let target = img.get_pixel_mut(px as u32, py as u32);
                            let a = alpha as f32 / 255.0;
                            let inv_a = 1.0 - a;

                            target[0] = (color.r as f32 * a + target[0] as f32 * inv_a).round() as u8;
                            target[1] = (color.g as f32 * a + target[1] as f32 * inv_a).round() as u8;
                            target[2] = (color.b as f32 * a + target[2] as f32 * inv_a).round() as u8;
                            target[3] = 255;
                        }
                    }
                }
            }
        }

        Ok(img)
    }

    /// Renders frame to PNG byte buffer
    pub fn render_to_png(
        frame: &AsciiFrame,
        font_bytes: &[u8],
        options: &RenderOptions,
    ) -> Result<Vec<u8>, String> {
        let img = Self::render_frame(frame, font_bytes, options)?;
        let mut buffer = Cursor::new(Vec::new());
        img.write_to(&mut buffer, image::ImageFormat::Png)
            .map_err(|e| format!("PNG encoding error: {}", e))?;
        Ok(buffer.into_inner())
    }

    /// Renders frame to JPEG byte buffer
    pub fn render_to_jpeg(
        frame: &AsciiFrame,
        font_bytes: &[u8],
        options: &RenderOptions,
    ) -> Result<Vec<u8>, String> {
        let img = Self::render_frame(frame, font_bytes, options)?;
        let rgb_img = image::DynamicImage::ImageRgba8(img).to_rgb8();
        let mut buffer = Cursor::new(Vec::new());
        rgb_img.write_to(&mut buffer, image::ImageFormat::Jpeg)
            .map_err(|e| format!("JPEG encoding error: {}", e))?;
        Ok(buffer.into_inner())
    }
}
