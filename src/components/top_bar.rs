use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
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

    pub fn render(&self, f: &mut Frame, area: Rect, _is_active: bool, title: &str) {
        let top_bar_block = Block::default()
            .borders(Borders::BOTTOM)
            .style(Style::default().fg(Color::White));

        let text = Paragraph::new(title).block(top_bar_block);
        f.render_widget(text, area);
    }
}