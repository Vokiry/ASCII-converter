use std::path::Path;
use std::process::Command;
use std::sync::Arc;

pub const EMBEDDED_FALLBACK_FONT: &[u8] = include_bytes!("../assets/fallback_font.ttf");

#[derive(Clone, Debug)]
pub struct SystemFontInfo {
    pub name: String,
    pub path: Option<String>,
    pub font_bytes: Vec<u8>,
}

impl SystemFontInfo {
    pub fn fallback() -> Self {
        Self {
            name: "Noto Sans Mono (Встроенный)".to_string(),
            path: None,
            font_bytes: EMBEDDED_FALLBACK_FONT.to_vec(),
        }
    }
}

/// Detects the system monospace font across Linux, Windows, and macOS.
pub fn detect_system_monospace_font() -> SystemFontInfo {
    #[cfg(target_os = "linux")]
    {
        // 1. Try fc-match for exact configured monospace font
        if let Ok(output) = Command::new("fc-match").args(["-f", "%{file}|%{family}", "monospace"]).output()
            && output.status.success()
        {
            let out_str = String::from_utf8_lossy(&output.stdout);
            let parts: Vec<&str> = out_str.trim().split('|').collect();
            if let Some(file_path) = parts.first() {
                let path = Path::new(file_path);
                if path.exists()
                    && let Ok(bytes) = std::fs::read(path)
                {
                    let family = parts.get(1).copied().unwrap_or("Monospace");
                    return SystemFontInfo {
                        name: format!("{} (Системный)", family),
                        path: Some(file_path.to_string()),
                        font_bytes: bytes,
                    };
                }
            }
        }

        // 2. Try common Linux font paths
        let candidate_paths = [
            "/usr/share/fonts/noto/NotoSansMono-Regular.ttf",
            "/usr/share/fonts/TTF/JetBrainsMonoNerdFontMono-Regular.ttf",
            "/usr/share/fonts/TTF/DejaVuSansMono.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf",
            "/usr/share/fonts/liberation-mono/LiberationMono-Regular.ttf",
        ];

        for &path_str in &candidate_paths {
            let path = Path::new(path_str);
            if path.exists()
                && let Ok(bytes) = std::fs::read(path)
            {
                let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("System Mono");
                return SystemFontInfo {
                    name: format!("{} (Системный)", file_name),
                    path: Some(path_str.to_string()),
                    font_bytes: bytes,
                };
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
        let fonts_dir = Path::new(&windir).join("Fonts");

        let candidates = [
            ("Consolas", "consola.ttf"),
            ("Cascadia Mono", "CascadiaMono.ttf"),
            ("Courier New", "cour.ttf"),
            ("Lucida Console", "lucon.ttf"),
        ];

        for (name, filename) in candidates {
            let p = fonts_dir.join(filename);
            if p.exists() {
                if let Ok(bytes) = std::fs::read(&p) {
                    return SystemFontInfo {
                        name: format!("{} (Системный Windows)", name),
                        path: Some(p.to_string_lossy().to_string()),
                        font_bytes: bytes,
                    };
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let candidates = [
            ("Menlo", "/System/Library/Fonts/Menlo.ttc"),
            ("SF NS Mono", "/System/Library/Fonts/SFNSMono.ttf"),
            ("Monaco", "/System/Library/Fonts/Monaco.ttf"),
            ("Courier New", "/Library/Fonts/Courier New.ttf"),
        ];

        for (name, path_str) in candidates {
            let p = Path::new(path_str);
            if p.exists() {
                if let Ok(bytes) = std::fs::read(p) {
                    return SystemFontInfo {
                        name: format!("{} (Системный macOS)", name),
                        path: Some(path_str.to_string()),
                        font_bytes: bytes,
                    };
                }
            }
        }
    }

    // Fallback to embedded clean font
    SystemFontInfo::fallback()
}

/// Injects the system font into egui font definitions with highest priority for Monospace
pub fn configure_egui_fonts(ctx: &egui::Context, font_info: &SystemFontInfo) {
    let mut fonts = egui::FontDefinitions::default();

    // Register font data under "system_mono"
    fonts.font_data.insert(
        "system_mono".to_string(),
        Arc::new(egui::FontData::from_owned(font_info.font_bytes.clone())),
    );

    // Set as highest priority for Monospace family
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "system_mono".to_string());

    ctx.set_fonts(fonts);
}
