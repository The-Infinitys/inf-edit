use crate::settings;
use anyhow::{anyhow, Result};
use ratatui::style::Color;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Theme {
    pub primary_bg: Color,
    pub secondary_bg: Color,
    pub text_fg: Color,
    pub highlight_fg: Color,
    pub highlight_bg: Color,
}

fn parse_color(s: &str) -> Result<Color> {
    if s.starts_with('#') {
        let hex_part = s.trim_start_matches('#');
        if hex_part.len() == 6 {
            let r = u8::from_str_radix(&hex_part[0..2], 16)?;
            let g = u8::from_str_radix(&hex_part[2..4], 16)?;
            let b = u8::from_str_radix(&hex_part[4..6], 16)?;
            Ok(Color::Rgb(r, g, b))
        } else {
            Err(anyhow!("Invalid hex color format: {}", s))
        }
    } else {
        Color::from_str(s).map_err(|e| anyhow!("Invalid color name: {}, error: {}", s, e))
    }
}

impl Theme {
    pub fn from_config(config_theme: &settings::Theme) -> Self {
        let default_theme = settings::Theme::default();
        Self {
            primary_bg: parse_color(&config_theme.primary_bg).unwrap_or_else(|_| parse_color(&default_theme.primary_bg).unwrap_or(Color::Black)),
            secondary_bg: parse_color(&config_theme.secondary_bg).unwrap_or_else(|_| parse_color(&default_theme.secondary_bg).unwrap_or(Color::DarkGray)),
            text_fg: parse_color(&config_theme.text_fg).unwrap_or_else(|_| parse_color(&default_theme.text_fg).unwrap_or(Color::White)),
            highlight_fg: parse_color(&config_theme.highlight_fg).unwrap_or_else(|_| parse_color(&default_theme.highlight_fg).unwrap_or(Color::Yellow)),
            highlight_bg: parse_color(&config_theme.highlight_bg).unwrap_or_else(|_| parse_color(&default_theme.highlight_bg).unwrap_or(Color::Blue)),
        }
    }
}