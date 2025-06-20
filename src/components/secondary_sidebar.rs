use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct SecondarySideBar {
    is_active: bool,
}

impl SecondarySideBar {
    pub fn new(is_active: bool) -> Self {
        Self { is_active }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let block = Block::default().title("Secondary Sidebar").borders(Borders::ALL);
        // .border_style(if self.is_active { ratatui::style::Style::default().fg(ratatui::style::Color::Green) } else { ratatui::style::Style::default() });
        let paragraph = Paragraph::new("Secondary Sidebar Content").block(block);
        f.render_widget(paragraph, area);
    }
}