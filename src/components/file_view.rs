
use ratatui::{Frame, layout::Rect, widgets::{Block, Borders}};

pub struct FileView;

impl FileView {
    pub fn render(f: &mut Frame, area: Rect) {
        let block = Block::default().title("File View").borders(Borders::ALL);
        f.render_widget(block, area);
    }
}