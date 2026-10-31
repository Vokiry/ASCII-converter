use arboard::{Clipboard, ImageData};
use image::RgbaImage;
use std::borrow::Cow;

pub struct ClipboardHelper;

impl ClipboardHelper {
    /// Copies plain text to system clipboard
    pub fn copy_text(text: &str) -> Result<(), String> {
        let mut clipboard = Clipboard::new().map_err(|e| format!("Clipboard error: {}", e))?;
        clipboard
            .set_text(text)
            .map_err(|e| format!("Failed to set clipboard text: {}", e))
    }

    /// Copies an RgbaImage to system clipboard
    pub fn copy_image(img: &RgbaImage) -> Result<(), String> {
        let mut clipboard = Clipboard::new().map_err(|e| format!("Clipboard error: {}", e))?;
        let image_data = ImageData {
            width: img.width() as usize,
            height: img.height() as usize,
            bytes: Cow::Borrowed(img.as_raw()),
        };
        clipboard
            .set_image(image_data)
            .map_err(|e| format!("Failed to set clipboard image: {}", e))
    }

    /// Reads an image from system clipboard if available
    pub fn read_image() -> Result<Option<image::DynamicImage>, String> {
        let mut clipboard = Clipboard::new().map_err(|e| format!("Clipboard error: {}", e))?;
        match clipboard.get_image() {
            Ok(img_data) => {
                let width = img_data.width as u32;
                let height = img_data.height as u32;
                let buffer = img_data.bytes.into_owned();
                let rgba_img = RgbaImage::from_raw(width, height, buffer)
                    .ok_or_else(|| "Failed to construct RGBA image from clipboard".to_string())?;
                Ok(Some(image::DynamicImage::ImageRgba8(rgba_img)))
            }
            Err(arboard::Error::ContentNotAvailable) => Ok(None),
            Err(e) => Err(format!("Clipboard read error: {}", e)),
        }
    }
}
