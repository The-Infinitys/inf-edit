use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    Terminal,
};
use std::{io, time::Duration};
use std::rc::Rc; // 追加

mod components {
    pub mod file_view;
    pub mod editor;
    pub mod term;
}
use components::{editor::Editor, file_view::FileView, term::Term};

struct App {
    show_file_view: bool,
    show_term: bool,
    // 必要に応じて他の状態も追加
}

impl App {
    fn new() -> Self {
        Self {
            show_file_view: true,
            show_term: false,
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
    let mut term=Term::new();
    let mut editor=Editor::new();
    loop {
        terminal.draw(|f| {
            let size = f.area();

            // 左側: file_view, 右側: editor(上), term(下)
            let chunks = if app.show_file_view {
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(30), Constraint::Min(1)].as_ref())
                    .split(size)
            } else {
                Rc::from(vec![Rect::new(0, 0, 0, 0), size])
            };

            // 右側を上下分割
            let right_chunks = if app.show_term {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(10), Constraint::Length(10)].as_ref())
                    .split(chunks[1])
            } else {
               Rc::from([chunks[1], Rect::new(0, 0, 0, 0)])
            };

            // file_view
            if app.show_file_view {
                FileView::render(f, chunks[0]);
            }
            // editor
            editor.render(f, right_chunks[0]);
            // term
            if app.show_term {
                term.render(f, right_chunks[1]);
            }
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
                }
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('j') {
                    app.show_term = !app.show_term;
                }

                // エディタがアクティブな場合
                let editor_is_active = true; // ここは実際のアクティブ状態に応じて変更
                if editor_is_active {
                    match key.code {
                        KeyCode::Char(c) => {
                            // 日本語なども含めUTF-8で送る
                            editor.send_input(c.to_string().as_bytes());
                        }
                        KeyCode::Enter => editor.send_input(b"\n"),
                        KeyCode::Tab => editor.send_input(b"\t"),
                        KeyCode::Backspace => editor.send_input(&[8]),
                        KeyCode::Left => editor.send_input(b"\x1b[D"),
                        KeyCode::Right => editor.send_input(b"\x1b[C"),
                        KeyCode::Up => editor.send_input(b"\x1b[A"),
                        KeyCode::Down => editor.send_input(b"\x1b[B"),
                        KeyCode::Esc => editor.send_input(b"\x1b"),
                        // 必要に応じて他のキーも追加
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        std::io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}