use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

#[derive(Default)]
pub struct SearchWidget {}

impl SearchWidget {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, _is_active: bool) {
        let search_block = Block::default()
            .title("Search")
            .borders(Borders::ALL);
        let text = Paragraph::new("Search functionality will be here.").block(search_block);
        f.render_widget(text, area);
    }
}