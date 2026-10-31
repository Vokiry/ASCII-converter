use crate::clipboard::ClipboardHelper;
use crate::engine::frame::AsciiFrame;
use crate::engine::gif_handler::GifAnimation;
use crate::engine::renderer::{ImageRenderer, RenderOptions};
use crate::engine::types::{ColorMode, ConversionConfig, DitherMode, PaletteType, Rgb};
use crate::engine::AsciiConverter;
use crate::presets::{get_default_presets, Preset};
use crate::sample_image::generate_sample_image;
use crate::system_font::SystemFontInfo;
use egui::text::{LayoutJob, TextFormat};
use egui::{vec2, Color32, FontId, Key, RichText, ScrollArea};
use image::DynamicImage;
use std::time::{Duration, Instant};

pub struct AsciiApp {
    /// System font information
    pub font_info: SystemFontInfo,
    /// Currently loaded original image
    pub original_image: Option<DynamicImage>,
    /// File path of currently loaded image, if opened from disk
    pub image_path: Option<String>,
    /// Animated GIF handler, if a GIF was loaded
    pub gif_animation: Option<GifAnimation>,
    pub current_frame_idx: usize,
    pub is_playing: bool,
    pub last_frame_time: Instant,
    /// Current conversion settings
    pub config: ConversionConfig,
    /// Converted ASCII frame
    pub current_ascii_frame: Option<AsciiFrame>,
    /// Cached plain text
    pub cached_text: String,
    /// Dirty flag indicating settings or frame changed
    pub dirty: bool,
    /// UI state
    pub advanced_mode: bool,
    pub custom_palette_text: String,
    pub zoom_level: f32,
    pub font_size: f32,
    pub show_original: bool,
    /// Feedback toast message (text, timestamp)
    pub toast: Option<(String, Instant)>,
    /// Available presets
    pub presets: Vec<Preset>,
    pub selected_preset_id: Option<String>,
    /// Render options for image export
    pub render_options: RenderOptions,
    /// Image preview texture in egui
    pub original_texture: Option<egui::TextureHandle>,
}

impl AsciiApp {
    pub fn new(cc: &eframe::CreationContext<'_>, font_info: SystemFontInfo) -> Self {
        crate::system_font::configure_egui_fonts(&cc.egui_ctx, &font_info);

        let default_presets = get_default_presets();
        let initial_preset = default_presets.first().cloned();

        let mut app = Self {
            font_info,
            original_image: None,
            image_path: None,
            gif_animation: None,
            current_frame_idx: 0,
            is_playing: false,
            last_frame_time: Instant::now(),
            config: initial_preset.as_ref().map(|p| p.config.clone()).unwrap_or_default(),
            current_ascii_frame: None,
            cached_text: String::new(),
            dirty: false,
            advanced_mode: false,
            custom_palette_text: "$@B%8&WM#*oahkbdpqwmZO0QLCJUYXzcvunxrjft/\\|()1{}[]?-_+~<>i!lI;:,\"^`'. ".to_string(),
            zoom_level: 1.0,
            font_size: 11.0,
            show_original: false,
            toast: None,
            presets: default_presets,
            selected_preset_id: initial_preset.map(|p| p.id.to_string()),
            render_options: RenderOptions::default(),
            original_texture: None,
        };

        // Load built-in sample image so the user has something gorgeous right upon launch!
        let sample = generate_sample_image();
        app.load_dynamic_image(sample, Some("Пример изображения (Компьютер CRT)".to_string()), &cc.egui_ctx);

        app
    }

    /// Loads a new image from DynamicImage
    pub fn load_dynamic_image(&mut self, img: DynamicImage, title: Option<String>, ctx: &egui::Context) {
        self.gif_animation = None;
        self.current_frame_idx = 0;
        self.is_playing = false;
        self.image_path = title;

        // Create texture for original preview
        let rgba = img.to_rgba8();
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [rgba.width() as usize, rgba.height() as usize],
            rgba.as_raw(),
        );
        self.original_texture = Some(ctx.load_texture("orig_img", color_image, Default::default()));

