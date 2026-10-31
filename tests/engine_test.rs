use ascii_converter::engine::dithering;
use ascii_converter::engine::frame::AsciiFrame;
use ascii_converter::engine::luminance;
use ascii_converter::engine::palettes::CharPalette;
use ascii_converter::engine::renderer::{ImageRenderer, RenderOptions};
use ascii_converter::engine::types::{ColorMode, ConversionConfig, DitherMode, PaletteType, Rgb};
use ascii_converter::engine::AsciiConverter;
use ascii_converter::system_font;
use image::{DynamicImage, RgbImage};

#[test]
fn test_luminance_and_adjust() {
    let black = Rgb::BLACK;
    let white = Rgb::WHITE;
    assert_eq!(luminance::rgb_to_luma(black), 0.0);
    assert!((luminance::rgb_to_luma(white) - 1.0).abs() < 1e-4);

    let adj = luminance::adjust_luminance(0.5, 0.2, 0.0, 1.0, false);
    assert!((adj - 0.7).abs() < 1e-4);

    let inv = luminance::adjust_luminance(0.2, 0.0, 0.0, 1.0, true);
    assert!((inv - 0.8).abs() < 1e-4);
}

#[test]
fn test_palette_selection() {
    let p = CharPalette::from_type(&PaletteType::Standard10);
    assert_eq!(p.get_char(0.0), ' ');
    assert_eq!(p.get_char(1.0), '@');
}

#[test]
fn test_dithering_execution() {
    let buf = vec![0.3, 0.7, 0.4, 0.9];
    let res = dithering::apply_dither(&buf, 2, 2, DitherMode::FloydSteinberg, 4);
    assert_eq!(res.len(), 4);
    for val in res {
        assert!(val >= 0.0 && val <= 1.0);
    }
}

#[test]
fn test_convert_solid_image() {
    let img = DynamicImage::ImageRgb8(RgbImage::from_pixel(50, 50, image::Rgb([255, 255, 255])));
    let config = ConversionConfig {
        width: 10,
        aspect_ratio: 0.5,
        palette: PaletteType::Standard10,
        ..Default::default()
    };

    let frame = AsciiConverter::convert_image(&img, &config);
    assert_eq!(frame.width, 10);
    assert!(frame.height >= 4 && frame.height <= 6);
    for &ch in &frame.characters {
        assert_eq!(ch, '@');
    }

    let text = frame.to_text();
    assert!(!text.is_empty());
    assert_eq!(text.lines().count(), frame.height);
}

#[test]
fn test_braille_conversion() {
    let img = DynamicImage::ImageRgb8(RgbImage::from_pixel(40, 40, image::Rgb([255, 255, 255])));
    let config = ConversionConfig {
        width: 10,
        aspect_ratio: 0.5,
        palette: PaletteType::Braille,
        ..Default::default()
    };

    let frame = AsciiConverter::convert_image(&img, &config);
    assert_eq!(frame.width, 10);
    for &ch in &frame.characters {
        assert_eq!(ch, '\u{28FF}');
    }
}

#[test]
fn test_html_and_svg_generation() {
    let chars = vec!['A', 'B', 'C', 'D'];
    let colors = vec![Rgb::WHITE; 4];
    let frame = AsciiFrame::new(2, 2, chars, colors);

    let html = frame.to_html(Rgb::BLACK, ColorMode::Monochrome);
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("span style=\"color:#ffffff\">A</span>"));

    let svg = frame.to_svg(Rgb::BLACK, "monospace");
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
}

#[test]
fn test_png_image_rendering() {
    let chars = vec!['#', '#', '#', '#'];
    let colors = vec![Rgb::WHITE; 4];
    let frame = AsciiFrame::new(2, 2, chars, colors);

    let font_info = system_font::detect_system_monospace_font();
    let render_opts = RenderOptions::default();
    let png_bytes = ImageRenderer::render_to_png(&frame, &font_info.font_bytes, &render_opts);

    assert!(png_bytes.is_ok());
    let bytes = png_bytes.unwrap();
    assert!(!bytes.is_empty());
    // PNG magic header: 0x89, 'P', 'N', 'G'
    assert_eq!(&bytes[0..4], &[0x89, 0x50, 0x4E, 0x47]);
}
