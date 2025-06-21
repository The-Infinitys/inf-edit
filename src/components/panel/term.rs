use crate::event_handler::PtyInput;
use anyhow::Result;
use portable_pty::{CommandBuilder, MasterPty, PtySize, native_pty_system};
use std::env;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::thread;
use tui_term::vt100::Parser;
use tui_term::widget::PseudoTerminal;

pub struct Term {
    parser: Arc<Mutex<Parser>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>, // 追加
    _pty: Box<dyn MasterPty + Send>,           // 保持しておくことでdropされないように
    dead: Arc<AtomicBool>,                     // ← 追加
}

impl Default for Term {
    fn default() -> Self {
        Self::new(None).expect("Failed to create default Term") // Or handle error appropriately
    }
}

impl Term {
    pub fn new(cwd: Option<PathBuf>) -> Result<Self> {
        // Changed to anyhow::Result
        // SHELL取得
        let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let pty_system = native_pty_system();
        let pty_pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // シェル起動
        let mut cmd = CommandBuilder::new(shell);
        if let Some(path) = cwd {
            cmd.cwd(path);
        }
        let _child = pty_pair.slave.spawn_command(cmd)?;

        // vt100パーサ
        let parser = Arc::new(Mutex::new(Parser::new(24, 80, 0)));
        let dead = Arc::new(AtomicBool::new(false)); // ← ここで毎回新規

        // PTYからの出力をパーサに流し込むスレッド
        {
            let parser = Arc::clone(&parser);
            let mut reader = pty_pair.master.try_clone_reader()?;
            let dead_clone = dead.clone();
            thread::spawn(move || {
                let mut buf = [0u8; 4096];
                while let Ok(n) = reader.read(&mut buf) {
                    if n == 0 {
                        dead_clone.store(true, Ordering::SeqCst);
                        break;
                    }
                    let mut parser = parser.lock().unwrap();
                    parser.process(&buf[..n]);
                }
            });
        }
        let writer = Arc::new(Mutex::new(pty_pair.master.take_writer()?)); // 追加

        Ok(Self {
            parser,
            writer, // 追加
            _pty: pty_pair.master,
            dead, // ← 追加
        })
    }

    pub fn render_with_block(
        &mut self,
        f: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
        block: ratatui::widgets::Block,
    ) {
        // The area for the terminal content is inside the block's borders.
        let inner_area = block.inner(area);
        let rows = inner_area.height.max(1); // Ensure at least 1 row
        let cols = inner_area.width.max(1);   // Ensure at least 1 col

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
        f.render_widget(pseudo_term, area); // area: このウィジェットのRect
        let (cur_y, cur_x) = parser.screen().cursor_position(); // This is 1-based (y, x)
        let cursor_x = inner_area.x + cur_x;
        let cursor_y = inner_area.y + cur_y;
        f.set_cursor_position((cursor_x, cursor_y));
    }

    /// プロセスが終了しているか
    pub fn is_dead(&self) -> bool {
        self.dead.load(Ordering::SeqCst)
    }
}

impl PtyInput for Term {
    fn send_input(&self, input: &[u8]) {
        if let Ok(mut writer) = self.writer.lock() {
            let _ = writer.write_all(input);
        }
    }
}
