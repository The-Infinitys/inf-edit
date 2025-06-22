pub mod command_palette;

use crate::app::App;
use ratatui::{
    prelude::*,
    style::Style,
    widgets::{Paragraph, Widget},
};

pub struct TopBar {}

impl Default for TopBar {
    fn default() -> Self {
        Self::new()
    }
}

impl TopBar {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_title_widget<'a>(&self, app: &'a App) -> impl Widget + 'a {
        Paragraph::new("inf-edit")
            .style(Style::default().bg(app.theme.secondary_bg).fg(app.theme.text_fg))
            .alignment(Alignment::Center)
    }
}