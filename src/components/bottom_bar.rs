use crate::{theme::Theme, ActiveTarget};
use ratatui::{
    prelude::*,
    widgets::{Paragraph, Widget},
};

pub struct BottomBar {}

impl BottomBar {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_status_widget<'a>(&self, active_target: ActiveTarget, theme: &'a Theme) -> impl Widget + 'a {
        let status_text = format!("Active: {:?}", active_target);
        Paragraph::new(status_text)
            .style(Style::default().bg(theme.secondary_bg).fg(theme.text_fg))
            .alignment(Alignment::Left)
    }
}