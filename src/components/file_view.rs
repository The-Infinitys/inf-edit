use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, List, ListItem},
};
use walkdir::WalkDir;
use std::path::PathBuf;

pub struct FileView {
    pub root: PathBuf,
    pub entries: Vec<PathBuf>,
    pub selected: usize,
}

impl FileView {
    pub fn new(root: PathBuf) -> Self {
        let entries = WalkDir::new(&root)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| e.path().to_path_buf())
            .collect();
        Self { root, entries, selected: 0 }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.entries.iter()
            .map(|p| {
                let name = p.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("<invalid>");
                ListItem::new(name.to_string())
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().title("File View").borders(Borders::ALL))
            .highlight_style(ratatui::style::Style::default().fg(ratatui::style::Color::Green).add_modifier(ratatui::style::Modifier::BOLD))
            .highlight_symbol("> ");

        f.render_stateful_widget(list, area, &mut self.selected_state());
    }

    fn selected_state(&self) -> ratatui::widgets::ListState {
        let mut state = ratatui::widgets::ListState::default();
        if !self.entries.is_empty() {
            state.select(Some(self.selected));
        }
        state
    }
}
