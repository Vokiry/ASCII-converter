use super::types::{ColorMode, Rgb};

#[derive(Clone, Debug, PartialEq)]
pub struct AsciiFrame {
    pub width: usize,
    pub height: usize,
    pub characters: Vec<char>,
    pub colors: Vec<Rgb>,
}

impl AsciiFrame {
    pub fn new(width: usize, height: usize, characters: Vec<char>, colors: Vec<Rgb>) -> Self {
        Self {
            width,
            height,
            characters,
            colors,
        }
    }

    pub fn empty() -> Self {
        Self {
            width: 0,
            height: 0,
            characters: Vec::new(),
            colors: Vec::new(),
        }
    }

    /// Converts frame into plain text lines separated by '\n'
    pub fn to_text(&self) -> String {
        if self.width == 0 || self.height == 0 {
            return String::new();
        }

        let mut s = String::with_capacity(self.characters.len() + self.height);
        for y in 0..self.height {
            let start = y * self.width;
            let end = (start + self.width).min(self.characters.len());
            for &ch in &self.characters[start..end] {
                s.push(ch);
            }
            s.push('\n');
        }
        s
    }

    /// Converts frame into Markdown monospace codeblock:
    /// ```text
    /// ...
    /// ```
    pub fn to_markdown(&self) -> String {
        format!("```text\n{}```\n", self.to_text())
    }

    /// Converts frame into standalone HTML with embedded styles
    pub fn to_html(&self, bg: Rgb, color_mode: ColorMode) -> String {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"utf-8\">\n<title>ASCII Art</title>\n");
        html.push_str("<style>\n");
        html.push_str(&format!(
            "body {{ background-color: {}; margin: 20px; display: flex; justify-content: center; }}\n",
            bg.to_hex()
        ));
        html.push_str("pre { font-family: 'Noto Sans Mono', 'Consolas', 'Courier New', monospace; line-height: 1.0; font-size: 10px; letter-spacing: 0; white-space: pre; }\n");
        html.push_str("</style>\n</head>\n<body>\n<pre>");

        for y in 0..self.height {
            let row_start = y * self.width;
            for x in 0..self.width {
                let idx = row_start + x;
                if idx >= self.characters.len() {
                    break;
                }
                let ch = self.characters[idx];
                let escaped = match ch {
                    '&' => "&amp;".to_string(),
                    '<' => "&lt;".to_string(),
                    '>' => "&gt;".to_string(),
                    '"' => "&quot;".to_string(),
                    _ => ch.to_string(),
                };

                let color = match color_mode {
                    ColorMode::TrueColor => self.colors.get(idx).copied().unwrap_or(Rgb::WHITE),
                    ColorMode::GreenPhosphor => Rgb::GREEN_PHOSPHOR,
                    ColorMode::AmberCrt => Rgb::AMBER_CRT,
                    ColorMode::Cyberpunk => {
                        if x % 2 == 0 { Rgb::CYAN } else { Rgb::MAGENTA }
                    }
                    ColorMode::Monochrome => Rgb::WHITE,
                };

                html.push_str(&format!("<span style=\"color:{}\">{}</span>", color.to_hex(), escaped));
            }
            html.push('\n');
        }

        html.push_str("</pre>\n</body>\n</html>\n");
        html
    }

    /// Converts frame into Scalable Vector Graphics (SVG)
    pub fn to_svg(&self, bg: Rgb, font_family: &str) -> String {
        let char_w = 8.0;
        let char_h = 16.0;
        let total_w = self.width as f32 * char_w;
        let total_h = self.height as f32 * char_h;

        let mut svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{:.1}\" height=\"{:.1}\" viewBox=\"0 0 {:.1} {:.1}\">\n",
            total_w, total_h, total_w, total_h
        );
        svg.push_str(&format!(
            "  <rect width=\"100%\" height=\"100%\" fill=\"{}\" />\n",
            bg.to_hex()
        ));
        svg.push_str(&format!(
            "  <text font-family=\"{}\" font-size=\"14px\" xml:space=\"preserve\">\n",
            font_family
        ));

        for y in 0..self.height {
            let y_pos = (y as f32 + 0.85) * char_h;
            for x in 0..self.width {
                let idx = y * self.width + x;
                if idx >= self.characters.len() {
                    break;
                }
                let ch = self.characters[idx];
                if ch == ' ' {
                    continue;
                }
                let x_pos = x as f32 * char_w;
                let color = self.colors.get(idx).copied().unwrap_or(Rgb::WHITE);
                let escaped = match ch {
                    '&' => "&amp;",
                    '<' => "&lt;",
                    '>' => "&gt;",
                    '"' => "&quot;",
                    _ => "",
                };

                if escaped.is_empty() {
                    svg.push_str(&format!(
                        "    <tspan x=\"{:.1}\" y=\"{:.1}\" fill=\"{}\">{}</tspan>\n",
                        x_pos, y_pos, color.to_hex(), ch
                    ));
                } else {
                    svg.push_str(&format!(
                        "    <tspan x=\"{:.1}\" y=\"{:.1}\" fill=\"{}\">{}</tspan>\n",
                        x_pos, y_pos, color.to_hex(), escaped
                    ));
                }
            }
        }

        svg.push_str("  </text>\n</svg>\n");
        svg
    }
}