        self.original_image = Some(img);
        self.dirty = true;
    }

    /// Loads a GIF file from bytes
    pub fn load_gif_bytes(&mut self, bytes: &[u8], title: Option<String>, ctx: &egui::Context) -> Result<(), String> {
        let mut anim = GifAnimation::from_bytes(bytes)?;
        anim.update_ascii_frames(&self.config);

        if let Some((first_frame, _)) = anim.frames.first() {
            let rgba = first_frame.to_rgba8();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [rgba.width() as usize, rgba.height() as usize],
                rgba.as_raw(),
            );
            self.original_texture = Some(ctx.load_texture("orig_gif", color_image, Default::default()));
            self.original_image = Some(first_frame.clone());
        }

        self.gif_animation = Some(anim);
        self.current_frame_idx = 0;
        self.is_playing = true;
        self.image_path = title;
        self.dirty = true;
        Ok(())
    }

    /// Loads image file from disk path
    pub fn load_file_from_path(&mut self, path: &std::path::Path, ctx: &egui::Context) {
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("file").to_string();

        if ext == "gif"
            && let Ok(bytes) = std::fs::read(path)
        {
            if let Err(e) = self.load_gif_bytes(&bytes, Some(name), ctx) {
                self.show_toast(format!("Ошибка загрузки GIF: {}", e));
            } else {
                self.show_toast("Анимированный GIF успешно загружен!".to_string());
            }
            return;
        }

        match image::open(path) {
            Ok(img) => {
                self.load_dynamic_image(img, Some(name), ctx);
                self.show_toast("Изображение успешно загружено!".to_string());
            }
            Err(e) => {
                self.show_toast(format!("Не удалось открыть файл: {}", e));
            }
        }
    }

    /// Shows a temporary toast message
    pub fn show_toast(&mut self, msg: String) {
        self.toast = Some((msg, Instant::now()));
    }

    /// Recomputes current ASCII frame
    pub fn recompute(&mut self) {
        if let Some(gif) = &mut self.gif_animation {
            gif.update_ascii_frames(&self.config);
            if let Some(frame) = gif.ascii_frames.get(self.current_frame_idx) {
                self.cached_text = frame.to_text();
                self.current_ascii_frame = Some(frame.clone());
            }
        } else if let Some(img) = &self.original_image {
            let frame = AsciiConverter::convert_image(img, &self.config);
            self.cached_text = frame.to_text();
            self.current_ascii_frame = Some(frame);
        }
        self.dirty = false;
    }

    /// Applies a preset
    pub fn apply_preset(&mut self, preset: &Preset) {
        self.config = preset.config.clone();
        self.selected_preset_id = Some(preset.id.to_string());
        self.dirty = true;
        self.show_toast(format!("Применен пресет: {}", preset.name_ru));
    }

    /// Prominent Copy Text action
    pub fn copy_text_to_clipboard(&mut self) {
        if self.cached_text.is_empty() {
            self.show_toast("Нет данных для копирования!".to_string());
            return;
        }

        match ClipboardHelper::copy_text(&self.cached_text) {
            Ok(_) => {
                self.show_toast("✅ Арт скопирован в буфер обмена (текст)!".to_string());
            }
            Err(e) => {
                self.show_toast(format!("Ошибка копирования в буфер: {}", e));
            }
        }
    }

    /// Copy as rendered PNG image action
    pub fn copy_image_to_clipboard(&mut self) {
        let frame = match &self.current_ascii_frame {
            Some(f) => f,
            None => {
                self.show_toast("Нет данных для копирования!".to_string());
                return;
            }
        };

        let mut opts = self.render_options.clone();
        opts.color_mode = self.config.color_mode;

        match ImageRenderer::render_frame(frame, &self.font_info.font_bytes, &opts) {
            Ok(rgba_img) => match ClipboardHelper::copy_image(&rgba_img) {
                Ok(_) => {
                    self.show_toast("✅ Картинка скопирована в буфер (PNG)!".to_string());
                }
                Err(e) => {
                    self.show_toast(format!("Ошибка копирования картинки: {}", e));
                }
            },
            Err(e) => {
                self.show_toast(format!("Ошибка рендеринга: {}", e));
            }
        }
    }

    /// Copy as Markdown action
    pub fn copy_markdown_to_clipboard(&mut self) {
        if let Some(frame) = &self.current_ascii_frame {
            let md = frame.to_markdown();
            if ClipboardHelper::copy_text(&md).is_ok() {
                self.show_toast("✅ Markdown код скопирован в буфер обмена!".to_string());
            }
        }
    }

    /// Paste image from clipboard
    pub fn paste_from_clipboard(&mut self, ctx: &egui::Context) {
        match ClipboardHelper::read_image() {
            Ok(Some(img)) => {
                self.load_dynamic_image(img, Some("Вставка из буфера обмена".to_string()), ctx);
                self.show_toast("✅ Изображение вставлено из буфера обмена!".to_string());
            }
            Ok(None) => {
                self.show_toast("В буфере обмена нет изображения".to_string());
            }
            Err(e) => {
                self.show_toast(format!("Ошибка чтения буфера: {}", e));
            }
        }
    }

    /// Saves ASCII art to file with dialog
    pub fn save_file_dialog(&mut self) {
        let frame = match &self.current_ascii_frame {
            Some(f) => f,
            None => {
                self.show_toast("Нет данных для сохранения!".to_string());
                return;
            }
        };

        let mut opts = self.render_options.clone();
        opts.color_mode = self.config.color_mode;

        let dialog = rfd::FileDialog::new()
            .set_file_name("ascii_art")
            .add_filter("PNG Изображение (*.png)", &["png"])
            .add_filter("Текстовый файл (*.txt)", &["txt"])
            .add_filter("HTML веб-страница (*.html)", &["html", "htm"])
            .add_filter("SVG Векторная графика (*.svg)", &["svg"])
            .add_filter("JPEG Изображение (*.jpg)", &["jpg", "jpeg"]);

        if let Some(path) = dialog.save_file() {
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("png").to_lowercase();
            let res = match ext.as_str() {
                "txt" => std::fs::write(&path, self.cached_text.as_bytes())
                    .map_err(|e| e.to_string()),
                "html" | "htm" => {
                    let html = frame.to_html(opts.background, self.config.color_mode);
                    std::fs::write(&path, html.as_bytes()).map_err(|e| e.to_string())
                }
                "svg" => {
                    let svg = frame.to_svg(opts.background, &self.font_info.name);
                    std::fs::write(&path, svg.as_bytes()).map_err(|e| e.to_string())
                }
                "jpg" | "jpeg" => {
                    match ImageRenderer::render_to_jpeg(frame, &self.font_info.font_bytes, &opts) {
                        Ok(bytes) => std::fs::write(&path, bytes).map_err(|e| e.to_string()),
                        Err(e) => Err(e),
                    }
                }
                _ => {
                    // Default to PNG
                    match ImageRenderer::render_to_png(frame, &self.font_info.font_bytes, &opts) {
                        Ok(bytes) => std::fs::write(&path, bytes).map_err(|e| e.to_string()),
                        Err(e) => Err(e),
                    }
                }
            };

            match res {
                Ok(_) => {
                    self.show_toast(format!("Файл успешно сохранён: {:?}", path.file_name().unwrap_or_default()));
                }
                Err(e) => {
                    self.show_toast(format!("Ошибка сохранения: {}", e));
                }
            }
        }
    }

    /// Builds egui LayoutJob for high-performance rendering of the ASCII frame
    pub fn build_layout_job(&self) -> Option<LayoutJob> {
        let frame = self.current_ascii_frame.as_ref()?;
        if frame.width == 0 || frame.height == 0 {
            return None;
        }

        let font_id = FontId::monospace(self.font_size * self.zoom_level);
        let mut job = LayoutJob::default();
        job.wrap.max_width = f32::INFINITY;
        job.break_on_newline = true;

        match self.config.color_mode {
            ColorMode::Monochrome => {
                let color = Color32::from_rgb(230, 230, 230);
                let format = TextFormat::simple(font_id, color);
                job.append(&self.cached_text, 0.0, format);
            }
            ColorMode::GreenPhosphor => {
                let color = Color32::from_rgb(51, 255, 51);
                let format = TextFormat::simple(font_id, color);
                job.append(&self.cached_text, 0.0, format);
            }
            ColorMode::AmberCrt => {
                let color = Color32::from_rgb(255, 176, 0);
                let format = TextFormat::simple(font_id, color);
                job.append(&self.cached_text, 0.0, format);
            }
            ColorMode::Cyberpunk => {
                for y in 0..frame.height {
                    let mut row_str = String::with_capacity(frame.width + 1);
                    for x in 0..frame.width {
                        let idx = y * frame.width + x;
                        let ch = frame.characters.get(idx).copied().unwrap_or(' ');
                        row_str.push(ch);
                    }
                    row_str.push('\n');
                    let color = if y % 2 == 0 {
                        Color32::from_rgb(0, 255, 255)
                    } else {
                        Color32::from_rgb(255, 0, 128)
                    };
                    job.append(&row_str, 0.0, TextFormat::simple(font_id.clone(), color));
                }
            }
            ColorMode::TrueColor => {
                // Group contiguous characters by matching color
                for y in 0..frame.height {
                    let mut current_color = Color32::WHITE;
                    let mut run = String::new();

                    for x in 0..frame.width {
                        let idx = y * frame.width + x;
                        let ch = frame.characters.get(idx).copied().unwrap_or(' ');
                        let rgb = frame.colors.get(idx).copied().unwrap_or(Rgb::WHITE);
                        let char_color = Color32::from_rgb(rgb.r, rgb.g, rgb.b);

                        if run.is_empty() {
                            current_color = char_color;
                            run.push(ch);
                        } else if char_color == current_color {
                            run.push(ch);
                        } else {
                            job.append(&run, 0.0, TextFormat::simple(font_id.clone(), current_color));
                            run.clear();
                            current_color = char_color;
                            run.push(ch);
                        }
                    }

                    if !run.is_empty() {
                        job.append(&run, 0.0, TextFormat::simple(font_id.clone(), current_color));
                    }
                    job.append("\n", 0.0, TextFormat::simple(font_id.clone(), Color32::TRANSPARENT));
                }
            }
        }

        Some(job)
    }
}

