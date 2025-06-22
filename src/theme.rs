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
        match config_theme.preset.to_lowercase().as_str() {
            "default-dark" => Self {
                primary_bg: parse_color("black").unwrap(),
                secondary_bg: parse_color("#222222").unwrap(),
                text_fg: parse_color("white").unwrap(),
                highlight_fg: parse_color("yellow").unwrap(),
                highlight_bg: parse_color("blue").unwrap(),
            },
            "default-light" => Self {
                primary_bg: parse_color("white").unwrap(),
                secondary_bg: parse_color("#eeeeee").unwrap(),
                text_fg: parse_color("black").unwrap(),
                highlight_fg: parse_color("blue").unwrap(),
                highlight_bg: parse_color("yellow").unwrap(),
            },
            "atom-dark" => Self {
                primary_bg: parse_color("#282c34").unwrap(),
                secondary_bg: parse_color("#21252b").unwrap(),
                text_fg: parse_color("#abb2bf").unwrap(),
                highlight_fg: parse_color("#61afef").unwrap(),
                highlight_bg: parse_color("#3a3f4b").unwrap(),
            },
            "dracula" => Self {
                primary_bg: parse_color("#282a36").unwrap(),
                secondary_bg: parse_color("#44475a").unwrap(),
                text_fg: parse_color("#f8f8f2").unwrap(),
                highlight_fg: parse_color("#50fa7b").unwrap(),
                highlight_bg: parse_color("#44475a").unwrap(),
            },
            "nord" => Self {
                primary_bg: parse_color("#2E3440").unwrap(),
                secondary_bg: parse_color("#3B4252").unwrap(),
                text_fg: parse_color("#D8DEE9").unwrap(),
                highlight_fg: parse_color("#88C0D0").unwrap(),
                highlight_bg: parse_color("#4C566A").unwrap(),
            },
            "solarized_dark" => Self {
                primary_bg: parse_color("#002b36").unwrap(),
                secondary_bg: parse_color("#073642").unwrap(),
                text_fg: parse_color("#839496").unwrap(),
                highlight_fg: parse_color("#268bd2").unwrap(),
                highlight_bg: parse_color("#073642").unwrap(),
            },
            "tokyo-night-blue" => Self {
                primary_bg: parse_color("#1a1b26").unwrap(),
                secondary_bg: parse_color("#24283b").unwrap(),
                text_fg: parse_color("#c0caf5").unwrap(),
                highlight_fg: parse_color("#7aa2f7").unwrap(),
                highlight_bg: parse_color("#414868").unwrap(),
            },
            // "Custom" or any other value falls back to parsing individual colors
            _ => {
                let default_theme = settings::Theme::default();
                Self {
                    primary_bg: parse_color(&config_theme.primary_bg)
                        .unwrap_or_else(|_| parse_color(&default_theme.primary_bg).unwrap_or(Color::Black)),
                    secondary_bg: parse_color(&config_theme.secondary_bg)
                        .unwrap_or_else(|_| parse_color(&default_theme.secondary_bg).unwrap_or(Color::DarkGray)),
                    text_fg: parse_color(&config_theme.text_fg)
                        .unwrap_or_else(|_| parse_color(&default_theme.text_fg).unwrap_or(Color::White)),
                    highlight_fg: parse_color(&config_theme.highlight_fg)
                        .unwrap_or_else(|_| parse_color(&default_theme.highlight_fg).unwrap_or(Color::Yellow)),
                    highlight_bg: parse_color(&config_theme.highlight_bg)
                        .unwrap_or_else(|_| parse_color(&default_theme.highlight_bg).unwrap_or(Color::Blue)),
                }
            }
        }
    }
}