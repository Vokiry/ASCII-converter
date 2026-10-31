/// Computes edge outline characters using Sobel filter
pub fn detect_edges(
    luma_buffer: &[f32],
    width: usize,
    height: usize,
    threshold: f32,
) -> Vec<char> {
    if width < 3 || height < 3 || luma_buffer.len() != width * height {
        return vec![' '; width * height];
    }

    let mut out = vec![' '; width * height];

    for y in 1..(height - 1) {
        for x in 1..(width - 1) {
            let get = |dx: isize, dy: isize| -> f32 {
                let px = (x as isize + dx) as usize;
                let py = (y as isize + dy) as usize;
                luma_buffer[py * width + px]
            };

            // Sobel convolution
            let gx = -1.0 * get(-1, -1) + 1.0 * get(1, -1)
                - 2.0 * get(-1,  0) + 2.0 * get(1,  0)
                - 1.0 * get(-1,  1) + 1.0 * get(1,  1);

            let gy = 1.0 * get(-1, -1) + 2.0 * get(0, -1) + 1.0 * get(1, -1)
                - 1.0 * get(-1,  1) - 2.0 * get(0,  1) - 1.0 * get(1,  1);

            let magnitude = (gx * gx + gy * gy).sqrt();

            if magnitude >= threshold {
                let angle = gy.atan2(gx).to_degrees().abs(); // 0 to 180

                // Map angle to directional stroke
                let ch = if angle >= 67.5 && angle <= 112.5 {
                    '-'
                } else if angle <= 22.5 || angle >= 157.5 {
                    '|'
                } else if gx * gy > 0.0 {
                    '/'
                } else {
                    '\\'
                };
                out[y * width + x] = ch;
            } else {
                out[y * width + x] = ' ';
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertical_edge() {
        // 5x5 image with sharp vertical split (left black, right white)
        let mut buf = vec![0.0; 25];
        for y in 0..5 {
            for x in 3..5 {
                buf[y * 5 + x] = 1.0;
            }
        }
        let edges = detect_edges(&buf, 5, 5, 0.5);
        // Middle column should detect vertical edge
        assert_eq!(edges[2 * 5 + 2], '|');
    }
}
