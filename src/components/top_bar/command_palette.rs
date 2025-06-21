use crate::components::notification::{send_notification, NotificationType};
use ratatui::{
    prelude::*,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, List, ListItem},
};
use crossterm::event::{KeyCode, KeyEvent};
use std::fs;
use unicode_segmentation::UnicodeSegmentation;

pub enum PaletteMode {
    Command,
    File,
}

pub struct CommandPalette {
    pub input: String,
    pub cursor_grapheme: usize,
    pub mode: PaletteMode,
    pub command_candidates: Vec<String>,
    pub file_candidates: Vec<String>,
    pub selected: usize,
}

pub enum CommandPaletteEvent {
    None,
    Exit,
    ExecuteCommand(String),
    OpenFile(String),
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPalette {
    pub fn new() -> Self {
        let commands = vec![
            ">workbench.action.files.newUntitledFile".to_string(),
            ">workbench.action.files.openFile".to_string(),
            ">workbench.action.files.save".to_string(),
            ">workbench.action.closeActiveEditor".to_string(),
            ">workbench.action.quickOpen".to_string(),
        ];
        Self {
            input: String::new(),
            cursor_grapheme: 0,
            mode: PaletteMode::Command,
            command_candidates: commands,
            file_candidates: Vec::new(),
            selected: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> CommandPaletteEvent {
        let graphemes: Vec<&str> = self.input.graphemes(true).collect();
        match key.code {
            KeyCode::Char(c) => {
                let mut new_input = String::new();
                for (i, g) in graphemes.iter().enumerate() {
                    if i == self.cursor_grapheme {
                        new_input.push(c);
                    }
                    new_input.push_str(g);
                }
                if self.cursor_grapheme == graphemes.len() {
                    new_input.push(c);
                }
                self.input = new_input;
                self.cursor_grapheme += 1;
                self.update_mode_and_candidates();
                CommandPaletteEvent::None
            }
            KeyCode::Backspace => {
                if self.input.is_empty() {
                    return CommandPaletteEvent::Exit;
                }
                if self.cursor_grapheme > 0 {
                    let mut new_input = String::new();
                    for (i, g) in graphemes.iter().enumerate() {
                        if i != self.cursor_grapheme - 1 {
                            new_input.push_str(g);
                        }
                    }
                    self.input = new_input;
                    self.cursor_grapheme -= 1;
                    self.update_mode_and_candidates();
                }
                CommandPaletteEvent::None
            }
            KeyCode::Left => {
                self.cursor_grapheme = self.cursor_grapheme.saturating_sub(1);
                CommandPaletteEvent::None
            }
            KeyCode::Right => {
                self.cursor_grapheme = (self.cursor_grapheme + 1).min(graphemes.len());
                CommandPaletteEvent::None
            }
            KeyCode::Down => {
                let max = match self.mode {
                    PaletteMode::Command => self.command_candidates.len(),
                    PaletteMode::File => self.file_candidates.len(),
                };
                if max > 0 {
                    self.selected = (self.selected + 1).min(max - 1);
                }
                CommandPaletteEvent::None
            }
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                CommandPaletteEvent::None
            }
            KeyCode::Enter => {
                match self.mode {
                    PaletteMode::Command => {
                        if self.command_candidates.is_empty() {
                            send_notification(
                                "Error: No command candidates",
                                NotificationType::Error,
                            );
                            return CommandPaletteEvent::None;
                        }
                        if let Some(cmd) = self.command_candidates.get(self.selected) {
                            let known_commands = [
                                ">workbench.action.files.newUntitledFile",
                                ">workbench.action.files.openFile",
                                ">workbench.action.files.save",
                                ">workbench.action.closeActiveEditor",
                                ">workbench.action.quickOpen",
                            ];
                            if known_commands.contains(&cmd.as_str()) {
                                return CommandPaletteEvent::ExecuteCommand(cmd.clone());
                            } else {
                                send_notification(
                                    format!("Error: Unknown Command: {}", cmd),
                                    NotificationType::Error,
                                );
                                return CommandPaletteEvent::None;
                            }
                        }
                    }
                    PaletteMode::File => {
                        if self.file_candidates.is_empty() {
                            send_notification(
                                "Error: No file candidates",
                                NotificationType::Error,
                            );
                            return CommandPaletteEvent::None;
                        }
                        if let Some(file) = self.file_candidates.get(self.selected) {
                            if std::path::Path::new(file).exists() {
                                return CommandPaletteEvent::OpenFile(file.clone());
                            } else {
                                send_notification(
                                    "Error: Couldn't find file",
                                    NotificationType::Error,
                                );
                                return CommandPaletteEvent::None;
                            }
                        }
                    }
                }
                CommandPaletteEvent::None
            }
            _ => CommandPaletteEvent::None,
        }
    }

    fn update_mode_and_candidates(&mut self) {
        if self.input.starts_with('>') {
            self.mode = PaletteMode::Command;
            let q = self.input.trim_start_matches('>').to_lowercase();
            let all_commands = vec![
                ">workbench.action.files.newUntitledFile".to_string(),
                ">workbench.action.files.openFile".to_string(),
                ">workbench.action.files.save".to_string(),
                ">workbench.action.closeActiveEditor".to_string(),
                ">workbench.action.quickOpen".to_string(),
            ];
            self.command_candidates = if q.is_empty() {
                all_commands
            } else {
                all_commands
                    .into_iter()
                    .filter(|c| c.to_lowercase().contains(&q))
                    .collect()
            };
            self.selected = 0;
        } else {
            self.mode = PaletteMode::File;
            let q = self.input.as_str().to_lowercase();
            let mut files = Vec::new();
            if let Ok(entries) = fs::read_dir(".") {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    // 大文字小文字無視で部分一致
                    if q.is_empty() || name.to_lowercase().contains(&q) {
                        files.push(name);
                    }
                }
            }
            self.file_candidates = files;
            self.selected = 0;
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let block = Block::default().borders(Borders::ALL).title("Command Palette");
        let input_line = Line::from(self.input.as_str());
        let input_paragraph = Paragraph::new(input_line).block(block.clone());
        f.render_widget(input_paragraph, area);

        // カーソル位置を「表示幅」で計算（Span::widthを使う）
        let graphemes: Vec<&str> = self.input.graphemes(true).collect();
        let cursor_str = graphemes[..self.cursor_grapheme].concat();
        let cursor_width = Span::raw(cursor_str).width();
        f.set_cursor_position((area.x + cursor_width as u16 + 1, area.y + 1));

        // 候補リスト表示
        let list_area = Rect {
            x: area.x,
            y: area.y + 3,
            width: area.width,
            height: area.height.saturating_sub(3).max(5), // 高さを最低5に
        };

        let items: Vec<ListItem> = match self.mode {
            PaletteMode::Command => self.command_candidates.iter().enumerate().map(|(i, c)| {
                if i == self.selected {
                    ListItem::new(c.clone()).style(Style::default().fg(Color::Yellow))
                } else {
                    ListItem::new(c.clone())
                }
            }).collect(),
            PaletteMode::File => self.file_candidates.iter().enumerate().map(|(i, name)| {
                if i == self.selected {
                    ListItem::new(name.clone()).style(Style::default().fg(Color::Green))
                } else {
                    ListItem::new(name.clone())
                }
            }).collect(),
        };
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(match self.mode {
                        PaletteMode::Command => "Commands",
                        PaletteMode::File => "Files",
                    })
                    .style(Style::default().bg(Color::Black)),
            )
            .highlight_style(Style::default().bg(Color::DarkGray));
        f.render_widget(list, list_area);
    }
}