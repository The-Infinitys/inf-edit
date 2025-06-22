use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use notify::{recommended_watcher, Event as NotifyEvent, RecursiveMode, Watcher};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState},
};
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver},
};
use walkdir::WalkDir;

pub struct FileView {
    path: PathBuf,
    files: Vec<PathBuf>,
    list_state: ListState,
    _watcher: Option<Box<dyn Watcher + Send>>,
    rx: Option<Receiver<Result<NotifyEvent, notify::Error>>>,
    needs_refresh: bool,
}

impl FileView {
    pub fn new(path: PathBuf) -> Self {
        let mut view = Self {
            path,
            files: Vec::new(),
            list_state: ListState::default(),
            _watcher: None,
            rx: None,
            needs_refresh: false,
        };
        view.refresh_files();
        view.start_watching();
        view
    }

    pub fn current_path(&self) -> &PathBuf {
        &self.path
    }

    fn refresh_files(&mut self) {
        self.files = WalkDir::new(&self.path)
            .min_depth(1)
            .max_depth(1) // Only show top-level files/dirs for simplicity
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| e.path().to_path_buf())
            .collect();
        self.files.sort();

        if !self.files.is_empty() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }
        self.needs_refresh = false;
    }

    fn start_watching(&mut self) {
        let (tx, rx) = mpsc::channel();
        let mut watcher = match recommended_watcher(move |res| {
            let _ = tx.send(res);
        }) {
            Ok(w) => w,
            Err(_) => return,
        };

        if watcher.watch(&self.path, RecursiveMode::Recursive).is_ok() {
            self._watcher = Some(Box::new(watcher));
            self.rx = Some(rx);
        }
    }

    pub fn poll_file_changes(&mut self) {
        if let Some(rx) = &self.rx {
            if rx.try_recv().is_ok() {
                self.needs_refresh = true;
            }
        }
    }

    pub fn refresh_if_needed(&mut self) {
        if self.needs_refresh {
            self.refresh_files();
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Down => self.select_next(),
            KeyCode::Up => self.select_previous(),
            _ => return false,
        }
        true
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, is_active: bool, theme: &Theme) {
        let border_style = if is_active {
            Style::default().fg(theme.highlight_fg)
        } else {
            Style::default().fg(theme.text_fg)
        };

        let items: Vec<ListItem> = self
            .files
            .iter()
            .map(|path| {
                let filename = path.file_name().unwrap_or_default().to_string_lossy();
                let style = if path.is_dir() {
                    Style::default().fg(theme.highlight_fg)
                } else {
                    Style::default().fg(theme.text_fg)
                };
                ListItem::new(Span::styled(filename, style))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title("Files")
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .highlight_style(Style::default().bg(theme.highlight_bg))
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, area, &mut self.list_state);
    }

    pub fn selected_file(&self) -> Option<PathBuf> {
        self.list_state
            .selected()
            .and_then(|i| self.files.get(i).cloned())
    }

    fn select_next(&mut self) {
        let len = self.files.len();
        if len == 0 {
            return;
        }
        let i = self.list_state.selected().map_or(0, |i| (i + 1) % len);
        self.list_state.select(Some(i));
    }

    fn select_previous(&mut self) {
        let len = self.files.len();
        if len == 0 {
            return;
        }
        let i = self
            .list_state
            .selected()
            .map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
        self.list_state.select(Some(i));
    }
}
