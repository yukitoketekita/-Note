use std::{
    fs,
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    /// User-configured notes directory. `None` means use the default.
    pub notes_dir: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self { notes_dir: None }
    }
}

/// Returns the config directory for this app.
/// Linux:   ~/.config/ipad/
/// Windows: C:\Users\<user>\AppData\Roaming\ipad\
/// macOS:   ~/Library/Application Support/ipad/
fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ipad")
}

/// Returns the path to config.json.
fn config_file_path() -> PathBuf {
    config_dir().join("config.json")
}

/// Reads the current config. Returns defaults when the file doesn't exist.
pub fn read_config() -> AppConfig {
    let path = config_file_path();

    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => AppConfig::default(),
    }
}

/// Writes config to disk. Creates the config directory if needed.
pub fn write_config(config: &AppConfig) -> Result<(), String> {
    let dir = config_dir();
    fs::create_dir_all(&dir)
        .map_err(|error| format!("Failed to create config directory: {error}"))?;

    let json = serde_json::to_string_pretty(config)
        .map_err(|error| format!("Failed to serialize config: {error}"))?;

    let path = config_file_path();
    fs::write(&path, json)
        .map_err(|error| format!("Failed to write config file: {error}"))?;

    Ok(())
}

/// Returns the effective notes directory:
/// 1. User-configured path (from config.json) if set
/// 2. XDG Documents directory + "Notes" (cross-platform fallback)
/// 3. Current directory + "Notes" (last resort)
pub fn notes_dir_path() -> Result<PathBuf, String> {
    let config = read_config();

    if let Some(custom_dir) = config.notes_dir {
        let path = PathBuf::from(&custom_dir);
        if path.is_absolute() {
            return Ok(path);
        }
    }

    // Fallback 1: use the system Documents directory
    if let Some(docs) = dirs::document_dir() {
        return Ok(docs.join("Notes"));
    }

    // Fallback 2: current directory
    let current_dir = std::env::current_dir()
        .map_err(|error| format!("Failed to read current directory: {error}"))?;

    Ok(current_dir.join("Notes"))
}

/// Ensures the notes directory exists on disk.
pub fn ensure_notes_dir() -> Result<PathBuf, String> {
    let notes_dir = notes_dir_path()?;
    fs::create_dir_all(&notes_dir)
        .map_err(|error| format!("Failed to create Notes directory: {error}"))?;
    Ok(notes_dir)
}
