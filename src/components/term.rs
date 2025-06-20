use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders},
};
use tui_term::widget::PseudoTerminal;
use tui_term::vt100::Parser;
use portable_pty::{CommandBuilder, native_pty_system, PtySize, MasterPty};
use std::env;
use std::io::{Read};
use std::thread;
use std::sync::{Arc, Mutex};

pub struct Term {
    parser: Arc<Mutex<Parser>>,
    _pty: Box<dyn MasterPty + Send>, // 保持しておくことでdropされないように
}

impl Term {
    pub fn new() -> Self {
        // SHELL取得
        let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let pty_system = native_pty_system();
        let pty_pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        }).expect("Failed to open PTY");

        // シェル起動
        let cmd = CommandBuilder::new(shell);
        let _child = pty_pair.slave.spawn_command(cmd).expect("Failed to spawn shell");

        // vt100パーサ
        let parser = Arc::new(Mutex::new(Parser::new(24, 80, 0)));

        // PTYからの出力をパーサに流し込むスレッド
        {
            let parser = Arc::clone(&parser);
            let mut reader = pty_pair.master.try_clone_reader().expect("clone reader");
            thread::spawn(move || {
                let mut buf = [0u8; 4096];
                while let Ok(n) = reader.read(&mut buf) {
                    if n == 0 { break; }
                    let mut parser = parser.lock().unwrap();
                    parser.process(&buf[..n]);
                }
            });
        }

        Self {
            parser,
            _pty: pty_pair.master,
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let parser = self.parser.lock().unwrap();
        let pseudo_term = PseudoTerminal::new(parser.screen())
            .block(
                Block::default()
                    .title("Terminal")
                    .borders(Borders::ALL),
            )
            .style(
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_widget(pseudo_term, area);
    }
}