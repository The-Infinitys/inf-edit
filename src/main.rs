use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
};
use std::time::Duration;
use std::{env, io, rc::Rc};

mod components {
    pub mod editor;
    pub mod file_view;
    pub mod term;
}
use components::{editor::Editor, file_view::FileView, term::Term};
use inf_edit::components::status::StatusBar;

#[derive(PartialEq)]
enum ActiveTarget {
    Editor,
    Term,
    FileView,
}

struct Tab<T> {
    content: T,
    title: String,
}

struct App {
    show_file_view: bool,
    show_term: bool,
    active_target: ActiveTarget,
    editors: Vec<Tab<Editor>>,
    terminals: Vec<Tab<Term>>,
    active_editor_tab: usize,
    active_terminal_tab: usize,
}

impl App {
    fn new() -> Self {
        Self {
            show_file_view: true,
            show_term: false,
            active_target: ActiveTarget::Editor,
            editors: vec![Tab {
                content: Editor::new(),
                title: "Editor 1".to_string(),
            }],
            terminals: vec![],
            active_editor_tab: 0,
            active_terminal_tab: 0,
        }
    }
}

fn main() -> Result<()> {
    // Changed to anyhow::Result
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let mut f_view = FileView::new(env::current_dir()?);
    let status = StatusBar::new(
        "Ctrl+Q:終了 Ctrl+B:ファイルビュー Ctrl+J:ターミナル Ctrl+N:新規エディタタブ Ctrl+T:タブ切替 ...",
    );
    loop {
        terminal.draw(|f| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(2), // タブバー用 高さを2に変更
                    Constraint::Min(1),
                    Constraint::Length(1),
                ])
                .split(f.area());
            let tabbar_area = layout[0];
            let main_area = layout[1];
            let status_area = layout[2];

            // タブバーの描画
            use ratatui::text::{Line, Span};
            use ratatui::widgets::{Block, Borders, Tabs}; // ← Spans ではなく Line を使う

            // EditorタブとTerminalタブのタイトルを作成
            let editor_titles: Vec<Line> = app
                .editors
                .iter()
                .enumerate()
                .map(|(i, tab)| {
                    let mut title = tab.title.clone();
                    if i == app.active_editor_tab && app.active_target == ActiveTarget::Editor {
                        title = format!("*{}", title);
                    }
                    Line::from(Span::raw(title))
                })
                .collect();

            let terminal_titles: Vec<Line> = app
                .terminals
                .iter()
                .enumerate()
                .map(|(i, tab)| {
                    let mut title = tab.title.clone();
                    if i == app.active_terminal_tab && app.active_target == ActiveTarget::Term {
                        title = format!("*{}", title);
                    }
                    Line::from(Span::raw(title))
                })
                .collect();

            // Tabsウィジェットでタブバーを描画
            let mut all_titles = editor_titles;
            if !terminal_titles.is_empty() {
                all_titles.push(Line::from(Span::raw(" | ")));
                all_titles.extend(terminal_titles);
            }
            let tabs = Tabs::new(all_titles)
                .block(Block::default().borders(Borders::BOTTOM).title("Tabs"))
                .highlight_style(ratatui::style::Style::default().fg(ratatui::style::Color::Green));
            f.render_widget(tabs, tabbar_area);

            let chunks = if app.show_file_view {
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(30), Constraint::Min(1)].as_ref())
                    .split(main_area)
            } else {
                Rc::from(vec![Rect::new(0, 0, 0, 0), main_area])
            };

            let right_chunks = if app.show_term
                && !app.terminals.is_empty()
                && app.active_target == ActiveTarget::Term
            {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                    .split(chunks[1])
            } else {
                Rc::from([chunks[1], Rect::new(0, 0, 0, 0)])
            };

            // file_view
            if app.show_file_view {
                f_view.render(
                    f,
                    chunks[0],
                    matches!(app.active_target, ActiveTarget::FileView),
                );
            }

            // editor
            if !app.editors.is_empty() {
                let editor_block = ratatui::widgets::Block::default();
                if let Some(active_editor_tab) = app.editors.get_mut(app.active_editor_tab) {
                    active_editor_tab
                        .content
                        .render_with_block(f, right_chunks[0], editor_block);
                }
            }

            // term
            if app.show_term && !app.terminals.is_empty() {
                if let Some(active_terminal_tab) = app.terminals.get_mut(app.active_terminal_tab) {
                    let term_block = ratatui::widgets::Block::default();
                    active_terminal_tab
                        .content
                        .render_with_block(f, right_chunks[1], term_block);
                }
            }

            // Render the static status bar message
            status.render(f, status_area);
        })?;

        // イベント処理
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // ctrl+q で終了
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('q') {
                    break;
                }
                // ctrl+c で終了
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c') {
                    break;
                }
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('b') {
                    app.show_file_view = !app.show_file_view;
                    app.active_target = if app.show_file_view {
                        ActiveTarget::FileView
                    } else if app.show_term {
                        ActiveTarget::Term
                    } else {
                        ActiveTarget::Editor
                    };
                    continue;
                }
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('j') {
                    if app.active_target == ActiveTarget::Term {
                        app.show_term = false;
                        app.active_target = ActiveTarget::Editor;
                    } else {
                        if app.terminals.is_empty() {
                            app.terminals.push(Tab {
                                content: Term::new()?,
                                title: format!("Term {}", app.terminals.len() + 1),
                            }); // エラー伝播を追加
                            app.active_terminal_tab = app.terminals.len() - 1;
                        } else {
                            if app.terminals[app.active_terminal_tab].content.is_dead() {
                                app.terminals[app.active_terminal_tab].content = Term::new()?; // エラー伝播を追加
                            }
                            app.active_terminal_tab = app
                                .active_terminal_tab
                                .min(app.terminals.len().saturating_sub(1));
                        }
                        app.show_term = true;
                        app.active_target = ActiveTarget::Term;
                    }
                    continue;
                }
                // ctrl+k でターゲット切り替え（ターミナルが開いているときのみ）
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('k') {
                    if app.show_term {
                        match app.active_target {
                            ActiveTarget::Editor => {
                                if !app.terminals.is_empty() {
                                    app.active_target = ActiveTarget::Term;
                                }
                            }
                            ActiveTarget::Term => {
                                if !app.editors.is_empty() {
                                    app.active_target = ActiveTarget::Editor;
                                }
                            }
                            ActiveTarget::FileView => {
                                // If in FileView, prioritize switching to Editor, then Term
                                if !app.editors.is_empty() {
                                    app.active_target = ActiveTarget::Editor;
                                } else if !app.terminals.is_empty() {
                                    app.active_target = ActiveTarget::Term;
                                }
                            }
                        }
                    }
                    continue;
                }
                // Ctrl+N for new editor tab
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('n') {
                    app.editors.push(Tab {
                        content: Editor::new(),
                        title: format!("Editor {}", app.editors.len() + 1),
                    });
                    app.active_editor_tab = app.editors.len() - 1;
                    app.active_target = ActiveTarget::Editor;
                    app.show_term = false;
                    continue;
                }
                // Ctrl+T to switch tabs
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('t') {
                    if app.active_target == ActiveTarget::Editor && !app.editors.is_empty() {
                        app.active_editor_tab = (app.active_editor_tab + 1) % app.editors.len();
                    } else if app.active_target == ActiveTarget::Term && !app.terminals.is_empty() {
                        app.active_terminal_tab =
                            (app.active_terminal_tab + 1) % app.terminals.len();
                    }
                    continue;
                }

                // アクティブなターゲットに応じてキー入力を送信
                match app.active_target {
                    ActiveTarget::Editor => {
                        if let Some(active_editor_tab) = app.editors.get_mut(app.active_editor_tab)
                        {
                            match key.code {
                                KeyCode::Char(c) => {
                                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                                        let ctrl = (c as u8) & 0x1f;
                                        active_editor_tab.content.send_input(&[ctrl]);
                                    } else {
                                        active_editor_tab
                                            .content
                                            .send_input(c.to_string().as_bytes());
                                    }
                                }
                                KeyCode::Enter => active_editor_tab.content.send_input(b"\n"),
                                KeyCode::Tab => active_editor_tab.content.send_input(b"\t"),
                                KeyCode::Backspace => active_editor_tab.content.send_input(&[8]),
                                KeyCode::Left => active_editor_tab.content.send_input(b"\x1b[D"),
                                KeyCode::Right => active_editor_tab.content.send_input(b"\x1b[C"),
                                KeyCode::Up => active_editor_tab.content.send_input(b"\x1b[A"),
                                KeyCode::Down => active_editor_tab.content.send_input(b"\x1b[B"),
                                KeyCode::Esc => active_editor_tab.content.send_input(b"\x1b"),
                                _ => {}
                            }
                        }
                    }
                    ActiveTarget::Term => {
                        if let Some(active_terminal_tab) =
                            app.terminals.get_mut(app.active_terminal_tab)
                        {
                            match key.code {
                                KeyCode::Char(c) => {
                                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                                        let ctrl = (c as u8) & 0x1f;
                                        active_terminal_tab.content.send_input(&[ctrl]);
                                    } else {
                                        active_terminal_tab
                                            .content
                                            .send_input(c.to_string().as_bytes());
                                    }
                                }
                                KeyCode::Enter => active_terminal_tab.content.send_input(b"\n"),
                                KeyCode::Tab => active_terminal_tab.content.send_input(b"\t"),
                                KeyCode::Backspace => active_terminal_tab.content.send_input(&[8]),
                                KeyCode::Left => active_terminal_tab.content.send_input(b"\x1b[D"),
                                KeyCode::Right => active_terminal_tab.content.send_input(b"\x1b[C"),
                                KeyCode::Up => active_terminal_tab.content.send_input(b"\x1b[A"),
                                KeyCode::Down => active_terminal_tab.content.send_input(b"\x1b[B"),
                                KeyCode::Esc => active_terminal_tab.content.send_input(b"\x1b"),
                                _ => {}
                            }
                        }
                    }
                    ActiveTarget::FileView => match key.code {
                        KeyCode::Down | KeyCode::Char('j') => f_view.next(),
                        KeyCode::Up | KeyCode::Char('k') => f_view.previous(),
                        KeyCode::Enter => {
                            if let Some(file) = f_view.selected_file() {
                                if let Some(active_editor_tab) =
                                    app.editors.get_mut(app.active_editor_tab)
                                {
                                    active_editor_tab.content.open_file(file);
                                }
                                app.active_target = ActiveTarget::Editor;
                            } else {
                                f_view.enter();
                            }
                        }
                        KeyCode::Backspace | KeyCode::Char('h') => f_view.back(),
                        _ => {}
                    },
                }
            }
        }

        // ターミナルプロセスの終了監視
        if app.active_target == ActiveTarget::Term && !app.terminals.is_empty() {
            if app.terminals[app.active_terminal_tab].content.is_dead() {
                app.active_target = ActiveTarget::Editor;
            }
        }
    }

    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}
