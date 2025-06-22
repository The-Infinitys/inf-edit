use ratatui::{
    layout::Rect,
    prelude::*,
    text::Line,
    widgets::{Block, Borders, Paragraph}, // Removed unused KeyModifiers
};
use crossterm::event::{KeyCode, KeyEvent};
use crate::theme::Theme;

#[derive(Default)]
pub struct SearchWidget {
    pub query: String,
    pub cursor_position: usize,
}

impl SearchWidget {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c) => {
                self.query.insert(self.cursor_position, c);
                self.cursor_position += 1;
                true
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.query.remove(self.cursor_position);
                }
                true
            }
            KeyCode::Left => {
                self.cursor_position = self.cursor_position.saturating_sub(1);
                true
            }
            KeyCode::Right => {
                self.cursor_position = (self.cursor_position + 1).min(self.query.len());
                true
            }
            KeyCode::Enter => {
                // For now, just clear the query on Enter.
                // Actual search logic would go here.
                self.query.clear();
                self.cursor_position = 0;
                true
            }
            _ => false, // Key not handled
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, is_active: bool, theme: &Theme) {
        let mut search_block = Block::default()
            .title("Search")
            .borders(Borders::ALL);
        if is_active {
            search_block = search_block.style(Style::default().fg(theme.highlight_fg));
        }
        let input_line = Line::from(self.query.as_str());
        let input_paragraph = Paragraph::new(input_line).block(search_block);
        f.render_widget(input_paragraph, area);

        if is_active {
            // Set cursor position relative to the input area
            f.set_cursor_position((area.x + self.cursor_position as u16 + 1, area.y + 1));
        }
    }
}