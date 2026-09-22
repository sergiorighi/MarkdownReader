use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tao::dpi::PhysicalPosition;
use tao::window::Window;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
    #[serde(default = "default_font_family")]
    pub font_family: String,
    #[serde(default = "default_code_font_family")]
    pub code_font_family: String,
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    #[serde(default = "default_content_width")]
    pub content_width: u32,
    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_font_family() -> String {
    "Inter, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif".to_string()
}

fn default_code_font_family() -> String {
    "'Roboto Mono', 'Cascadia Code', 'Cascadia Mono', Consolas, 'Courier New', monospace".to_string()
}

fn default_font_size() -> f32 {
    16.0
}

fn default_content_width() -> u32 {
    960
}

fn default_theme() -> String {
    "dark".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            x: None,
            y: None,
            width: 1024,
            height: 768,
            maximized: false,
            font_family: default_font_family(),
            code_font_family: default_code_font_family(),
            font_size: default_font_size(),
            content_width: default_content_width(),
            theme: default_theme(),
        }
    }
}

impl Config {
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("MarkdownReader").join("config.json"))
    }

    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(config) = serde_json::from_str::<Config>(&content) {
                        return config;
                    }
                }
            }
        }
        Config::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let json = serde_json::to_string_pretty(self)?;
            fs::write(path, json)?;
        }
        Ok(())
    }

    pub fn update_from_window(&mut self, window: &Window) {
        self.maximized = window.is_maximized();
        if !self.maximized {
            if let Ok(pos) = window.outer_position() {
                self.x = Some(pos.x);
                self.y = Some(pos.y);
            }
            let size = window.inner_size();
            let scale_factor = window.scale_factor();
            let logical_size = size.to_logical::<f64>(scale_factor);
            self.width = (logical_size.width as u32).max(400);
            self.height = (logical_size.height as u32).max(300);
        }
    }

    pub fn is_position_valid(&self, window: &Window) -> bool {
        if let (Some(x), Some(y)) = (self.x, self.y) {
            let target_pos = PhysicalPosition::new(x, y);
            for monitor in window.available_monitors() {
                let mon_pos = monitor.position();
                let mon_size = monitor.size();
                // Check if target position is within monitor bounds (with a safety margin)
                if target_pos.x >= mon_pos.x - 100
                    && target_pos.x < mon_pos.x + mon_size.width as i32 - 100
                    && target_pos.y >= mon_pos.y - 100
                    && target_pos.y < mon_pos.y + mon_size.height as i32 - 100
                {
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults_and_serialization() {
        let config = Config::default();
        assert_eq!(config.width, 1024);
        assert_eq!(config.height, 768);
        assert_eq!(config.maximized, false);
        assert_eq!(config.font_size, 16.0);
        assert_eq!(config.content_width, 960);
        assert_eq!(config.theme, "dark");

        let json = serde_json::to_string(&config).expect("Serialize error");
        let parsed: Config = serde_json::from_str(&json).expect("Deserialize error");
        assert_eq!(parsed.width, 1024);
        assert_eq!(parsed.height, 768);
    }
}

