use ascii_converter::engine::gif_handler::GifAnimation;
use ascii_converter::engine::renderer::{ImageRenderer, RenderOptions};
use ascii_converter::engine::types::{ColorMode, ConversionConfig, Rgb};
use ascii_converter::engine::AsciiConverter;
use ascii_converter::presets::get_default_presets;
use ascii_converter::sample_image::generate_sample_image;
use ascii_converter::system_font;
use image::codecs::gif::{GifEncoder, Repeat};
use image::{Delay, Frame, RgbaImage};

#[test]
fn test_all_presets_with_sample_image() {
    let sample = generate_sample_image();
    let presets = get_default_presets();

    for preset in presets {
        let frame = AsciiConverter::convert_image(&sample, &preset.config);
        assert!(frame.width > 0, "Preset {} generated 0 width", preset.id);
        assert!(frame.height > 0, "Preset {} generated 0 height", preset.id);
        assert_eq!(
            frame.characters.len(),
            frame.width * frame.height,
            "Characters count mismatch for preset {}",
            preset.id
        );
        let text = frame.to_text();
        assert!(!text.trim().is_empty(), "Text empty for preset {}", preset.id);
    }
}

#[test]
fn test_animated_gif_pipeline() {
    // 1. Create a dummy 2-frame animated GIF in memory
    let mut gif_bytes = Vec::new();
    {
        let mut encoder = GifEncoder::new(&mut gif_bytes);
        encoder.set_repeat(Repeat::Infinite).unwrap();

        let frame1_img = RgbaImage::from_pixel(30, 30, image::Rgba([255, 0, 0, 255]));
        let frame2_img = RgbaImage::from_pixel(30, 30, image::Rgba([0, 255, 0, 255]));

        let delay = Delay::from_numer_denom_ms(100, 1);
        encoder.encode_frame(Frame::from_parts(frame1_img, 0, 0, delay)).unwrap();
        encoder.encode_frame(Frame::from_parts(frame2_img, 0, 0, delay)).unwrap();
    }

    // 2. Decode with GifAnimation
    let mut anim = GifAnimation::from_bytes(&gif_bytes).expect("GIF decoding failed");
    assert_eq!(anim.frames.len(), 2);

    // 3. Convert frames
    let config = ConversionConfig {
        width: 15,
        aspect_ratio: 0.5,
        ..Default::default()
    };
    anim.update_ascii_frames(&config);
    assert_eq!(anim.ascii_frames.len(), 2);

    // 4. Export back to animated GIF
    let font_info = system_font::detect_system_monospace_font();
    let render_opts = RenderOptions::default();
    let exported_gif = anim.export_to_gif(&font_info.font_bytes, &render_opts);

    assert!(exported_gif.is_ok());
    let bytes = exported_gif.unwrap();
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..3], b"GIF");
}

#[test]
fn test_image_renderer_jpeg_and_png() {
    let sample = generate_sample_image();
    let config = ConversionConfig {
        width: 20,
        aspect_ratio: 0.5,
        color_mode: ColorMode::TrueColor,
        ..Default::default()
    };
    let frame = AsciiConverter::convert_image(&sample, &config);
    let font_info = system_font::detect_system_monospace_font();
    let opts = RenderOptions::default();

    // Test PNG
    let png = ImageRenderer::render_to_png(&frame, &font_info.font_bytes, &opts).unwrap();
    assert_eq!(&png[0..4], &[0x89, 0x50, 0x4E, 0x47]);

    // Test JPEG
    let jpeg = ImageRenderer::render_to_jpeg(&frame, &font_info.font_bytes, &opts).unwrap();
    assert_eq!(&jpeg[0..2], &[0xFF, 0xD8]);
}

#[test]
fn test_system_font_detection() {
    let font_info = system_font::detect_system_monospace_font();
    assert!(!font_info.name.is_empty());
    assert!(!font_info.font_bytes.is_empty());
    let parsed = fontdue::Font::from_bytes(&font_info.font_bytes[..], fontdue::FontSettings::default());
    assert!(parsed.is_ok(), "System font could not be parsed by fontdue!");
}

#[test]
fn test_export_formats_content_validity() {
    let sample = generate_sample_image();
    let config = ConversionConfig {
        width: 30,
        aspect_ratio: 0.5,
        ..Default::default()
    };
    let frame = AsciiConverter::convert_image(&sample, &config);

    // Plain text
    let txt = frame.to_text();
    assert_eq!(txt.lines().count(), frame.height);

    // Markdown
    let md = frame.to_markdown();
    assert!(md.starts_with("```text\n"));
    assert!(md.ends_with("```\n"));

    // HTML
    let html = frame.to_html(Rgb::BLACK, ColorMode::GreenPhosphor);
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("<style>"));
    assert!(html.contains("<pre>"));
    assert!(html.contains("</pre>"));

    // SVG
    let svg = frame.to_svg(Rgb::BLACK, "Noto Sans Mono");
    assert!(svg.contains("<svg"));
    assert!(svg.contains("<rect width=\"100%\""));
    assert!(svg.contains("<text font-family=\"Noto Sans Mono\""));
    assert!(svg.contains("</svg>"));
}

#[test]
fn test_sample_gif_and_animated_html_export() {
    let gif_bytes = ascii_converter::sample_image::generate_sample_gif();
    assert!(!gif_bytes.is_empty());
    assert_eq!(&gif_bytes[0..3], b"GIF");

    let mut anim = GifAnimation::from_bytes(&gif_bytes).expect("Sample GIF decoding failed");
    assert_eq!(anim.frames.len(), 12);

    let config = ConversionConfig {
        width: 25,
        aspect_ratio: 0.5,
        color_mode: ColorMode::GreenPhosphor,
        ..Default::default()
    };
    anim.update_ascii_frames(&config);
    assert_eq!(anim.ascii_frames.len(), 12);

    // Test animated HTML export
    let html = anim
        .export_to_animated_html(Rgb::BLACK, ColorMode::GreenPhosphor)
        .expect("Failed to export animated HTML");
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("const frameContents = ["));
    assert!(html.contains("const frameDelays = ["));
    assert!(html.contains("function togglePlay()"));
    assert!(html.contains("asciiDisplay"));
}
