use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    text::Line,
};

pub struct StatusBar {
    pub message: String,
}

impl StatusBar {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }

    pub fn set_message(&mut self, msg: impl Into<String>) {
        self.message = msg.into();
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let paragraph = Paragraph::new(Line::from(self.message.clone()))
            .block(Block::default().borders(Borders::TOP));
        f.render_widget(paragraph, area);
    }
}