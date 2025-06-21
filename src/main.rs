use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
// Updated import path
use inf_edit::event_handler;
use inf_edit::ui;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{env, io};

use inf_edit::ActiveTarget;
use inf_edit::components::primary_sidebar::FileView;
use inf_edit::components::status::StatusBar;

use inf_edit::app::App;

fn main() -> Result<()> {
    // Changed to anyhow::Result
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let mut f_view = FileView::new(env::current_dir()?);
    let status_bar = StatusBar::new();
    loop {
        ui::draw(&mut terminal, &mut app, &mut f_view, &status_bar)?;

        // イベント処理
        match event_handler::handle_events(&mut app, &mut f_view)? {
            event_handler::AppEvent::Quit => break,
            event_handler::AppEvent::Continue => {}
        }

        // ターミナルプロセスの終了監視 (Ensure active_terminal_tab is valid before indexing)
        if app.active_target == ActiveTarget::Panel && !app.terminals.is_empty() {
            // Was Term
            if app.active_terminal_tab < app.terminals.len()
                && app.terminals[app.active_terminal_tab].content.is_dead()
            {
                // If the active terminal died, switch focus to editor.
                // Optionally, could remove the dead terminal tab here.
                app.active_target = ActiveTarget::Editor;
                // app.show_panel = false; // Optionally hide the panel area
            }
        }
    }

    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}
