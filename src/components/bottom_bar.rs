use crate::app::App;
use ratatui::{
    prelude::*,
    widgets::{Paragraph, Widget},
};

pub struct BottomBar {}

impl BottomBar {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_status_widget<'a>(&self, app: &'a App) -> impl Widget + 'a {
        let mut status_parts = vec![];
        status_parts.push(format!("Active: {:?}", app.active_target));

        let size_state_str = match app.active_target {
            crate::ActiveTarget::PrimarySideBar => format!(" | Size: {:?}", app.sidebar_width_state),
            crate::ActiveTarget::Panel => format!(" | Size: {:?}", app.panel_height_state),
            _ => "".to_string(),
        };
        status_parts.push(size_state_str);

        let status_text = status_parts.join("");
        Paragraph::new(status_text)
            .style(Style::default().bg(app.theme.secondary_bg).fg(app.theme.text_fg))
            .alignment(Alignment::Left)
    }
}