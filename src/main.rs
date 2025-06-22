use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
// Updated import path
use inf_edit::app::App;
use inf_edit::event_handler;
use inf_edit::ui;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

fn main() -> Result<()> {
    // Changed to anyhow::Result
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;
    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        // イベント処理
        match event_handler::handle_events(&mut app)? {
            event_handler::AppEvent::Quit => break,
            event_handler::AppEvent::Continue => {}
        }
    }

    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}
