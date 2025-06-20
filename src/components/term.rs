use ratatui::{Frame, layout::Rect, widgets::{Block, Borders}};

pub struct Term;

impl Term {
    pub fn render(f: &mut Frame, area: Rect) {
        let block = Block::default().title("Terminal").borders(Borders::ALL);
        f.render_widget(block, area);
    }
}