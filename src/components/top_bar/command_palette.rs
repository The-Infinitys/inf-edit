use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};
use std::env;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub action: CommandAction,
}

#[derive(Debug, Clone)]
pub enum CommandAction {
    OpenFile(PathBuf),
    SetThemePreset(String),
    OpenSettings,
    ResetSettings,
}

#[derive(Debug, Clone)]
pub enum CommandPaletteEvent {
    Exit,
    OpenFile(PathBuf),
    SetThemePreset(String),
    OpenSettings,
    ResetSettings,
    None,
}

pub struct CommandPalette {
    pub input: String,
    static_commands: Vec<Command>,
    filtered_items: Vec<Command>,
    selected_index: usize,
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPalette {
    pub fn new() -> Self {
        let static_commands = vec![
            Command {
                name: "Open Settings".to_string(),
                action: CommandAction::OpenSettings,
            },
            Command {
                name: "Reset Settings".to_string(),
                action: CommandAction::ResetSettings,
            },
            Command {
                name: "Set Theme: Gruvbox".to_string(),
                action: CommandAction::SetThemePreset("gruvbox".to_string()),
            },
            Command {
                name: "Set Theme: Catppuccin".to_string(),
                action: CommandAction::SetThemePreset("catppuccin".to_string()),
            },
        ];
        Self {
            input: String::new(),
            filtered_items: static_commands.clone(),
            static_commands,
            selected_index: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> CommandPaletteEvent {
        match key.code {
            KeyCode::Char(c) => {
                self.input.push(c);
                self.update_filtered_items();
            }
            KeyCode::Backspace => {
                self.input.pop();
                self.update_filtered_items();
            }
            KeyCode::Down => {
                if !self.filtered_items.is_empty() {
                    self.selected_index = (self.selected_index + 1) % self.filtered_items.len();
                }
            }
            KeyCode::Up => {
                if !self.filtered_items.is_empty() {
                    self.selected_index = self
                        .selected_index
                        .checked_sub(1)
                        .unwrap_or(self.filtered_items.len() - 1);
                }
            }
            KeyCode::Enter => {
                // Clone the command to release the borrow on `self.filtered_items` immediately.
                if let Some(cmd) = self.filtered_items.get(self.selected_index).cloned() {
                    // Now that the borrow is released, we can safely modify the state.
                    self.input.clear();
                    self.filtered_items = self.static_commands.clone();
                    self.selected_index = 0;

                    // Use the owned `cmd` to create the event.
                    return match cmd.action {
                        CommandAction::OpenFile(p) => CommandPaletteEvent::OpenFile(p),
                        CommandAction::SetThemePreset(p) => CommandPaletteEvent::SetThemePreset(p),
                        CommandAction::OpenSettings => CommandPaletteEvent::OpenSettings,
                        CommandAction::ResetSettings => CommandPaletteEvent::ResetSettings,
                    };
                }
            }
            KeyCode::Esc => {
                self.input.clear();
                self.filtered_items = self.static_commands.clone();
                self.selected_index = 0;
                return CommandPaletteEvent::Exit;
            }
            _ => {}
        }
        CommandPaletteEvent::None
    }

    fn update_filtered_items(&mut self) {
        if self.input.starts_with('>') {
            let query = &self.input[1..].to_lowercase();
            if query.is_empty() {
                self.filtered_items = self.static_commands.clone();
            } else {
                self.filtered_items = self
                    .static_commands
                    .iter()
                    .filter(|cmd| cmd.name.to_lowercase().contains(query))
                    .cloned()
                    .collect();
            }
        } else if !self.input.is_empty() {
            self.filtered_items = self.find_files(&self.input);
        } else {
            self.filtered_items = self.static_commands.clone();
        }
        self.selected_index = 0;
    }

    fn find_files(&self, query: &str) -> Vec<Command> {
        let mut results = Vec::new();
        let current_dir = match env::current_dir() {
            Ok(dir) => dir,
            Err(_) => return results,
        };

        for entry in WalkDir::new(current_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let path_str = e.path().to_string_lossy();
                !path_str.contains("/.git/") && !path_str.contains("/target/")
            })
            .filter(|e| e.file_type().is_file())
            .take(50)
        {
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.to_lowercase().contains(&query.to_lowercase()) {
                    results.push(Command {
                        name: path.to_string_lossy().to_string(),
                        action: CommandAction::OpenFile(path.to_path_buf()),
                    });
                }
            }
        }
        results
    }

    pub fn render(&self, f: &mut Frame, area: Rect, app: &App) {
        let popup_width = (area.width / 2).max(80);
        let popup_area_base = Rect {
            x: (area.width.saturating_sub(popup_width)) / 2,
            y: 1,
            width: popup_width,
            height: area.height,
        };

        let list_height = self.filtered_items.len().min(15) as u16;
        let popup_height = list_height + 3; // +3 for the input box with borders

        let popup_area = Rect {
            height: popup_height,
            ..popup_area_base
        };
        f.render_widget(Clear, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(popup_area);

        let input_paragraph = Paragraph::new(self.input.as_str()).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Command Palette ('>' for commands)")
                .border_style(Style::default().fg(app.theme.highlight_fg)),
        );
        f.render_widget(input_paragraph, chunks[0]);
        f.set_cursor_position((chunks[0].x + self.input.len() as u16 + 1, chunks[0].y + 1));

        let items: Vec<ListItem> = self
            .filtered_items
            .iter()
            .map(|cmd| ListItem::new(cmd.name.clone()))
            .collect();
        let mut list_state = ListState::default();
        list_state.select(Some(self.selected_index));

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Results"))
            .highlight_style(
                Style::default()
                    .bg(app.theme.highlight_bg)
                    .fg(app.theme.text_fg),
            )
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, chunks[1], &mut list_state);
    }
}
