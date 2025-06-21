use ratatui::{
    prelude::*, // Keep for implicit use by widgets
    text::Line,
    widgets::{Block, Borders, Paragraph},
};
use crossterm::event::{KeyCode, KeyEvent};

#[derive(Default)]
pub struct CommandPalette {
    pub input: String,
    pub cursor_position: usize,
    // Add fields for command list, selected command, etc. later
}

impl CommandPalette {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c) => {
                self.input.insert(self.cursor_position, c);
                self.cursor_position += 1;
                true
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.input.remove(self.cursor_position);
                }
                true
            }
            KeyCode::Left => {
                self.cursor_position = self.cursor_position.saturating_sub(1);
                true
            }
            KeyCode::Right => {
                self.cursor_position = (self.cursor_position + 1).min(self.input.len());
                true
            }
            _ => false, // Key not handled by command palette input
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let block = Block::default().borders(Borders::ALL).title("Command Palette");
        let input_line = Line::from(self.input.as_str());
        let input_paragraph = Paragraph::new(input_line).block(block);
        f.render_widget(input_paragraph, area);

        // Set cursor position
        f.set_cursor_position((area.x + self.cursor_position as u16 + 1, area.y + 1));
    }
}