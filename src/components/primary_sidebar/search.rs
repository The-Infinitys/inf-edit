use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

enum ActiveInput {
    Search,
    Replace,
    Button,
    Results,
}

#[derive(Clone)]
struct SearchResult {
    path: PathBuf,
    line_number: usize,
    line_content: String,
}

pub struct SearchWidget {
    search_input: String,
    replace_input: String,
    active_input: ActiveInput,
    search_results: Vec<SearchResult>,
    results_state: ListState,
}

impl SearchWidget {
    pub fn new() -> Self {
        Self {
            search_input: String::new(),
            replace_input: String::new(),
            active_input: ActiveInput::Search,
            search_results: vec![],
            results_state: ListState::default(),
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, is_active: bool, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Search input
                Constraint::Length(3), // Replace input
                Constraint::Length(3), // Buttons
                Constraint::Min(0),    // Results
            ])
            .split(area);

        let search_input_p = Paragraph::new(self.search_input.as_str()).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Search")
                .border_style(self.get_border_style(is_active, &ActiveInput::Search, theme))
                .bg(theme.primary_bg),
        );

        let replace_input_p = Paragraph::new(self.replace_input.as_str()).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Replace")
                .border_style(self.get_border_style(is_active, &ActiveInput::Replace, theme))
                .bg(theme.primary_bg),
        );

        let replace_button = Paragraph::new("Replace All")
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(self.get_border_style(is_active, &ActiveInput::Button, theme))
                    .bg(theme.secondary_bg),
            );

        let results: Vec<ListItem> = self
            .search_results
            .iter()
            .map(|r| {
                let path_display =
                    if let Ok(rel_path) = r.path.strip_prefix(env::current_dir().unwrap()) {
                        rel_path.to_string_lossy()
                    } else {
                        r.path.to_string_lossy()
                    };
                let content = format!("{}:{}: {}", path_display, r.line_number, r.line_content);
                ListItem::new(content)
            })
            .collect();
        let results_list = List::new(results)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Results")
                    .border_style(self.get_border_style(is_active, &ActiveInput::Results, theme))
                    .bg(theme.primary_bg),
            )
            .highlight_style(Style::default().bg(theme.highlight_bg).fg(theme.text_fg));

        f.render_widget(search_input_p, chunks[0]);
        f.render_widget(replace_input_p, chunks[1]);
        f.render_widget(replace_button, chunks[2]);
        f.render_stateful_widget(results_list, chunks[3], &mut self.results_state);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Tab => self.cycle_focus(),
            _ => match self.active_input {
                ActiveInput::Search | ActiveInput::Replace => self.handle_text_input(key),
                ActiveInput::Button if key.code == KeyCode::Enter => {
                    self.perform_replace_all();
                    true
                }
                ActiveInput::Results => self.handle_list_nav(key),
                _ => false,
            },
        }
    }

    fn cycle_focus(&mut self) -> bool {
        self.active_input = match self.active_input {
            ActiveInput::Search => ActiveInput::Replace,
            ActiveInput::Replace => ActiveInput::Button,
            ActiveInput::Button => ActiveInput::Results,
            ActiveInput::Results => ActiveInput::Search,
        };
        true
    }

    fn handle_text_input(&mut self, key: KeyEvent) -> bool {
        let input_str = match self.active_input {
            ActiveInput::Search => &mut self.search_input,
            ActiveInput::Replace => &mut self.replace_input,
            _ => return false,
        };
        match key.code {
            KeyCode::Char(c) => {
                input_str.push(c);
                true
            }
            KeyCode::Backspace => {
                input_str.pop();
                true
            }
            KeyCode::Enter if matches!(self.active_input, ActiveInput::Search) => {
                self.perform_search();
                true
            }
            _ => false,
        }
    }

    fn handle_list_nav(&mut self, key: KeyEvent) -> bool {
        if self.search_results.is_empty() {
            return false;
        }
        let len = self.search_results.len();
        match key.code {
            KeyCode::Down => {
                let i = self.results_state.selected().map_or(0, |i| (i + 1) % len);
                self.results_state.select(Some(i));
                true
            }
            KeyCode::Up => {
                let i =
                    self.results_state
                        .selected()
                        .map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
                self.results_state.select(Some(i));
                true
            }
            _ => false,
        }
    }

    fn perform_search(&mut self) {
        self.search_results.clear();
        self.results_state.select(None);
        if self.search_input.is_empty() {
            return;
        }

        let search_term = self.search_input.clone();
        let root = env::current_dir().unwrap_or_else(|_| "/".into());

        for entry in WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if path.to_string_lossy().contains(".git") {
                continue;
            }

            if let Ok(content) = fs::read_to_string(path) {
                for (i, line) in content.lines().enumerate() {
                    if line.contains(&search_term) {
                        self.search_results.push(SearchResult {
                            path: path.to_path_buf(),
                            line_number: i + 1,
                            line_content: line.trim().to_string(),
                        });
                    }
                }
            }
        }
        if !self.search_results.is_empty() {
            self.results_state.select(Some(0));
        }
    }

    fn perform_replace_all(&mut self) {
        if self.search_input.is_empty() || self.search_results.is_empty() {
            return;
        }

        let mut file_changes: HashMap<PathBuf, String> = HashMap::new();
        for result in &self.search_results {
            if !file_changes.contains_key(&result.path) {
                if let Ok(content) = fs::read_to_string(&result.path) {
                    file_changes.insert(result.path.clone(), content);
                }
            }
        }

        for (_path, content) in file_changes.iter_mut() {
            *content = content.replace(&self.search_input, &self.replace_input);
        }

        for (path, new_content) in file_changes {
            let _ = fs::write(path, new_content);
        }

        self.perform_search();
    }

    fn get_border_style(&self, is_active: bool, input: &ActiveInput, theme: &Theme) -> Style {
        if is_active && std::mem::discriminant(&self.active_input) == std::mem::discriminant(input)
        {
            Style::default().fg(theme.highlight_fg)
        } else {
            Style::default().fg(theme.text_fg)
        }
    }
}

impl Default for SearchWidget {
    fn default() -> Self {
        Self::new()
    }
}
