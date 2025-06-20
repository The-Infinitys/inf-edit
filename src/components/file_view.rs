use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, List, ListItem},
    style::{Style, Color, Modifier},
    widgets::ListState,
};
use walkdir::WalkDir;
use std::path::{Path, PathBuf};

pub struct FileView {
    pub root: PathBuf,
    pub entries: Vec<PathBuf>,
    pub selected: usize,
    pub history: Vec<(PathBuf, usize)>, // 戻る用
}

impl FileView {
    pub fn new(root: PathBuf) -> Self {
        let entries = Self::read_entries(&root);
        Self { root, entries, selected: 0, history: Vec::new() }
    }

    fn read_entries(dir: &Path) -> Vec<PathBuf> {
        let mut entries: Vec<_> = WalkDir::new(dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| e.path().to_path_buf())
            .collect();
        entries.sort();
        entries
    }

    pub fn render(&self, f: &mut Frame, area: Rect, active: bool) {
        let items: Vec<ListItem> = self.entries.iter()
            .map(|p| {
                let name = p.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("<invalid>");
                let display = if p.is_dir() {
                    format!("{}/", name)
                } else {
                    name.to_string()
                };
                ListItem::new(display)
            })
            .collect();

        let block = Block::default()
            .title("File View")
            .borders(Borders::ALL)
            .border_style(
                if active {
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                }
            );

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");

        f.render_stateful_widget(list, area, &mut self.selected_state());
    }

    fn selected_state(&self) -> ListState {
        let mut state = ListState::default();
        if !self.entries.is_empty() {
            state.select(Some(self.selected));
        }
        state
    }

    /// 下に移動
    pub fn next(&mut self) {
        if !self.entries.is_empty() {
            self.selected = (self.selected + 1) % self.entries.len();
        }
    }

    /// 上に移動
    pub fn previous(&mut self) {
        if !self.entries.is_empty() {
            if self.selected == 0 {
                self.selected = self.entries.len() - 1;
            } else {
                self.selected -= 1;
            }
        }
    }

    /// ディレクトリなら中に入る
    pub fn enter(&mut self) {
        if let Some(path) = self.entries.get(self.selected) {
            if path.is_dir() {
                self.history.push((self.root.clone(), self.selected));
                self.root = path.clone();
                self.entries = Self::read_entries(&self.root);
                self.selected = 0;
            }
        }
    }

    /// 1つ上のディレクトリに戻る
    pub fn back(&mut self) {
        if let Some((prev_root, prev_selected)) = self.history.pop() {
            self.root = prev_root;
            self.entries = Self::read_entries(&self.root);
            self.selected = prev_selected;
        }
    }

    /// 選択中のファイルパスを返す（ファイルのみ）
    pub fn selected_file(&self) -> Option<PathBuf> {
        self.entries.get(self.selected).and_then(|p| {
            if p.is_file() {
                Some(p.clone())
            } else {
                None
            }
        })
    }
}