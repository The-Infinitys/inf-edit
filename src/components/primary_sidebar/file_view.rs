use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::ListState,
    widgets::{Block, Borders, List, ListItem},
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct TreeDisplayItem {
    path: PathBuf,
    name: String,
    depth: usize,
    is_dir: bool,
    is_expanded: bool,
    is_parent_link: bool,
}

pub struct FileView {
    current_root: PathBuf,
    display_items: Vec<TreeDisplayItem>,
    list_state: ListState,
    expanded_state: HashSet<PathBuf>,
    path_history: Vec<(PathBuf, ListState, HashSet<PathBuf>)>, // For current_root navigation
}

impl FileView {
    pub fn new(initial_path: PathBuf) -> Self {
        let mut fv = Self {
            current_root: initial_path,
            display_items: Vec::new(),
            list_state: ListState::default(),
            expanded_state: HashSet::new(),
            path_history: Vec::new(),
        };
        fv.list_state.select(Some(0));
        fv.refresh_items();
        fv
    }

    fn refresh_items(&mut self) {
        let old_selected_path = self
            .list_state
            .selected()
            .and_then(|idx| self.display_items.get(idx))
            .map(|item| item.path.clone());

        self.display_items.clear();
        let mut temp_list = Vec::new();

        // if self.current_root.parent().is_some() {
        //     self.display_items.push(TreeDisplayItem {
        //         path: self.current_root.parent().unwrap().to_path_buf(),
        //         name: "[..]".to_string(),
        //         depth: 0,
        //         is_dir: true,
        //         is_expanded: false,
        //         is_parent_link: true,
        //     });
        // }

        self.build_recursive_list(&self.current_root, 0, &mut temp_list);
        self.display_items.extend(temp_list);

        if let Some(selected_path) = old_selected_path {
            if let Some(new_idx) = self
                .display_items
                .iter()
                .position(|item| item.path == selected_path)
            {
                self.list_state.select(Some(new_idx));
            } else {
                let new_len = self.display_items.len();
                if new_len > 0 {
                    let old_idx = self.list_state.selected().unwrap_or(0);
                    self.list_state.select(Some(old_idx.min(new_len - 1)));
                } else {
                    self.list_state.select(None);
                }
            }
        } else if !self.display_items.is_empty() {
            self.list_state.select(Some(0));
        }

        if self.list_state.selected().is_none() && !self.display_items.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    fn build_recursive_list(
        &self,
        dir_path: &Path,
        depth: usize,
        items_list: &mut Vec<TreeDisplayItem>,
    ) {
        if let Ok(entries) = std::fs::read_dir(dir_path) {
            let mut current_level_entries = Vec::new();
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
                if name.starts_with('.') {
                    continue;
                } // Skip hidden files/dirs

                let is_dir = path.is_dir();
                current_level_entries.push(TreeDisplayItem {
                    path: path.clone(),
                    name,
                    depth,
                    is_dir,
                    is_expanded: is_dir && self.expanded_state.contains(&path),
                    is_parent_link: false,
                });
            }

            current_level_entries.sort_by(|a, b| {
                a.is_dir
                    .cmp(&b.is_dir)
                    .reverse()
                    .then_with(|| a.name.cmp(&b.name))
            });

            for item in current_level_entries {
                items_list.push(item.clone());
                if item.is_dir && item.is_expanded {
                    self.build_recursive_list(&item.path, depth + 1, items_list);
                }
            }
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, active: bool, theme: &crate::theme::Theme) {
        let items: Vec<ListItem> = self
            .display_items
            .iter()
            .map(|item| {
                let indent = "  ".repeat(item.depth);
                let icon = if item.is_parent_link {
                    "📁 " // Icon for "[..]"
                } else if item.is_dir {
                    if item.is_expanded { "▼ " } else { "▶ " }
                } else {
                    "📄 " // File icon
                };
                let name_suffix = if item.is_dir && !item.is_parent_link {
                    "/"
                } else {
                    ""
                };
                let display_name = format!("{}{}{}{}", indent, icon, item.name, name_suffix);
                ListItem::new(Line::from(Span::raw(display_name))).style(Style::default().fg(theme.text_fg))
            })
            .collect();

        let block = Block::default()
            .title(format!(" File View: {} ", self.current_root.display()))
            .borders(Borders::ALL)
            .bg(theme.primary_bg)
            .border_style(
                if active {
                    Style::default().fg(theme.highlight_fg)
                } else {
                    Style::default().fg(theme.text_fg)
                }
            );

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default().bg(theme.highlight_bg).fg(theme.text_fg).add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        f.render_stateful_widget(list, area, &mut self.list_state);
    }

    pub fn next(&mut self) {
        if !self.display_items.is_empty() {
            let i = self.list_state.selected().map_or(0, |i| (i + 1) % self.display_items.len());
            self.list_state.select(Some(i));
        }
    }

    pub fn previous(&mut self) {
        if !self.display_items.is_empty() {
            let i = self.list_state.selected().map_or(0, |i| {
                if i == 0 {
                    self.display_items.len() - 1
                } else {
                    i - 1
                }
            });
            self.list_state.select(Some(i));
        }
    }

    pub fn enter(&mut self) {
        if let Some(selected_index) = self.list_state.selected() {
            if let Some(item) = self.display_items.get(selected_index).cloned() {
                // Cloned to avoid borrow checker issues with self.refresh_items
                if item.is_parent_link {
                    self.go_to_parent_directory();
                } else if item.is_dir {
                    if self.expanded_state.contains(&item.path) {
                        self.expanded_state.remove(&item.path);
                    } else {
                        self.expanded_state.insert(item.path.clone());
                    }
                    self.refresh_items();
                }
                // If it's a file, selected_file() will be called by the event handler
            }
        }
    }

    pub fn back(&mut self) {
        if let Some(selected_index) = self.list_state.selected() {
            if let Some(item) = self.display_items.get(selected_index).cloned() {
            if item.is_dir && item.is_expanded && !item.is_parent_link {
                self.expanded_state.remove(&item.path);
                self.refresh_items();
            } else if item.depth > 0 {
                // Try to select parent in the list
                let mut parent_idx = selected_index;
                while parent_idx > 0 {
                    parent_idx -= 1;
                    if let Some(parent_item) = self.display_items.get(parent_idx) {
                        if parent_item.depth < item.depth {
                            self.list_state.select(Some(parent_idx));
                            return;
                        }
                    }
                }
                // If no direct parent found above, or at depth 0, try to go to parent directory
                self.go_to_parent_directory();
            } else {
                self.go_to_parent_directory();
            }
            }
        } else {
            self.go_to_parent_directory();
        }
    }

    fn go_to_parent_directory(&mut self) {
        if let Some(parent) = self.current_root.parent() {
            self.path_history.push((
                self.current_root.clone(),
                self.list_state.clone(),
                self.expanded_state.clone(),
            ));
            self.current_root = parent.to_path_buf();
            self.list_state.select(Some(0)); // Or try to find the old current_root name
            // self.expanded_state.clear(); // Optionally clear or manage intelligently
            self.refresh_items();
        } else if let Some((prev_root, prev_selected, prev_expanded)) = self.path_history.pop() {
            // Fallback to history if current_root has no parent but history exists
            self.current_root = prev_root;
            self.list_state = prev_selected;
            self.expanded_state = prev_expanded;
            self.refresh_items();
        }
    }

    pub fn selected_file(&self) -> Option<PathBuf> {
        self.list_state.selected().and_then(|i| {
            self.display_items.get(i).and_then(|item| {
                if !item.is_dir && !item.is_parent_link {
                    Some(item.path.clone())
                } else {
                    None
                }
            })
        })
    }

    pub fn current_path(&self) -> &PathBuf {
        &self.current_root
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.next();
                true
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.previous();
                true
            }
            KeyCode::Enter => {
                self.enter();
                true
            }
            KeyCode::Char('h') => { // ← バックスペースは除外
                self.back();
                true
            }
            _ => false,
        }
    }
}
