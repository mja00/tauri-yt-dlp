use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

const CONFIG_FILE_NAME: &str = "config.json";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub download_location: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            download_location: None,
        }
    }
}

fn get_config_path() -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| "Failed to get config directory".to_string())?
        .join("mac-ytdlp");
    
    // Create config directory if it doesn't exist
    fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;
    
    Ok(config_dir.join(CONFIG_FILE_NAME))
}

pub fn load_config() -> AppConfig {
    let config_path = match get_config_path() {
        Ok(path) => path,
        Err(_) => return AppConfig::default(),
    };

    if !config_path.exists() {
        return AppConfig::default();
    }

    match fs::read_to_string(&config_path) {
        Ok(content) => {
            serde_json::from_str(&content).unwrap_or_else(|_| AppConfig::default())
        }
        Err(_) => AppConfig::default(),
    }
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let config_path = get_config_path()?;
    
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;
    
    Ok(())
}

pub fn get_download_path() -> Result<PathBuf, String> {
    let config = load_config();
    
    if let Some(location) = config.download_location {
        let path = PathBuf::from(&location);
        if path.exists() && path.is_dir() {
            return Ok(path);
        }
    }
    
    // Fallback to default Downloads directory
    dirs::download_dir()
        .ok_or_else(|| "Failed to get Downloads directory".to_string())
}