impl eframe::App for AsciiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Handle Drag & Drop of image files
        ctx.input(|i| {
            if let Some(file) = i.raw.dropped_files.first() {
                let path = file.path();
                if path.is_file() {
                    self.load_file_from_path(path, &ctx);
                } else if let Ok(bytes) = file.bytes()
                    && let Ok(img) = image::load_from_memory(&bytes)
                {
                    self.load_dynamic_image(img, Some("Перетащенный файл".to_string()), &ctx);
                    self.show_toast("Файл загружен!".to_string());
                }
            }

            // Keyboard shortcuts
            if i.modifiers.command || i.modifiers.ctrl {
                if i.key_pressed(Key::V) {
                    self.paste_from_clipboard(&ctx);
                }
                if i.key_pressed(Key::C) {
                    self.copy_text_to_clipboard();
                }
                if i.key_pressed(Key::O)
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter("Изображения", &["png", "jpg", "jpeg", "webp", "gif", "bmp"])
                        .pick_file()
                {
                    self.load_file_from_path(&path, &ctx);
                }
                if i.key_pressed(Key::S) {
                    self.save_file_dialog();
                }
                if i.key_pressed(Key::Plus) || i.key_pressed(Key::Equals) {
                    self.zoom_level = (self.zoom_level * 1.15).min(4.0);
                }
                if i.key_pressed(Key::Minus) {
                    self.zoom_level = (self.zoom_level / 1.15).max(0.3);
                }
                if i.key_pressed(Key::Num0) {
                    self.zoom_level = 1.0;
                }
            }

            if i.key_pressed(Key::Space) && self.gif_animation.is_some() {
                self.is_playing = !self.is_playing;
            }
        });

        // Animated GIF timer
        if let Some(gif) = &self.gif_animation
            && self.is_playing
            && !gif.frames.is_empty()
        {
            let duration = gif.frames.get(self.current_frame_idx).map(|f| f.1).unwrap_or(Duration::from_millis(100));
            if self.last_frame_time.elapsed() >= duration {
                self.current_frame_idx = (self.current_frame_idx + 1) % gif.frames.len();
                self.last_frame_time = Instant::now();
                if let Some(frame) = gif.ascii_frames.get(self.current_frame_idx) {
                    self.cached_text = frame.to_text();
                    self.current_ascii_frame = Some(frame.clone());
                }
                ctx.request_repaint();
            } else {
                ctx.request_repaint_after(duration.saturating_sub(self.last_frame_time.elapsed()));
            }
        }

        // Recompute if dirty
        if self.dirty {
            self.recompute();
        }

        // Top Toolbar
        egui::Panel::top("top_toolbar").show(ui, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading(RichText::new("⚡ ASCII Studio").size(20.0).strong().color(Color32::from_rgb(100, 200, 255)));

                ui.separator();

                // Open file
                if ui.button(RichText::new("📂 Открыть").size(14.0)).on_hover_text("Открыть картинку или GIF (Ctrl+O)").clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter("Изображения", &["png", "jpg", "jpeg", "webp", "gif", "bmp"])
                        .pick_file()
                {
                    self.load_file_from_path(&path, &ctx);
                }

                // Paste from clipboard
                if ui.button(RichText::new("📋 Вставить").size(14.0)).on_hover_text("Вставить картинку из буфера обмена (Ctrl+V)").clicked() {
                    self.paste_from_clipboard(&ctx);
                }

                // Load sample
                if ui.button(RichText::new("🧪 Пример").size(14.0)).on_hover_text("Загрузить встроенное тестовое изображение").clicked() {
                    let sample = generate_sample_image();
                    self.load_dynamic_image(sample, Some("Пример (Компьютер CRT)".to_string()), &ctx);
                    self.show_toast("Загружен пример изображения!".to_string());
                }

                ui.separator();

                // PROMINENT COPY BUTTON
                let copy_btn = ui.add(
                    egui::Button::new(
                        RichText::new("📋 СКОПИРОВАТЬ АРТ")
                            .size(15.0)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(Color32::from_rgb(30, 140, 230))
                    .min_size(vec2(180.0, 30.0)),
                );
                if copy_btn.on_hover_text("Скопировать ASCII текст в буфер обмена (Ctrl+C)").clicked() {
                    self.copy_text_to_clipboard();
                }

                // Copy as image button
                if ui.button(RichText::new("🖼️ Копировать картинку").size(14.0)).on_hover_text("Скопировать как готовую картинку PNG в буфер").clicked() {
                    self.copy_image_to_clipboard();
                }

                // Save file button
                if ui.button(RichText::new("💾 Сохранить файл...").size(14.0)).on_hover_text("Сохранить в PNG, TXT, HTML или SVG (Ctrl+S)").clicked() {
                    self.save_file_dialog();
                }

                ui.separator();

                // Zoom controls
                if ui.button("🔍-").on_hover_text("Уменьшить (Ctrl + -)").clicked() {
                    self.zoom_level = (self.zoom_level / 1.2).max(0.3);
                }
                ui.label(format!("{:.0}%", self.zoom_level * 100.0));
                if ui.button("🔍+").on_hover_text("Увеличить (Ctrl + +)").clicked() {
                    self.zoom_level = (self.zoom_level * 1.2).min(4.0);
                }
                if ui.button("1:1").on_hover_text("Сбросить масштаб").clicked() {
                    self.zoom_level = 1.0;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // System font badge
                    let font_badge = RichText::new(format!("🔤 {}", self.font_info.name))
                        .size(12.0)
                        .color(Color32::from_rgb(180, 220, 255));
                    ui.label(font_badge).on_hover_text(format!(
                        "Системный моноширинный шрифт, используемый для рендеринга и экспорта:\n{:?}",
                        self.font_info.path.as_deref().unwrap_or("Встроенный")
                    ));
                });
            });
            ui.add_space(4.0);
        });

        // Bottom Status Bar & Toast
        egui::Panel::bottom("bottom_status").show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                if let Some(frame) = &self.current_ascii_frame {
                    ui.label(RichText::new(format!(
                        "Размер: {} x {} символов (всего {} знаков)",
                        frame.width,
                        frame.height,
                        frame.characters.len()
                    )).size(12.0).color(Color32::LIGHT_GRAY));
                }

                if let Some(title) = &self.image_path {
                    ui.separator();
                    ui.label(RichText::new(format!("Файл: {}", title)).size(12.0).color(Color32::GRAY));
                }

                // Toast notification
                if let Some((msg, time)) = &self.toast
                    && time.elapsed() < Duration::from_secs(3)
                {
                    let alpha = ((3.0 - time.elapsed().as_secs_f32()) / 0.5).clamp(0.0, 1.0);
                    let bg_color = Color32::from_rgba_unmultiplied(40, 160, 80, (alpha * 255.0) as u8);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(msg).color(Color32::WHITE).background_color(bg_color).strong().size(13.0));
                    });
                    ctx.request_repaint();
                }
            });
            ui.add_space(2.0);
        });

        // Left Settings & Presets Panel
        egui::Panel::left("left_settings_panel")
            .resizable(true)
            .default_size(310.0)
            .size_range(280.0..=450.0)
            .show(ui, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(6.0);

                    // PRESETS SECTION
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("✨ Готовые пресеты").strong().size(16.0));
                    });
                    ui.add_space(2.0);

                    let mut preset_to_apply = None;
                    for preset in &self.presets {
                        let is_selected = self.selected_preset_id.as_deref() == Some(preset.id);
                        let btn_text = format!("{} {}", preset.icon, preset.name_ru);

                        let mut btn = egui::Button::new(RichText::new(btn_text).size(13.0));
                        if is_selected {
                            btn = btn.fill(Color32::from_rgb(45, 90, 140));
                        }

                        if ui.add_sized([ui.available_width(), 26.0], btn)
                            .on_hover_text(preset.description_ru)
                            .clicked()
                        {
                            preset_to_apply = Some(preset.clone());
                        }
                    }
                    if let Some(preset) = preset_to_apply {
                        self.apply_preset(&preset);
                    }

                    ui.add_space(10.0);
                    ui.separator();

                    // QUICK SLIDERS
                    ui.add_space(4.0);
                    ui.label(RichText::new("⚙️ Основные параметры").strong().size(15.0));
                    ui.add_space(4.0);

                    // Width slider
                    ui.horizontal(|ui| {
                        ui.label("Ширина (символов):");
                        let width_resp = ui.add(egui::Slider::new(&mut self.config.width, 30..=250));
                        if width_resp.changed() {
                            self.dirty = true;
                            self.selected_preset_id = None;
                        }
                    });

                    // Aspect ratio slider
                    ui.horizontal(|ui| {
                        ui.label("Пропорция символа:").on_hover_text("Компенсация высоты шрифта (для моноширинного шрифта обычно ~0.50)");
                        let aspect_resp = ui.add(egui::Slider::new(&mut self.config.aspect_ratio, 0.30..=0.80).step_by(0.01));
                        if aspect_resp.changed() {
                            self.dirty = true;
                            self.selected_preset_id = None;
                        }
                    });

                    // Color Mode
                    ui.add_space(4.0);
                    ui.label("Цветовой режим:");
                    for mode in ColorMode::all() {
                        let is_current = self.config.color_mode == *mode;
                        if ui.selectable_label(is_current, mode.label_ru()).clicked() {
                            self.config.color_mode = *mode;
                            self.dirty = true;
                            self.selected_preset_id = None;
                        }
                    }

                    // Palette Selector
                    ui.add_space(6.0);
                    ui.label("Набор символов:");
                    let palette_options = [
                        (PaletteType::Standard70, "Детальная (70 символов)"),
                        (PaletteType::Standard10, "Простая (10 символов)"),
                        (PaletteType::Blocks, "Блоки ( ░▒▓█)"),
                        (PaletteType::Braille, "Брайль 8x (Ультра-детализация)"),
                        (PaletteType::Matrix, "Матрица (0123...$#@)"),
                        (PaletteType::SobelOutline, "Контурный скетч (Собель)"),
                        (PaletteType::Minimalist, "Минимализм (.oO@)"),
                        (PaletteType::Binary, "Двоичный код (01)"),
                        (PaletteType::Custom(self.custom_palette_text.clone()), "Пользовательский"),
                    ];

                    for (p_type, label) in palette_options {
                        let is_selected = match (&self.config.palette, &p_type) {
                            (PaletteType::Custom(_), PaletteType::Custom(_)) => true,
                            (a, b) => a == b,
                        };
                        if ui.selectable_label(is_selected, label).clicked() {
                            self.config.palette = p_type;
                            self.dirty = true;
                            self.selected_preset_id = None;
                        }
                    }

                    if let PaletteType::Custom(_) = self.config.palette {
                        ui.add_space(4.0);
                        ui.label("Символы от темного к светлому:");
                        if ui.text_edit_singleline(&mut self.custom_palette_text).changed() {
                            self.config.palette = PaletteType::Custom(self.custom_palette_text.clone());
                            self.dirty = true;
                        }
                    }

                    // Sobel threshold if outline selected
                    if let PaletteType::SobelOutline = self.config.palette {
                        ui.add_space(4.0);
                        ui.label("Чувствительность контура:");
                        if ui.add(egui::Slider::new(&mut self.config.edge_threshold, 0.05..=0.80)).changed() {
                            self.dirty = true;
                        }
                    }

                    ui.add_space(10.0);
                    ui.separator();

                    // ADVANCED ACCORDION
                    ui.add_space(4.0);
                    ui.checkbox(&mut self.advanced_mode, RichText::new("🛠️ Дополнительные настройки").size(14.0));

                    if self.advanced_mode {
                        ui.add_space(4.0);

                        // Brightness
                        ui.horizontal(|ui| {
                            ui.label("Яркость:");
                            if ui.add(egui::Slider::new(&mut self.config.brightness, -0.8..=0.8).step_by(0.02)).changed() {
                                self.dirty = true;
                            }
                        });

                        // Contrast
                        ui.horizontal(|ui| {
                            ui.label("Контраст:");
                            if ui.add(egui::Slider::new(&mut self.config.contrast, -0.8..=0.8).step_by(0.02)).changed() {
                                self.dirty = true;
                            }
                        });

                        // Gamma
                        ui.horizontal(|ui| {
                            ui.label("Гамма:");
                            if ui.add(egui::Slider::new(&mut self.config.gamma, 0.2..=2.5).step_by(0.05)).changed() {
                                self.dirty = true;
                            }
                        });

                        // Invert
                        if ui.checkbox(&mut self.config.invert, "Инвертировать цвета").changed() {
                            self.dirty = true;
                        }

                        // Dithering
                        ui.add_space(4.0);
                        ui.label("Алгоритм дизеринга:");
                        for dither in DitherMode::all() {
                            let is_curr = self.config.dither == *dither;
                            if ui.selectable_label(is_curr, dither.label_ru()).clicked() {
                                self.config.dither = *dither;
                                self.dirty = true;
                            }
                        }

                        ui.add_space(6.0);
                        ui.separator();

                        // Export Image options
                        ui.label(RichText::new("🖼️ Настройки экспорта картинки:").size(13.0).strong());
                        ui.horizontal(|ui| {
                            ui.label("Размер шрифта:");
                            ui.add(egui::Slider::new(&mut self.render_options.font_size, 8.0..=32.0).step_by(1.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Отступ:");
                            ui.add(egui::Slider::new(&mut self.render_options.padding, 0..=64));
                        });
                    }

                    // GIF Controls if GIF is active
                    let total_frames = self.gif_animation.as_ref().map(|g| g.frames.len()).unwrap_or(0);
                    if total_frames > 0 {
                        ui.add_space(10.0);
                        ui.separator();
                        ui.label(RichText::new("🎬 Управление анимацией GIF").strong().size(15.0));

                        let mut step_prev = false;
                        let mut step_next = false;
                        let mut toggle_play = false;

                        ui.horizontal(|ui| {
                            let play_icon = if self.is_playing { "⏸ Пауза" } else { "▶ Воспроизведение" };
                            if ui.button(play_icon).clicked() {
                                toggle_play = true;
                            }

                            if ui.button("⏮").clicked() {
                                step_prev = true;
                            }
                            if ui.button("⏭").clicked() {
                                step_next = true;
                            }
                        });

                        if toggle_play {
                            self.is_playing = !self.is_playing;
                        }
                        if step_prev && self.current_frame_idx > 0 {
                            self.current_frame_idx -= 1;
                            self.recompute();
                        }
                        if step_next && self.current_frame_idx + 1 < total_frames {
                            self.current_frame_idx += 1;
                            self.recompute();
                        }

                        ui.horizontal(|ui| {
                            ui.label(format!("Кадр {}/{}", self.current_frame_idx + 1, total_frames));
                            if total_frames > 1 {
                                let mut frame_val = self.current_frame_idx + 1;
                                if ui.add(egui::Slider::new(&mut frame_val, 1..=total_frames)).changed() {
                                    self.current_frame_idx = frame_val - 1;
                                    self.recompute();
                                }
                            }
                        });
                    }

                    ui.add_space(20.0);
                });
            });

        // Center Viewport (Interactive ASCII Art Canvas)
        egui::CentralPanel::default().show(ui, |ui| {
            // Viewport controls header
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.show_original, false, "🖥️ ASCII Арт");
                ui.selectable_value(&mut self.show_original, true, "🖼️ Оригинал");

                ui.separator();

                if ui.button("📋 Скопировать текст").clicked() {
                    self.copy_text_to_clipboard();
                }
                if ui.button("📄 Markdown").clicked() {
                    self.copy_markdown_to_clipboard();
                }
                if ui.button("🖼️ Скопировать как PNG").clicked() {
                    self.copy_image_to_clipboard();
                }
            });

            ui.separator();

            if self.show_original {
                // Show original image texture
                if let Some(tex) = &self.original_texture {
                    ScrollArea::both().show(ui, |ui| {
                        let size = tex.size_vec2() * self.zoom_level;
                        ui.image((tex.id(), size));
                    });
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("Изображение не загружено");
                    });
                }
            } else {
                // Render ASCII art
                if let Some(job) = self.build_layout_job() {
                    ScrollArea::both()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.label(job);
                        });
                } else {
                    // Empty state drop zone
                    ui.centered_and_justified(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(40.0);
                            ui.label(RichText::new("📂").size(48.0));
                            ui.heading("Перетащите изображение сюда");
                            ui.label("Поддерживаются форматы PNG, JPG, WEBP, GIF, BMP");
                            ui.label(RichText::new("или нажмите Ctrl+V для вставки из буфера обмена").color(Color32::LIGHT_BLUE));
                            ui.add_space(10.0);

                            if ui.button(RichText::new("Загрузить пример изображения").size(16.0)).clicked() {
                                let sample = generate_sample_image();
                                self.load_dynamic_image(sample, Some("Пример".to_string()), &ctx);
                            }
                        });
                    });
                }
            }
        });
    }
}
