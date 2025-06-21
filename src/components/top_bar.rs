use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Clear},
};
pub mod command_palette; // Declare the sub-module
use self::command_palette::CommandPalette; // Import from its own sub-module

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

    pub fn render(&mut self, f: &mut Frame, area: Rect, _is_active: bool, title: &str, command_palette: &mut CommandPalette, show_command_palette: bool) {
        let top_bar_block = Block::default()
            .borders(Borders::BOTTOM)
            .style(Style::default().fg(Color::White));

        if show_command_palette {
            let popup_width = 80;
            let popup_height = 3;
            let popup_area = Rect::new(
                area.x + (area.width.saturating_sub(popup_width)) / 2,
                area.y,
                popup_width,
                popup_height,
            );
            f.render_widget(Clear, popup_area);
            command_palette.render(f, popup_area);
        } else {
            let text = Paragraph::new(title).block(top_bar_block);
            f.render_widget(text, area);
        }
    }
}