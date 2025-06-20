use portable_pty::{CommandBuilder, MasterPty, PtySize, native_pty_system};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders},
};
use std::env;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use tui_term::vt100::Parser;
use tui_term::widget::PseudoTerminal;

pub struct Editor {
    parser: Arc<Mutex<Parser>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    _pty: Box<dyn MasterPty + Send>, // 保持しておくことでdropされないように
}

impl Editor {
    pub fn new() -> Self {
        let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
        let pty_system = native_pty_system();
        let pty_pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("Failed to open PTY");

        let cmd = CommandBuilder::new(editor);
        let _child = pty_pair
            .slave
            .spawn_command(cmd)
            .expect("Failed to spawn editor");

        let parser = Arc::new(Mutex::new(Parser::new(24, 80, 0)));

        // ライターを保持
        let writer = Arc::new(Mutex::new(
            pty_pair.master.take_writer().expect("clone writer"),
        ));

        // PTYからの出力をパーサに流し込むスレッド
        {
            let parser = Arc::clone(&parser);
            let mut reader = pty_pair.master.try_clone_reader().expect("clone reader");
            thread::spawn(move || {
                let mut buf = [0u8; 4096];
                while let Ok(n) = reader.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    let mut parser = parser.lock().unwrap();
                    parser.process(&buf[..n]);
                }
            });
        }

        Self {
            parser,
            writer,
            _pty: pty_pair.master,
        }
    }

    /// 入力をエディタプロセスに送る
    pub fn send_input(&self, input: &[u8]) {
        if let Ok(mut writer) = self.writer.lock() {
            let _ = writer.write_all(input);
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        // areaのサイズに合わせてパーサとPTYをリサイズ
        let rows = area.height.saturating_sub(2).max(1);
        let cols = area.width.saturating_sub(2).max(1);

        {
            let mut parser = self.parser.lock().unwrap();
            parser.set_size(rows as u16, cols as u16);
        }
        // PTYのリサイズ
        let _ = self._pty.resize(portable_pty::PtySize {
            rows: rows as u16,
            cols: cols as u16,
            pixel_width: 0,
            pixel_height: 0,
        });

        let parser = self.parser.lock().unwrap();
        let pseudo_term = PseudoTerminal::new(parser.screen())
            .block(Block::default().title("Editor").borders(Borders::ALL))
            .style(
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_widget(pseudo_term, area);
    }
    pub fn render_with_block(
        &mut self,
        f: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
        block: ratatui::widgets::Block,
    ) {
        let rows = area.height.saturating_sub(2).max(1);
        let cols = area.width.max(1);

        {
            let mut parser = self.parser.lock().unwrap();
            parser.set_size(rows as u16, cols as u16);
        }
        let _ = self._pty.resize(portable_pty::PtySize {
            rows: rows as u16,
            cols: cols as u16,
            pixel_width: 0,
            pixel_height: 0,
        });

        let parser = self.parser.lock().unwrap();
        let pseudo_term = PseudoTerminal::new(parser.screen()).block(block);
        f.render_widget(pseudo_term, area);
        let (cur_y, cur_x) = parser.screen().cursor_position(); // vt100は(1,1)始まり
        let cursor_x = area.x + cur_x.saturating_sub(1);
        let cursor_y = area.y + cur_y.saturating_sub(1);
        f.set_cursor_position((cursor_x, cursor_y));
    }
}
