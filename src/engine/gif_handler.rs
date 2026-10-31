use super::frame::AsciiFrame;
use super::renderer::{ImageRenderer, RenderOptions};
use super::types::{ColorMode, ConversionConfig, Rgb};
use super::AsciiConverter;
use image::codecs::gif::{GifDecoder, GifEncoder, Repeat};
use image::{AnimationDecoder, Delay, DynamicImage, Frame};
use rayon::prelude::*;
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
            let ms = num.checked_div(denom).unwrap_or(100);
            let duration = Duration::from_millis(ms.max(20) as u64);
            let dynamic_img = DynamicImage::ImageRgba8(frame.into_buffer());
            extracted.push((dynamic_img, duration));
        }

        Ok(Self {
            frames: extracted,
            ascii_frames: Vec::new(),
        })
    }

    /// Converts all GIF frames to ASCII frames with the given config in parallel using Rayon
    pub fn update_ascii_frames(&mut self, config: &ConversionConfig) {
        self.ascii_frames = self
            .frames
            .par_iter()
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

        // Parallel render all ASCII frames into RGBA images using Rayon
        let rendered_frames: Result<Vec<image::RgbaImage>, String> = self
            .ascii_frames
            .par_iter()
            .map(|frame| ImageRenderer::render_frame(frame, font_bytes, options))
            .collect();
        let rendered_frames = rendered_frames?;

        let mut out_buffer = Cursor::new(Vec::new());
        {
            let mut encoder = GifEncoder::new(&mut out_buffer);
            encoder
                .set_repeat(Repeat::Infinite)
                .map_err(|e| format!("GIF repeat error: {}", e))?;

            for (idx, rgba_img) in rendered_frames.into_iter().enumerate() {
                let duration = self
                    .frames
                    .get(idx)
                    .map(|f| f.1)
                    .unwrap_or(Duration::from_millis(100));

                let delay = Delay::from_numer_denom_ms(duration.as_millis() as u32, 1);
                let frame = Frame::from_parts(rgba_img, 0, 0, delay);

                encoder
                    .encode_frame(frame)
                    .map_err(|e| format!("GIF frame encode error: {}", e))?;
            }
        }

        Ok(out_buffer.into_inner())
    }

    /// Exports the animation to an interactive standalone HTML page with playback controls
    pub fn export_to_animated_html(
        &self,
        bg: Rgb,
        color_mode: ColorMode,
    ) -> Result<String, String> {
        if self.ascii_frames.is_empty() {
            return Err("No ASCII frames available".to_string());
        }

        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"utf-8\">\n<title>ASCII Art Animation</title>\n");
        html.push_str("<style>\n");
        html.push_str(&format!(
            "body {{ background-color: {}; color: #fff; margin: 0; padding: 24px; display: flex; flex-direction: column; align-items: center; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; }}\n",
            bg.to_hex()
        ));
        html.push_str(".controls { margin-bottom: 16px; display: flex; gap: 12px; align-items: center; background: rgba(255,255,255,0.06); padding: 8px 16px; border-radius: 8px; }\n");
        html.push_str("button { background: #1e8ce6; color: white; border: none; padding: 8px 16px; border-radius: 6px; cursor: pointer; font-size: 14px; font-weight: bold; transition: background 0.15s; }\n");
        html.push_str("button:hover { background: #1470be; }\n");
        html.push_str("span#frameInfo { font-size: 13px; color: #aaa; }\n");
        html.push_str("pre { font-family: 'Noto Sans Mono', 'Consolas', 'Courier New', monospace; line-height: 1.0; font-size: 10px; letter-spacing: 0; white-space: pre; background: rgba(0,0,0,0.4); padding: 20px; border-radius: 10px; box-shadow: 0 4px 20px rgba(0,0,0,0.5); }\n");
        html.push_str("</style>\n</head>\n<body>\n");
        html.push_str("<div class=\"controls\">\n");
        html.push_str("  <button id=\"playBtn\" onclick=\"togglePlay()\">⏸ Пауза</button>\n");
        html.push_str("  <button onclick=\"prevFrame()\">⏮</button>\n");
        html.push_str("  <button onclick=\"nextFrame()\">⏭</button>\n");
        html.push_str(&format!("  <span id=\"frameInfo\">Кадр: 1 / {}</span>\n", self.ascii_frames.len()));
        html.push_str("</div>\n");
        html.push_str("<pre id=\"asciiDisplay\"></pre>\n");

        html.push_str("<script>\n");
        html.push_str("const frameContents = [\n");
        for frame in &self.ascii_frames {
            let mut frame_html = String::new();
            for y in 0..frame.height {
                let row_start = y * frame.width;
                for x in 0..frame.width {
                    let idx = row_start + x;
                    if idx >= frame.characters.len() {
                        break;
                    }
                    let ch = frame.characters[idx];
                    let escaped = match ch {
                        '&' => "&amp;".to_string(),
                        '<' => "&lt;".to_string(),
                        '>' => "&gt;".to_string(),
                        '"' => "&quot;".to_string(),
                        _ => ch.to_string(),
                    };
                    let color = match color_mode {
                        ColorMode::TrueColor => frame.colors.get(idx).copied().unwrap_or(Rgb::WHITE),
                        ColorMode::GreenPhosphor => Rgb::GREEN_PHOSPHOR,
                        ColorMode::AmberCrt => Rgb::AMBER_CRT,
                        ColorMode::Cyberpunk => if x % 2 == 0 { Rgb::CYAN } else { Rgb::MAGENTA },
                        ColorMode::Monochrome => Rgb::WHITE,
                    };
                    frame_html.push_str(&format!("<span style=\\\"color:{}\\\">{}</span>", color.to_hex(), escaped));
                }
                frame_html.push_str("\\n");
            }
            html.push_str(&format!("  \"{}\",\n", frame_html));
        }
        html.push_str("];\n");

        html.push_str("const frameDelays = [");
        for (_, dur) in &self.frames {
            html.push_str(&format!("{},", dur.as_millis().max(20)));
        }
        html.push_str("];\n");

        html.push_str(r#"
let currentIdx = 0;
let isPlaying = true;
let timer = null;

function renderCurrent() {
  document.getElementById('asciiDisplay').innerHTML = frameContents[currentIdx].replace(/\\n/g, '\n');
  document.getElementById('frameInfo').innerText = `Кадр: ${currentIdx + 1} / ${frameContents.length}`;
}

function tick() {
  if (!isPlaying) return;
  currentIdx = (currentIdx + 1) % frameContents.length;
  renderCurrent();
  timer = setTimeout(tick, frameDelays[currentIdx] || 100);
}

function togglePlay() {
  isPlaying = !isPlaying;
  document.getElementById('playBtn').innerText = isPlaying ? '⏸ Пауза' : '▶ Воспроизведение';
  if (isPlaying) {
    tick();
  } else if (timer) {
    clearTimeout(timer);
  }
}

function nextFrame() {
  currentIdx = (currentIdx + 1) % frameContents.length;
  renderCurrent();
}

function prevFrame() {
  currentIdx = (currentIdx - 1 + frameContents.length) % frameContents.length;
  renderCurrent();
}

renderCurrent();
timer = setTimeout(tick, frameDelays[0] || 100);
</script>
</body>
</html>
"#);

        Ok(html)
    }
}
