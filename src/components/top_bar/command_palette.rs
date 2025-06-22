use crate::theme::Theme;
use crate::{
    app::App,
    components::notification::{send_notification, NotificationType},
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};
use std::env::current_dir;
use std::path::PathBuf;
use std::{
    borrow::Cow,
    path::Path,
    sync::{mpsc, Arc},
};
use walkdir::WalkDir;

/// アプリケーションの状態を変更するためのアクションを定義します。
/// `Arc`でラップすることで、複数のコマンドアイテムで共有可能になります。
pub type Action = Arc<dyn Fn(&mut App) + Send + Sync>;

/// コマンドパレットに表示されるアイテム。コマンドまたはファイルパスを表します。
pub enum CommandItem {
    Command { name: String, action: Action },
    File { name: String, path: String },
}

impl CommandItem {
    /// リストに表示するためのテキストを返します。
    /// ファイルの場合は「ファイル名 <タブ> ディレクトリパス」の形式で表示します。
    fn display_text(&self) -> Cow<str> {
        match self {
            CommandItem::Command { name, .. } => Cow::Borrowed(name),
            CommandItem::File { name, path } => {
                let p = Path::new(path);
                let parent_dir = p
                    .parent()
                    .unwrap_or_else(|| Path::new(""))
                    .to_string_lossy();
                Cow::Owned(format!("{}: {}", name, parent_dir))
            }
        }
    }

    // アイテムが表すファイルパスを返します（ファイルアイテムの場合のみ）。
    // fn file_path(&self) -> Option<&str> {
    //     match self {
    //         CommandItem::File { path, .. } => Some(path),
    //         _ => None,
    //     }
    // }
}

/// パレットのモード。コマンド検索かファイル検索かを切り替えます。
enum PaletteMode {
    Command,
    File,
}

/// Events that the command palette can emit to the main application.
pub enum CommandPaletteEvent {
    Execute,
    Close,
    None,
}

pub struct CommandPalette {
    input: String,
    commands: Vec<CommandItem>,
    files: Vec<CommandItem>,
    filtered_indices: Vec<usize>,
    list_state: ListState,
    mode: PaletteMode,
    is_searching: bool,
    file_view_changed: bool,
    file_receiver: Option<mpsc::Receiver<CommandItem>>,
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPalette {
    pub fn new() -> Self {
        let mut s = Self {
            input: String::new(),
            commands: Self::load_commands(),
            files: Vec::new(),
            filtered_indices: Vec::new(),
            list_state: ListState::default(),
            mode: PaletteMode::File, // デフォルトをファイル検索モードに変更
            is_searching: false,
            file_view_changed: true, // Start dirty to trigger initial scan
            file_receiver: None,
        };
        // Don't start search on creation, wait until palette is opened.
        s.filter_items();
        s
    }

    /// アプリケーションで利用可能な全コマンドを定義します。
    fn load_commands() -> Vec<CommandItem> {
        vec![
            CommandItem::Command {
                name: "File: Open...".to_string(),
                action: Arc::new(|app| app.command_palette.enter_file_mode()),
            },
            CommandItem::Command {
                name: "File: Save".to_string(),
                action: Arc::new(|app| {
                    if let Some(editor) = app.get_active_editor_mut() {
                        // This requires you to implement the `save` method on your Editor struct.
                        if let Err(e) = editor.save() {
                            send_notification(
                                format!("Error saving file: {}", e),
                                NotificationType::Error,
                            );
                        }
                    }
                }),
            },
            CommandItem::Command {
                name: "Settings: Open".to_string(),
                action: Arc::new(|app| app.add_settings_tab()),
            },
            CommandItem::Command {
                name: "View: Toggle Primary Sidebar".to_string(),
                action: Arc::new(|app| app.toggle_primary_sidebar()),
            },
            CommandItem::Command {
                name: "View: Toggle Panel".to_string(),
                action: Arc::new(|app| app.toggle_panel()),
            },
            CommandItem::Command {
                name: "Terminal: Open New".to_string(),
                action: Arc::new(|app| app.open_new_terminal()),
            },
            CommandItem::Command {
                name: "Application: Quit".to_string(),
                action: Arc::new(|app| app.show_quit_popup()),
            },
        ]
    }

    /// ファイル検索モードに切り替え、ファイルリストを（必要なら）読み込みます。
    pub fn enter_file_mode(&mut self) {
        self.mode = PaletteMode::File;
        self.input.clear();
        // Only start a new search if the file view has changed, or if there are no files and no search is running.
        if self.file_view_changed || (self.files.is_empty() && !self.is_searching) {
            self.files.clear(); // Clear stale results
            let (tx, rx) = mpsc::channel();
            self.file_receiver = Some(rx);
            self.is_searching = true;

            std::thread::spawn(move || {
                for entry in WalkDir::new(".")
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file())
                {
                    let path_str = entry.path().to_string_lossy();
                    // A more robust ignore mechanism would be better (e.g., using .gitignore)
                    if path_str.contains(".git") || path_str.contains("target") {
                        continue;
                    }
                    let name = entry.file_name().to_string_lossy().to_string();
                    let path = path_str.to_string();
                    if tx.send(CommandItem::File { name, path }).is_err() {
                        // Receiver has been dropped, so the palette was closed.
                        break; // Receiver has been dropped
                    }
                }
            });
        }
        self.filter_items();
    }

