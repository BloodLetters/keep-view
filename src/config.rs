use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub width: f64,
    pub height: f64,
    pub always_on_top: bool,
    pub compact_mode: bool,
    pub zoom_level: f64,
    pub minimize_to_tray: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            x: None,
            y: None,
            width: 440.0,
            height: 700.0,
            always_on_top: false,
            compact_mode: true,
            zoom_level: 1.0,
            minimize_to_tray: true,
        }
    }
}

impl AppConfig {
    pub fn load(config_path: &PathBuf) -> Self {
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(config_path) {
                if let Ok(cfg) = serde_json::from_str(&content) {
                    return cfg;
                }
            }
        }
        let default_cfg = Self::default();
        let _ = default_cfg.save(config_path);
        default_cfg
    }

    pub fn save(&self, config_path: &PathBuf) -> std::io::Result<()> {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(config_path, content)?;
        Ok(())
    }
}
