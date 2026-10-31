use super::frame::AsciiFrame;
use super::renderer::{ImageRenderer, RenderOptions};
use super::types::ConversionConfig;
use super::AsciiConverter;
use image::codecs::gif::{GifDecoder, GifEncoder, Repeat};
use image::{AnimationDecoder, Delay, DynamicImage, Frame};
use std::io::Cursor;
use std::time::Duration;

pub struct GifAnimation {
    pub frames: Vec<(DynamicImage, Duration)>,
    pub ascii_frames: Vec<AsciiFrame>,
}

impl GifAnimation {
    /// Attempts to decode animated GIF frames from raw bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let cursor = Cursor::new(bytes);
        let decoder = GifDecoder::new(cursor).map_err(|e| format!("Failed to decode GIF: {}", e))?;
        let frames_result = decoder.into_frames().collect_frames();
        let frames = frames_result.map_err(|e| format!("Failed to read GIF frames: {}", e))?;

        if frames.is_empty() {
            return Err("GIF has no frames".to_string());
        }

        let mut extracted = Vec::with_capacity(frames.len());
        for frame in frames {
            let (num, denom) = frame.delay().numer_denom_ms();
            let ms = if denom > 0 { num / denom } else { 100 };
            let duration = Duration::from_millis(ms.max(20) as u64);
            let dynamic_img = DynamicImage::ImageRgba8(frame.into_buffer());
            extracted.push((dynamic_img, duration));
        }

        Ok(Self {
            frames: extracted,
            ascii_frames: Vec::new(),
        })
    }

    /// Converts all GIF frames to ASCII frames with the given config
    pub fn update_ascii_frames(&mut self, config: &ConversionConfig) {
        self.ascii_frames = self
            .frames
            .iter()
            .map(|(img, _)| AsciiConverter::convert_image(img, config))
            .collect();
    }

    /// Exports animated ASCII frames to a new animated GIF file
    pub fn export_to_gif(
        &self,
        font_bytes: &[u8],
        options: &RenderOptions,
    ) -> Result<Vec<u8>, String> {
        if self.ascii_frames.is_empty() {
            return Err("No ASCII frames available".to_string());
        }

        let mut out_buffer = Cursor::new(Vec::new());
        {
            let mut encoder = GifEncoder::new(&mut out_buffer);
            encoder
                .set_repeat(Repeat::Infinite)
                .map_err(|e| format!("GIF repeat error: {}", e))?;

            for (idx, ascii_frame) in self.ascii_frames.iter().enumerate() {
                let duration = self.frames.get(idx).map(|f| f.1).unwrap_or(Duration::from_millis(100));
                let rgba_img = ImageRenderer::render_frame(ascii_frame, font_bytes, options)?;

                let delay = Delay::from_numer_denom_ms(duration.as_millis() as u32, 1);
                let frame = Frame::from_parts(rgba_img, 0, 0, delay);

                encoder
                    .encode_frame(frame)
                    .map_err(|e| format!("GIF frame encode error: {}", e))?;
            }
        }

        Ok(out_buffer.into_inner())
    }
}
