use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
};

pub struct SecondarySideBar {
    _is_active: bool, // Prefixed to silence warning, assuming it might be used later
}

impl SecondarySideBar {
    pub fn new(is_active: bool) -> Self {
        Self {
            _is_active: is_active,
        }
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
