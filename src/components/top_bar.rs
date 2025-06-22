use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
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

    pub fn render(&mut self, f: &mut Frame, area: Rect, _is_active: bool, title: &str, command_palette: &mut CommandPalette, show_command_palette: bool, theme: &crate::theme::Theme) {
        let top_bar_block = Block::default()
            .borders(Borders::NONE)
            .style(Style::default().fg(Color::White));

        if show_command_palette {
            // The command palette is rendered as an overlay, so it needs the full frame area
            // to calculate its centered position, not just the top_bar_area.
            // We'll pass the theme to its render method.
            // The actual rendering of the command palette as an overlay is handled in ui.rs
            // This method only sets up the top bar's own content.
            // The `Clear` widget should be used in `ui.rs` before rendering the palette.
        } else {
            let text = Paragraph::new(title).block(top_bar_block);
            f.render_widget(text, area);
        }
    }
}