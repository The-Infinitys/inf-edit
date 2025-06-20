use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
};

pub struct SecondarySideBar {
    pub is_visible: bool,
}

impl Default for SecondarySideBar {
    fn default() -> Self {
        Self::new()
    }
}

impl SecondarySideBar {
    pub fn new() -> Self {
        Self {
            is_visible: false, // Start hidden by default
        }
    }

    pub fn toggle_visibility(&mut self) {
        self.is_visible = !self.is_visible;
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title("Secondary Sidebar")
            .borders(Borders::ALL);
        // .border_style(if self.is_active { ratatui::style::Style::default().fg(ratatui::style::Color::Green) } else { ratatui::style::Style::default() });
        let paragraph = Paragraph::new("Secondary Sidebar Content").block(block);
        f.render_widget(paragraph, area);
    }
}
