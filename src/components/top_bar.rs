pub mod command_palette;

use crate::app::App;
use ratatui::{
    prelude::*,
    style::{Modifier, Style},
    widgets::{Block, Tabs, Widget},
};

pub struct TopBar {}

impl TopBar {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_tabs_widget<'a>(&self, app: &'a App) -> impl Widget + 'a {
        let titles: Vec<String> = app.main_tabs.iter().map(|t| t.title.clone()).collect();
        Tabs::new(titles)
            .block(Block::default().bg(app.theme.primary_bg))
            .select(app.active_main_tab)
            .style(Style::default().fg(app.theme.text_fg))
            .highlight_style(
                Style::default().fg(app.theme.highlight_fg).add_modifier(Modifier::BOLD),
            )
    }
}