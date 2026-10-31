use ascii_converter::app::AsciiApp;
use ascii_converter::system_font;
use eframe::egui;

fn main() -> eframe::Result {
    let font_info = system_font::detect_system_monospace_font();
    println!("Detected system monospace font: {}", font_info.name);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("ASCII Studio - Image to ASCII Converter")
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([850.0, 550.0]),
        ..Default::default()
    };

    eframe::run_native(
        "ASCII Studio",
        native_options,
        Box::new(move |cc| {
            // Apply dark theme by default
            cc.egui_ctx.set_theme(egui::Theme::Dark);
            Ok(Box::new(AsciiApp::new(cc, font_info)))
        }),
    )
}
