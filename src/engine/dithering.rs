use super::types::DitherMode;

pub const BAYER_4X4: [[f32; 4]; 4] = [
    [0.0 / 16.0, 8.0 / 16.0, 2.0 / 16.0, 10.0 / 16.0],
    [12.0 / 16.0, 4.0 / 16.0, 14.0 / 16.0, 6.0 / 16.0],
    [3.0 / 16.0, 11.0 / 16.0, 1.0 / 16.0, 9.0 / 16.0],
    [15.0 / 16.0, 7.0 / 16.0, 13.0 / 16.0, 5.0 / 16.0],
];

/// Applies dithering to a 2D luminance buffer of size (width x height)
/// Returns a new buffer with quantized luminance values [0.0, 1.0].
pub fn apply_dither(
    buffer: &[f32],
    width: usize,
    height: usize,
    mode: DitherMode,
    levels: usize,
) -> Vec<f32> {
    if width == 0 || height == 0 || buffer.len() != width * height {
        return Vec::new();
    }

    let steps = if levels > 1 { (levels - 1) as f32 } else { 1.0 };
    let quantize = |val: f32| -> f32 {
        let clamped = val.clamp(0.0, 1.0);
        (clamped * steps).round() / steps
    };

    match mode {
        DitherMode::None => buffer.iter().map(|&v| quantize(v)).collect(),

        DitherMode::Bayer4x4 => {
            let mut out = Vec::with_capacity(buffer.len());
            for y in 0..height {
                for x in 0..width {
                    let orig = buffer[y * width + x];
                    let threshold = BAYER_4X4[y % 4][x % 4] - 0.5;
                    let spread = 1.0 / steps;
                    let dithered = orig + threshold * spread;
                    out.push(quantize(dithered));
                }
            }
            out
        }

        DitherMode::FloydSteinberg => {
            let mut work = buffer.to_vec();
            let mut out = vec![0.0; buffer.len()];

            for y in 0..height {
                for x in 0..width {
                    let idx = y * width + x;
                    let old_val = work[idx];
                    let new_val = quantize(old_val);
                    out[idx] = new_val;
                    let err = old_val - new_val;

                    // (x + 1, y): 7/16
                    if x + 1 < width {
                        work[y * width + (x + 1)] += err * (7.0 / 16.0);
                    }
                    // (x - 1, y + 1): 3/16
                    if x > 0 && y + 1 < height {
                        work[(y + 1) * width + (x - 1)] += err * (3.0 / 16.0);
                    }
                    // (x, y + 1): 5/16
                    if y + 1 < height {
                        work[(y + 1) * width + x] += err * (5.0 / 16.0);
                    }
                    // (x + 1, y + 1): 1/16
                    if x + 1 < width && y + 1 < height {
                        work[(y + 1) * width + (x + 1)] += err * (1.0 / 16.0);
                    }
                }
            }
            out
        }

        DitherMode::Atkinson => {
            let mut work = buffer.to_vec();
            let mut out = vec![0.0; buffer.len()];

            for y in 0..height {
                for x in 0..width {
                    let idx = y * width + x;
                    let old_val = work[idx];
                    let new_val = quantize(old_val);
                    out[idx] = new_val;
                    let err = old_val - new_val;
                    let eighth = err * (1.0 / 8.0);

                    // (x + 1, y)
                    if x + 1 < width {
                        work[y * width + (x + 1)] += eighth;
                    }
                    // (x + 2, y)
                    if x + 2 < width {
                        work[y * width + (x + 2)] += eighth;
                    }
                    // (x - 1, y + 1)
                    if x > 0 && y + 1 < height {
                        work[(y + 1) * width + (x - 1)] += eighth;
                    }
                    // (x, y + 1)
                    if y + 1 < height {
                        work[(y + 1) * width + x] += eighth;
                    }
                    // (x + 1, y + 1)
                    if x + 1 < width && y + 1 < height {
                        work[(y + 1) * width + (x + 1)] += eighth;
                    }
                    // (x, y + 2)
                    if y + 2 < height {
                        work[(y + 2) * width + x] += eighth;
                    }
                }
            }
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dither_dimensions() {
        let buf = vec![0.5; 16];
        let res = apply_dither(&buf, 4, 4, DitherMode::FloydSteinberg, 4);
        assert_eq!(res.len(), 16);
        for v in res {
            assert!(v >= 0.0 && v <= 1.0);
        }
    }
}
