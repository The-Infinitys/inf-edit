use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
};
use std::{env, rc::Rc};
use std::{io, time::Duration};

mod components {
    pub mod editor;
    pub mod file_view;
    pub mod term;
}
use components::{editor::Editor, file_view::FileView, term::Term};
use inf_edit::components::status::StatusBar;
enum ActiveTarget {
    Editor,
    Term,
    FileView,
}

struct App {
    show_file_view: bool,
    show_term: bool,
    active_target: ActiveTarget,
    term: Term,
}

impl App {
    fn new() -> Self {
        Self {
            show_file_view: true,
            show_term: false,
            active_target: ActiveTarget::Editor,
            term: Term::new(), // Initialize Term instance here
        }
    }
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    // let mut term = Term::new(); // Term is now part of App
    let mut editor = Editor::new();
    let mut f_view = FileView::new(env::current_dir()?);
    let status = StatusBar::new("Ctrl+Q:終了 Ctrl+B:ファイルビュー Ctrl+J:ターミナル ...");
    loop {
        terminal.draw(|f| {
            // let size = f.area();

            // レイアウトの定義
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),           // メイン画面
                    Constraint::Length(1),        // ステータスバー
                ])
                .split(f.area());
            let main_area = layout[0];
            let status_area = layout[1];

            // 左側: file_view, 右側: editor(上), term(下)
            let chunks = if app.show_file_view {
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(30), Constraint::Min(1)].as_ref())
                    .split(main_area)
            } else {
                Rc::from(vec![Rect::new(0, 0, 0, 0), main_area])
            };

            // 右側を上下分割
            let right_chunks = if app.show_term {
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
            let editor_block = ratatui::widgets::Block::default()
                .title("Editor")
                .borders(ratatui::widgets::Borders::ALL)
                .border_style(if matches!(app.active_target, ActiveTarget::Editor) {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                });
            editor.render_with_block(f, right_chunks[0], editor_block);

            // term
            if app.show_term {
                let term_block = ratatui::widgets::Block::default()
                    .title("Terminal")
                    .borders(ratatui::widgets::Borders::ALL)
                    .border_style(if matches!(app.active_target, ActiveTarget::Term) {
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    });
                app.term.render_with_block(f, right_chunks[1], term_block);
            }

            // ステータスバーの描画
            status.render(f, status_area);
        })?;

        // イベント処理
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // ctrl+q で終了
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('q') {
                    break;
                }
                // ctrl+c で終了（不要なら削除可）
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c') {
                    break;
                }
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('b') {
                    app.show_file_view = !app.show_file_view;
                    // file_viewを開いたときはFileViewをアクティブに、閉じたらEditorをアクティブに
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
                    if app.show_term { // If terminal is currently shown, hide it
                        app.show_term = false;
                        app.active_target = ActiveTarget::Editor;
                    } else { // If terminal is currently hidden, show it
                        // If the existing terminal process is dead, create a new one.
                        if app.term.is_dead() {
                            app.term = Term::new();
                        }
                        app.show_term = true;
                        app.active_target = ActiveTarget::Term;
                    }
                    continue;
                }
                // ctrl+k でターゲット切り替え（ターミナルが開いているときのみ）
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('k') {
                    if app.show_term {
                        app.active_target = match app.active_target {
                            ActiveTarget::Editor => ActiveTarget::Term,
                            ActiveTarget::Term => ActiveTarget::Editor,
                            ActiveTarget::FileView => ActiveTarget::FileView,
                        };
                    }
                    continue;
                }

                // アクティブなターゲットに応じてキー入力を送信
                match app.active_target {
                    ActiveTarget::Editor => match key.code {
                        KeyCode::Char(c) => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                let ctrl = (c as u8) & 0x1f;
                                editor.send_input(&[ctrl]);
                            } else {
                                editor.send_input(c.to_string().as_bytes());
                            }
                        }
                        KeyCode::Enter => editor.send_input(b"\n"),
                        KeyCode::Tab => editor.send_input(b"\t"),
                        KeyCode::Backspace => editor.send_input(&[8]),
                        KeyCode::Left => editor.send_input(b"\x1b[D"),
                        KeyCode::Right => editor.send_input(b"\x1b[C"),
                        KeyCode::Up => editor.send_input(b"\x1b[A"),
                        KeyCode::Down => editor.send_input(b"\x1b[B"),
                        KeyCode::Esc => editor.send_input(b"\x1b"),
                        _ => {}
                    },
                    ActiveTarget::Term => match key.code {
                        KeyCode::Char(c) => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                let ctrl = (c as u8) & 0x1f;
                                app.term.send_input(&[ctrl]);
                            } else {
                                app.term.send_input(c.to_string().as_bytes());
                            }
                        }
                        KeyCode::Enter => app.term.send_input(b"\n"),
                        KeyCode::Tab => app.term.send_input(b"\t"),
                        KeyCode::Backspace => app.term.send_input(&[8]),
                        KeyCode::Left => app.term.send_input(b"\x1b[D"),
                        KeyCode::Right => app.term.send_input(b"\x1b[C"),
                        KeyCode::Up => app.term.send_input(b"\x1b[A"),
                        KeyCode::Down => app.term.send_input(b"\x1b[B"),
                        KeyCode::Esc => app.term.send_input(b"\x1b"),
                        _ => {}
                    },
                    ActiveTarget::FileView => {
                        // ファイルビューの操作例
                        match key.code {
                            KeyCode::Down | KeyCode::Char('j') => f_view.next(),
                            KeyCode::Up | KeyCode::Char('k') => f_view.previous(),
                            KeyCode::Enter => {
                                if let Some(file) = f_view.selected_file() {
                                    // editorでファイルを開く
                                    editor.open_file(file);
                                    app.active_target = ActiveTarget::Editor;
                                } else {
                                    f_view.enter();
                                }
                            }
                            KeyCode::Backspace | KeyCode::Char('h') => f_view.back(),
                            _ => {}
                        }
                    }
                }
            }
        }

        // ターミナルプロセスの終了監視
        if app.show_term {
            if app.term.is_dead() {
                app.show_term = false;
                app.active_target = ActiveTarget::Editor;
            }
        }
    }

    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}
