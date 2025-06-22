use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
// Updated import path
use inf_edit::app::App;
use inf_edit::event_handler;
use inf_edit::ui;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

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

        // Poll for events with a timeout to keep the UI responsive and run ticks.
        if event::poll(Duration::from_millis(100))? {
            // If an event is available, handle it.
            match event_handler::handle_events(&mut app)? {
                event_handler::AppEvent::Quit => break,
                event_handler::AppEvent::Continue => {}
            }
        }

        // Run periodic tasks like checking for exited terminals.
        app.tick();
    }

    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}
