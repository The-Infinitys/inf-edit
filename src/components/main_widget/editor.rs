use crate::event_handler::PtyInput;
use portable_pty::{CommandBuilder, MasterPty, PtySize, native_pty_system};
use std::env;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use tui_term::vt100::Parser;
use tui_term::widget::PseudoTerminal;

pub struct Editor {
    parser: Arc<Mutex<Parser>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    dead: Arc<AtomicBool>,
    _pty: Box<dyn MasterPty + Send>, // 保持しておくことでdropされないように
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
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
        let dead = Arc::new(AtomicBool::new(false));

        // PTYからの出力をパーサに流し込むスレッド
        {
            let parser = Arc::clone(&parser);
            let mut reader = pty_pair.master.try_clone_reader().expect("clone reader");
            let dead_clone = dead.clone(); // Clone dead for the thread
            thread::spawn(move || { // Move the cloned dead into the thread
                let mut buf = [0u8; 4096];
                while let Ok(n) = reader.read(&mut buf) {
                    if n == 0 {
                        dead_clone.store(true, Ordering::SeqCst); // Use the cloned dead
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
            dead,
            _pty: pty_pair.master,
        }
    }

    pub fn render_with_block(
        &mut self,
        f: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
        block: ratatui::widgets::Block,
    ) {
        // The area for the terminal content is inside the block's borders.
        let inner_area = block.inner(area);
        let rows = inner_area.height;
        let cols = inner_area.width;

        {
            let mut parser = self.parser.lock().unwrap();
            parser.set_size(rows, cols);
        }
        let _ = self._pty.resize(portable_pty::PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        });

        let parser = self.parser.lock().unwrap();
        let pseudo_term = PseudoTerminal::new(parser.screen()).block(block);
        f.render_widget(pseudo_term, area);
        let (cur_y, cur_x) = parser.screen().cursor_position(); // This is 1-based (y, x)
        let cursor_x = inner_area.x + cur_x - 1;
        let cursor_y = inner_area.y + cur_y - 1;
        f.set_cursor_position((cursor_x, cursor_y));
    }

    /// ファイルを開く
    pub fn open_file(&mut self, path: std::path::PathBuf) {
        // 新しいPTYとプロセスを作成
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
        let pty_system = native_pty_system();
        let pty_pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("Failed to open PTY");

        let mut cmd = CommandBuilder::new(editor);
        cmd.arg(path);
        let _child = pty_pair
            .slave
            .spawn_command(cmd)
            .expect("Failed to spawn editor");

        let parser = Arc::new(Mutex::new(Parser::new(24, 80, 0)));
        let writer = Arc::new(Mutex::new(
            pty_pair.master.take_writer().expect("clone writer"),
        ));
        let dead = Arc::new(AtomicBool::new(false));

        // PTYからの出力をパーサに流し込むスレッド
        {
            let parser = Arc::clone(&parser);
            let mut reader = pty_pair.master.try_clone_reader().expect("clone reader");
            let dead_clone = dead.clone(); // Clone dead for the thread
            thread::spawn(move || { // Move the cloned dead into the thread
                let mut buf = [0u8; 4096];
                while let Ok(n) = reader.read(&mut buf) {
                    if n == 0 {
                        dead_clone.store(true, Ordering::SeqCst); // Use the cloned dead
                        break;
                    }
                    let mut parser = parser.lock().unwrap();
                    parser.process(&buf[..n]);
                }
            });
        }

        // 新しいリソースで上書き
        self.parser = parser;
        self.writer = writer;
        self.dead = dead;
        self._pty = pty_pair.master;
    }

    pub fn is_dead(&self) -> bool {
        self.dead.load(Ordering::SeqCst)
    }
}

impl PtyInput for Editor {
    /// 入力をエディタプロセスに送る
    fn send_input(&self, input: &[u8]) {
        if let Ok(mut writer) = self.writer.lock() {
            let _ = writer.write_all(input);
        }
    }
}
