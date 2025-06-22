use crate::app::App;
use ratatui::{
    prelude::*,
    widgets::{Paragraph, Widget},
};

pub struct BottomBar {}

impl Default for BottomBar {
    fn default() -> Self {
        Self::new()
    }
}

impl BottomBar {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_status_widget<'a>(&self, app: &'a App) -> impl Widget + 'a {
        let status_text = format!("Active: {:?}", app.active_target);
        Paragraph::new(status_text)
            .style(
                Style::default()
                    .bg(app.theme.secondary_bg)
                    .fg(app.theme.text_fg),
            )
            .alignment(Alignment::Left)
    }
}
