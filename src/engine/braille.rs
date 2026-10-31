/// Generates Braille Unicode characters from a high-resolution luminance buffer.
/// Each Braille glyph represents a 2 (width) x 4 (height) sub-pixel grid.
///
/// Returns:
/// (braille_chars, char_width, char_height)
pub fn generate_braille(
    sub_buffer: &[f32],
    sub_width: usize,
    sub_height: usize,
    threshold: f32,
) -> (Vec<char>, usize, usize) {
    let char_width = sub_width / 2;
    let char_height = sub_height / 4;

    if char_width == 0 || char_height == 0 {
        return (Vec::new(), 0, 0);
    }

    let mut out = Vec::with_capacity(char_width * char_height);

    for cy in 0..char_height {
        for cx in 0..char_width {
            let mut byte: u32 = 0;

            // Helper to check sub-pixel dot
            let check_dot = |dx: usize, dy: usize| -> bool {
                let px = cx * 2 + dx;
                let py = cy * 4 + dy;
                if px < sub_width && py < sub_height {
                    sub_buffer[py * sub_width + px] >= threshold
                } else {
                    false
                }
            };

            // Dot 1: (0, 0) -> 0x01
            if check_dot(0, 0) { byte |= 0x01; }
            // Dot 2: (0, 1) -> 0x02
            if check_dot(0, 1) { byte |= 0x02; }
            // Dot 3: (0, 2) -> 0x04
            if check_dot(0, 2) { byte |= 0x04; }
            // Dot 4: (1, 0) -> 0x08
            if check_dot(1, 0) { byte |= 0x08; }
            // Dot 5: (1, 1) -> 0x10
            if check_dot(1, 1) { byte |= 0x10; }
            // Dot 6: (1, 2) -> 0x20
            if check_dot(1, 2) { byte |= 0x20; }
            // Dot 7: (0, 3) -> 0x40
            if check_dot(0, 3) { byte |= 0x40; }
            // Dot 8: (1, 3) -> 0x80
            if check_dot(1, 3) { byte |= 0x80; }

            // Unicode Braille starts at 0x2800
            let ch = char::from_u32(0x2800 + byte).unwrap_or(' ');
            out.push(ch);
        }
    }

    (out, char_width, char_height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_braille_empty_and_full() {
        // 2x4 subpixels: all 0.0 -> \u{2800} (empty braille pattern)
        let empty_buf = vec![0.0; 8];
        let (res_empty, w, h) = generate_braille(&empty_buf, 2, 4, 0.5);
        assert_eq!(w, 1);
        assert_eq!(h, 1);
        assert_eq!(res_empty[0], '\u{2800}');

        // 2x4 subpixels: all 1.0 -> \u{28FF} (full 8 dots)
        let full_buf = vec![1.0; 8];
        let (res_full, _, _) = generate_braille(&full_buf, 2, 4, 0.5);
        assert_eq!(res_full[0], '\u{28FF}');
    }
}