    /// 入力に基づいてアイテムをフィルタリングします。
    fn filter_items(&mut self) {
        let (source_items, filter_text) = match self.mode {
            PaletteMode::Command => {
                // コマンドモードでは、">" の後のテキストでフィルタリングする
                let filter = self.input.strip_prefix('>').unwrap_or(&self.input);
                (&self.commands, filter)
            }
            PaletteMode::File => (&self.files, self.input.as_str()),
        };
        let input_lower = filter_text.to_lowercase();

        self.filtered_indices = source_items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                let text_to_search = match item {
                    CommandItem::Command { name, .. } => name.as_str(), // コマンド名で検索
                    CommandItem::File { path, .. } => path.as_str(),
                };
                text_to_search.to_lowercase().contains(&input_lower)
            })
            .map(|(i, _)| i)
            .collect();

        if !self.filtered_indices.is_empty() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }
    }

    /// キー入力を処理します。
    pub fn handle_key(&mut self, key: KeyEvent) -> CommandPaletteEvent {
        match key.code {
            KeyCode::Esc => return CommandPaletteEvent::Close,
            KeyCode::Enter => return CommandPaletteEvent::Execute,
            KeyCode::Char(c) => {
                self.input.push(c);
                // 入力が ">" で始まっていればコマンドモードに切り替える
                if self.input.starts_with('>') {
                    self.mode = PaletteMode::Command;
                }
                self.filter_items();
            }
            KeyCode::Backspace => {
                // If input is empty, backspace closes the palette.
                if self.input.is_empty() {
                    return CommandPaletteEvent::Close;
                }
                self.input.pop();
                // 入力が ">" で始まらなくなったら（または空になったら）ファイルモードに戻す
                if !self.input.starts_with('>') {
                    self.mode = PaletteMode::File;
                }
                self.filter_items();
            }
            KeyCode::Down => self.select_next(),
            KeyCode::Up => self.select_previous(),
            _ => {}
        }
        CommandPaletteEvent::None
    }

    /// 選択されているアイテムのアクションを返します。
    pub fn get_selected_action(&self) -> Option<Action> {
        self.list_state.selected().and_then(|selected_idx| {
            self.filtered_indices.get(selected_idx).map(|&item_idx| {
                let item = match self.mode {
                    PaletteMode::Command => &self.commands[item_idx],
                    PaletteMode::File => &self.files[item_idx],
                };
                match item {
                    CommandItem::Command { action, .. } => action.clone(),
                    CommandItem::File { path, .. } => {
                        let path_clone = path.clone();
                        Arc::new(move |app: &mut App| {
                            app.open_editor(
                                current_dir()
                                    .unwrap_or(PathBuf::from("./"))
                                    .join(&path_clone).canonicalize().unwrap_or_default()
                                    .as_path(),
                            );
                        })
                    }
                }
            })
        })
    }

    /// パレットの状態を初期状態（コマンドモード）にリセットします。
    pub fn reset(&mut self) {
        self.mode = PaletteMode::File; // Reset to file mode
        self.input.clear();
        self.filter_items();
    }

    /// Marks the file list as dirty, forcing a re-scan on the next opportunity.
    pub fn set_file_view_changed(&mut self) {
        self.file_view_changed = true;
    }

    /// Polls for asynchronously found files and updates the list.
    pub fn poll_files(&mut self) {
        let mut received_any = false;
        let mut disconnected = false;

        if let Some(rx) = &self.file_receiver {
            while let Ok(file_item) = rx.try_recv() {
                self.files.push(file_item);
                received_any = true;
            }
            // If the channel is disconnected, the search is over.
            if let Err(mpsc::TryRecvError::Disconnected) = rx.try_recv() {
                disconnected = true;
            }
        }

        if disconnected {
            self.is_searching = false;
            self.file_receiver = None;
        }

        if received_any {
            self.filter_items();
        }
    }

    fn select_next(&mut self) {
        let len = self.filtered_indices.len();
        if len == 0 {
            return;
        }
        let i = self.list_state.selected().map_or(0, |i| (i + 1) % len);
        self.list_state.select(Some(i));
    }

    fn select_previous(&mut self) {
        let len = self.filtered_indices.len();
        if len == 0 {
            return;
        }
        let i = self
            .list_state
            .selected()
            .map_or(0, |i| if i == 0 { len - 1 } else { i - 1 });
        self.list_state.select(Some(i));
    }

    /// コマンドパレットを描画します。
    pub fn render(&mut self, f: &mut Frame, area: Rect, theme: &Theme) {
        let palette_height = 20.min(area.height.saturating_sub(5));
        let area = centered_rect(60, palette_height, area);

        f.render_widget(Clear, area); // 背景をクリア

        let source_items = match self.mode {
            PaletteMode::Command => &self.commands,
            PaletteMode::File => &self.files,
        };

        let list_items: Vec<ListItem> = self
            .filtered_indices
            .iter()
            .map(|&i| ListItem::new(source_items[i].display_text().into_owned()))
            .collect();

        // If searching, show a loading indicator until the first results arrive.
        if self.is_searching && list_items.is_empty() {
            let loading_text = "Searching for files...";
            let loading_para = Paragraph::new(loading_text).alignment(Alignment::Center);
            let block = Block::default().title("File Palette").borders(Borders::ALL);
            f.render_widget(loading_para.block(block), area);
            return;
        }

        let title = match self.mode {
            PaletteMode::Command => "Command",
            PaletteMode::File => "File",
        };

        let list = List::new(list_items)
            .block(
                Block::default()
                    .title(format!(" {} Palette > {} ", title, self.input))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.highlight_fg)),
            )
            .highlight_style(Style::default().bg(theme.highlight_bg))
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, area, &mut self.list_state);
    }
}

/// 中央に配置する矩形を計算するヘルパー関数
fn centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - 90) / 2),
            Constraint::Length(height),
            Constraint::Percentage((100 - 90) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
