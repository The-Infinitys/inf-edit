use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

#[derive(Default)]
pub struct GitWidget {}

impl GitWidget {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render(&self, f: &mut Frame, area: Rect, is_active: bool) {
        let mut git_block = Block::default()
            .title("Git")
            .borders(Borders::ALL);
        if is_active {
            git_block = git_block.style(Style::default().fg(Color::Yellow));
        }
        let text = Paragraph::new("Git functionality will be here.").block(git_block);
        f.render_widget(text, area);
    }
}