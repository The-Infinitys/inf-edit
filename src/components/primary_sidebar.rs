use super::file_view::FileView;
use ratatui::{Frame, layout::Rect, widgets::Block};

pub struct PrimarySideBar<'a> {
    file_view: &'a mut FileView,
    is_active: bool,
}

impl<'a> PrimarySideBar<'a> {
    pub fn new(file_view: &'a mut FileView, is_active: bool) -> Self {
        Self {
            file_view,
            is_active,
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let _block = Block::default().title("Primary Sidebar (File View)"); // Prefixed with _ or remove if not used
        // .borders(Borders::ALL) // Optional: add borders if needed
        // .border_style(if self.is_active {
        //     ratatui::style::Style::default().fg(ratatui::style::Color::Green)
        // } else {
        //     ratatui::style::Style::default()
        // });
        self.file_view.render(f, area, self.is_active); // Pass block if FileView::render supports it
    }
}
