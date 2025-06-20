use ratatui::{Frame, layout::Rect, widgets::{Block, Borders}};

pub struct Editor;

impl Editor {
    pub fn render(f: &mut Frame, area: Rect) {
        let block = Block::default().title("Editor").borders(Borders::ALL);
        f.render_widget(block, area);
    }
}