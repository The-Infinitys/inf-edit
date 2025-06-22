use crate::components::notification::{send_notification, NotificationType};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::fs;
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;

pub enum PaletteMode {
    Command,
    File,
}

pub struct Command {
    pub name: String,
    pub explain: String,
    pub func: Option<Box<dyn FnMut(&mut CommandPalette) + Send>>,
}

pub struct CommandPalette {
    pub input: String,
    pub cursor_grapheme: usize,
    pub mode: PaletteMode,
    pub command_candidates: Vec<String>,
    pub file_candidates: Vec<String>,
    pub state: ListState, // ListStateを追加
    pub commands: HashMap<String, Command>,
    pub input_mode: Option<InputMode>,
    pub input_buffer: String,
    pub selection_items: Vec<String>,
    pub check_items: Vec<(String, bool)>,
}

pub enum InputMode {
    StringInput { prompt: String },
    Selection { prompt: String, items: Vec<String> },
    Check { prompt: String, items: Vec<(String, bool)> },
}

pub enum CommandPaletteEvent {
    None,
    Exit,
    ExecuteCommand(String),
    OpenFile(String),
    OpenSettings,
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPalette {
    pub fn new() -> Self {
        let mut cp = Self {
            input: String::new(),
            cursor_grapheme: 0,
            mode: PaletteMode::Command,
            command_candidates: Vec::new(),
            file_candidates: Vec::new(), // selectedを削除
            state: ListState::default(), // ListStateを初期化
            commands: HashMap::new(),
            input_mode: None,
            input_buffer: String::new(),
            selection_items: Vec::new(),
            check_items: Vec::new(),
        };
        // デフォルトコマンドを追加
        cp.add_command(
            ">workbench.action.files.newUntitledFile".to_string(),
            "新しいファイルを作成".to_string(),
            Box::new(|_cp| {
                send_notification("新しいファイルを作成", NotificationType::Info);
            }),
        );
        cp.add_command(
            ">workbench.action.files.openFile".to_string(),
            "ファイルを開く".to_string(),
            Box::new(|_cp| {
                send_notification("ファイルを開く", NotificationType::Info);
            }),
        );
        cp.add_command(
            ">workbench.action.files.save".to_string(),
            "ファイルを保存".to_string(),
            Box::new(|_cp| {
                send_notification("ファイルを保存", NotificationType::Info);
            }),
        );
        cp.add_command(
            ">workbench.action.closeActiveEditor".to_string(),
            "エディタを閉じる".to_string(),
            Box::new(|_cp| {
                send_notification("エディタを閉じる", NotificationType::Info);
            }),
        );
        cp.add_command(
            ">workbench.action.quickOpen".to_string(),
            "クイックオープン".to_string(),
            Box::new(|_cp| {
                send_notification("クイックオープン", NotificationType::Info);
            }),
        );
        cp.add_command(
            ">workbench.action.openSettings".to_string(),
            "設定を開く".to_string(),
            Box::new(|_cp| {
                // This will be handled by the event enum
            }),
        );
        cp.update_mode_and_candidates();
        cp
    }

    /// コマンドを追加
    pub fn add_command(
        &mut self,
        name: String,
        explain: String,
        func: Box<dyn FnMut(&mut CommandPalette) + Send>,
    ) {
        self.commands.insert(
            name.clone(),
            Command {
                name: name.clone(),
                explain,
                func: Some(func),
            },
        );
        self.update_mode_and_candidates();
    }

    pub fn get_results_count(&self) -> usize {
        match self.mode {
            PaletteMode::Command => self.command_candidates.len(),
            PaletteMode::File => self.file_candidates.len(),
        }
    }

    /// テキスト入力を受け付ける
    pub fn get_string(&mut self, prompt: String) {
        self.input_mode = Some(InputMode::StringInput { prompt });
        self.input_buffer.clear();
    }

    /// 選択肢から選ばせる
    pub fn get_selection(&mut self, prompt: String, items: Vec<String>) {
        self.input_mode = Some(InputMode::Selection { prompt, items: items.clone() });
        self.selection_items = items;
        self.state.select(Some(0)); // ListStateの選択を初期化
    }

    /// チェックボックス選択
    pub fn get_check(&mut self, prompt: String, items: Vec<(String, bool)>) {
        self.input_mode = Some(InputMode::Check { prompt, items: items.clone() });
        self.check_items = items;
        self.state.select(Some(0)); // ListStateの選択を初期化
    }

    /// コマンド候補リストの最大高さを計算（縦幅を可変にする）
    fn calc_command_list_height(&self, max_height: u16) -> u16 {
        let items_len = match self.mode {
            PaletteMode::Command => self.command_candidates.len(),
            PaletteMode::File => self.file_candidates.len(),
        } as u16;
        items_len.min(max_height)
    }

    /// コマンドを実行
    pub fn execute_selected_command(&mut self) {
        if let Some(cmd_name) = self.command_candidates.get(self.state.selected().unwrap_or(0)).cloned() { // ListStateから選択項目を取得
            // コマンドを一時的に取り出す
            let mut func_opt: Option<Box<dyn FnMut(&mut CommandPalette) + Send>> = None;
            if let Some(cmd) = self.commands.get_mut(&cmd_name) {
                // funcをOptionで一時的にmove
                std::mem::swap(&mut func_opt, &mut cmd.func.take());
            }
            if let Some(mut func) = func_opt {
                func(self);
                // 実行後、funcを戻す
                if let Some(cmd) = self.commands.get_mut(&cmd_name) {
                    cmd.func = Some(func);
                }
            } else {
                send_notification(
                    format!("Error: Unknown Command: {}", cmd_name),
                    NotificationType::Error,
                );
            }
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
                let items_len = match self.mode {
                    PaletteMode::Command => self.command_candidates.len(),
                    PaletteMode::File => self.file_candidates.len(),
                };
                if items_len > 0 {
                    let i = match self.state.selected() {
                        Some(i) => (i + 1) % items_len,
                        None => 0,
                    };
                    self.state.select(Some(i));
                }
                CommandPaletteEvent::None
            }
            KeyCode::Up => {
                let items_len = match self.mode {
                    PaletteMode::Command => self.command_candidates.len(),
                    PaletteMode::File => self.file_candidates.len(),
                };
                if items_len > 0 {
                    let i = match self.state.selected() {
                        Some(i) => {
                            if i == 0 {
                                items_len - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    self.state.select(Some(i));
                }
                CommandPaletteEvent::None
            }
            KeyCode::PageUp => { // PageUp/PageDownを追加
                self.state.select(self.state.selected().map(|s| s.saturating_sub(10).max(0)));
                CommandPaletteEvent::None
            }
            KeyCode::Enter => {
                let selected_index = self.state.selected().unwrap_or(0); // ListStateから選択項目を取得
                match self.mode {
                    PaletteMode::Command => {
                        if let Some(cmd_name) = self.command_candidates.get(selected_index).cloned() {
                            if cmd_name == ">workbench.action.openSettings" {
                                return CommandPaletteEvent::OpenSettings;
                            }
                            self.execute_selected_command();
                            CommandPaletteEvent::ExecuteCommand(cmd_name)
                        } else {
                            CommandPaletteEvent::None
                        }
                    }
                    PaletteMode::File => {
                        if let Some(file) = self.file_candidates.get(selected_index) {
                            if std::path::Path::new(file).exists() {
                                CommandPaletteEvent::OpenFile(file.clone())
                            } else {
                                send_notification(
                                    "Error: Couldn't find file",
                                    NotificationType::Error,
                                );
                                CommandPaletteEvent::None
                            }
                        } else {
                            CommandPaletteEvent::None
                        }
                    }
                }
            }
            KeyCode::PageDown => {
                let items_len = match self.mode {
                    PaletteMode::Command => self.command_candidates.len(),
                    PaletteMode::File => self.file_candidates.len(),
                };
                self.state.select(self.state.selected().map(|s| (s + 10).min(items_len.saturating_sub(1))));
                CommandPaletteEvent::None
            }
            _ => CommandPaletteEvent::None,
        }
    }

    fn update_mode_and_candidates(&mut self) {
        if self.input.starts_with('>') {
            self.mode = PaletteMode::Command;
            let q = self.input.trim_start_matches('>').to_lowercase();
            let mut all_commands: Vec<String> = self.commands.keys().cloned().collect();
            all_commands.sort();
            self.command_candidates = if q.is_empty() {
                all_commands
            } else {
                all_commands
                    .into_iter()
                    .filter(|c| c.to_lowercase().contains(&q))
                    .collect()
            };
            self.state.select(if !self.command_candidates.is_empty() { Some(0) } else { None }); // 選択状態をリセット
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
            self.state.select(if !self.file_candidates.is_empty() { Some(0) } else { None }); // 選択状態をリセット
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) { // Added theme parameter
        let block = Block::default().borders(Borders::ALL).title("Command Palette");
        let input_line = Line::from(self.input.as_str());
        let input_paragraph = Paragraph::new(input_line).block(block.clone());
        f.render_widget(input_paragraph, area);

        // カーソル位置
        let graphemes: Vec<&str> = self.input.graphemes(true).collect();
        let cursor_str = graphemes[..self.cursor_grapheme].concat();
        let cursor_width = Span::raw(cursor_str).width();
        f.set_cursor_position((area.x + cursor_width as u16 + 1, area.y + 1));

        // input_modeがSomeなら、専用UIのみ描画してreturn
        if let Some(input_mode) = &self.input_mode {
            let mode_block = Block::default().borders(Borders::ALL).title("Input Mode");
            match input_mode {
                InputMode::StringInput { prompt } => {
                    let prompt_line = Line::from(prompt.as_str());
                    let prompt_paragraph = Paragraph::new(prompt_line).block(mode_block.clone());
                    f.render_widget(prompt_paragraph, area);

                    let input_line = Line::from(self.input_buffer.as_str());
                    let input_paragraph = Paragraph::new(input_line).block(mode_block.clone());
                    f.render_widget(input_paragraph, area);
                }
                InputMode::Selection { prompt, items } => {
                    let prompt_line = Line::from(prompt.as_str());
                    let prompt_paragraph = Paragraph::new(prompt_line).block(mode_block.clone());
                    f.render_widget(prompt_paragraph, area);

                    let items: Vec<ListItem> = items.iter().map(|name| {
                        ListItem::new(name.clone())
                    }).collect();
                    let mut list_state = self.state.clone(); // ListStateをクローンして可変にする
                    let list = List::new(items)
                        .highlight_style(Style::default().bg(theme.highlight_bg).fg(theme.text_fg)) // ハイライトスタイルをListに設定
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title("Select an Option")
                                .style(Style::default().bg(theme.primary_bg)),
                        )
                        .highlight_style(Style::default().bg(theme.highlight_bg));
                    f.render_stateful_widget(list, area, &mut list_state); // render_stateful_widgetを使用
                }
                InputMode::Check { prompt, items } => {
                    let prompt_line = Line::from(prompt.as_str());
                    let prompt_paragraph = Paragraph::new(prompt_line).block(mode_block.clone());
                    f.render_widget(prompt_paragraph, area);

                    let items: Vec<ListItem> = items.iter().map(|(name, checked)| {
                        let display_text = if *checked {
                            format!("✓ {}", name)
                        } else {
                            format!("  {}", name)
                        };
                        ListItem::new(display_text)
                    }).collect();
                    let mut list_state = self.state.clone(); // ListStateをクローンして可変にする
                    let list = List::new(items)
                        .highlight_style(Style::default().bg(theme.highlight_bg).fg(theme.text_fg)) // ハイライトスタイルをListに設定
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title("Select Options")
                                .style(Style::default().bg(theme.primary_bg)),
                        );
                    f.render_stateful_widget(list, area, &mut list_state); // render_stateful_widgetを使用
                }
            }
            return; // 通常の候補リストは描画しない
        }

        // 通常のコマンドパレット候補リスト描画
        let max_height = area.height.saturating_sub(2);
        let list_height = self.calc_command_list_height(max_height);

        let list_area = Rect {
            x: area.x,
            y: area.y + 3,
            width: area.width,
            height: list_height,
        };

        let items: Vec<ListItem> = match self.mode { // 手動ハイライトを削除
            PaletteMode::Command => self.command_candidates.iter().map(|c| {
                ListItem::new(c.clone())
            }).collect(),
            PaletteMode::File => self.file_candidates.iter().map(|name| {
                ListItem::new(name.clone())
            }).collect(),
        };

        let mut list_state = self.state.clone(); // ListStateをクローンして可変にする
        let list = List::new(items)
            .highlight_style(Style::default().bg(theme.highlight_bg).fg(theme.text_fg)) // ハイライトスタイルをListに設定
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(match self.mode {
                        PaletteMode::Command => "Commands",
                        PaletteMode::File => "Files",
                    })
                    .style(Style::default().bg(theme.primary_bg)),
            )
            .highlight_style(Style::default().bg(theme.highlight_bg)); // 重複するhighlight_styleを削除
        f.render_stateful_widget(list, list_area, &mut list_state); // render_stateful_widgetを使用
    }
}