use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Config {
    pub editor: String,
    pub terminal: Option<String>,
    pub keybindings: Keybindings,
    pub theme: Theme,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(default)]
pub struct Keybindings {
    // Example: "Ctrl+Q" -> "quit"
    pub global: HashMap<String, String>,
    // You can add other contexts like "editor", "file_view"
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(default)]
pub struct Theme {
    pub primary_bg: String,
    pub secondary_bg: String,
    pub text_fg: String,
    pub highlight_fg: String,
    pub highlight_bg: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string()),
            terminal: None, // None means use default shell
            keybindings: Keybindings::default(),
            theme: Theme::default(),
        }
    }
}

impl Default for Keybindings {
    fn default() -> Self {
        let mut global = HashMap::new();
        global.insert("Ctrl-Q".to_string(), "quit".to_string());
        global.insert("Ctrl-B".to_string(), "toggle_primary_sidebar".to_string());
        global.insert("Ctrl-J".to_string(), "toggle_panel".to_string());
        global.insert("Ctrl-K".to_string(), "cycle_focus".to_string());
        global.insert("Ctrl-P".to_string(), "toggle_command_palette".to_string());
        global.insert("Ctrl-N".to_string(), "new_tab".to_string());
        global.insert("Ctrl-W".to_string(), "close_tab".to_string());
        global.insert("Alt-H".to_string(), "prev_tab".to_string());
        global.insert("Alt-L".to_string(), "next_tab".to_string());
        Self { global }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary_bg: "Black".to_string(),
            secondary_bg: "#222222".to_string(),
            text_fg: "White".to_string(),
            highlight_fg: "Yellow".to_string(),
            highlight_bg: "Blue".to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = Self::get_config_path()?;
        if path.exists() {
            let content = fs::read_to_string(path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Config::default())
        }
    }

    fn get_config_path() -> Result<PathBuf> {
        let proj_dirs = directories_next::ProjectDirs::from("com", "inf-edit", "inf-edit")
            .ok_or_else(|| anyhow!("Could not find a valid home directory"))?;
        let config_dir = proj_dirs.config_dir();
        fs::create_dir_all(config_dir)?;
        Ok(config_dir.join("settings.toml"))
    }
}