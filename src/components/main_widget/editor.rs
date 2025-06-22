use crate::{
    components::notification::{send_notification, NotificationType},
    event_handler::PtyInput,
};
use portable_pty::{CommandBuilder, MasterPty, PtySize, native_pty_system};
use std::env;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use tui_term::vt100::Parser;
use tui_term::widget::PseudoTerminal;

pub struct Editor {
    parser: Arc<Mutex<Parser>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    dead: Arc<AtomicBool>,
    _pty: Box<dyn MasterPty + Send>, // 保持しておくことでdropされないように
}

type PtyResources = (
    Arc<Mutex<Parser>>,
    Arc<Mutex<Box<dyn Write + Send>>>,
    Arc<AtomicBool>,
    Box<dyn MasterPty + Send>,
);

/// Helper function to initialize a PTY and spawn an editor process.
fn init_pty(path: Option<std::path::PathBuf>) -> PtyResources {
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

    let mut cmd = CommandBuilder::new(editor);
    if let Some(p) = path {
        cmd.arg(p);
    }
    let _child = pty_pair.slave.spawn_command(cmd).expect("Failed to spawn editor");

    let parser = Arc::new(Mutex::new(Parser::new(24, 80, 0)));
    let writer = Arc::new(Mutex::new(pty_pair.master.take_writer().expect("clone writer")));
    let dead = Arc::new(AtomicBool::new(false));

    // Thread to stream PTY output to the parser
    let parser_clone = Arc::clone(&parser);
    let mut reader = pty_pair.master.try_clone_reader().expect("clone reader");
    let dead_clone = dead.clone();
    thread::spawn(move || {
        let mut buf = [0u8; 4096];
        while let Ok(n) = reader.read(&mut buf) {
            if n == 0 {
                dead_clone.store(true, Ordering::SeqCst);
                break;
            }
            parser_clone.lock().unwrap().process(&buf[..n]);
        }
    });

    (parser, writer, dead, pty_pair.master)
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

impl Editor {
    pub fn new() -> Self {
        let (parser, writer, dead, _pty) = init_pty(None);
        Self {
            parser,
            writer,
            dead,
            _pty,
        }
    }

    pub fn with_file(path: std::path::PathBuf) -> Self {
        let (parser, writer, dead, _pty) = init_pty(Some(path));
        Self {
            parser,
            writer,
            dead,
            _pty,
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
        f.render_widget(pseudo_term, area);
        let (cur_y, cur_x) = parser.screen().cursor_position(); // This is 1-based (y, x)
        let cursor_x = inner_area.x + cur_x;
        let cursor_y = inner_area.y + cur_y.saturating_sub(1);
        f.set_cursor_position((cursor_x, cursor_y));
    }

    pub fn is_dead(&self) -> bool {
        self.dead.load(Ordering::SeqCst)
    }

    /// Sends the save command to the underlying editor process.
    /// NOTE: This is brittle as it assumes a vi-like editor and that it's in normal mode.
    /// A more robust implementation would require more complex state tracking or a different
    /// approach to editing files (e.g., managing the buffer directly in `inf-edit`).
    pub fn save(&mut self) -> std::io::Result<()> {
        // Send Esc, then :w, then Enter.
        let save_command = b"\x1b:w\r";
        if let Ok(mut writer) = self.writer.lock() {
            writer.write_all(save_command)?;
            writer.flush()?;
            send_notification(
                "Save command sent to editor.".to_string(),
                NotificationType::Info,
            );
            Ok(())
        } else {
            Err(std::io::Error::other(
                "Could not lock editor writer to save",
            ))
        }
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
