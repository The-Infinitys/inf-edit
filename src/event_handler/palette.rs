use super::AppEvent;
use crate::{app::App, components::top_bar::command_palette::CommandPaletteEvent};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_command_palette_events(key: KeyEvent, app: &mut App) -> Result<AppEvent> {
    // Allow closing with the same keybinding that opened it
    if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('p') {
        app.show_command_palette = false;
        return Ok(AppEvent::Continue);
    }

    match app.command_palette.handle_key(key) {
        CommandPaletteEvent::Execute => {
            app.execute_command_palette_action();
        }
        CommandPaletteEvent::Close => {
            app.show_command_palette = false;
            app.command_palette.reset();
        }
        CommandPaletteEvent::None => {}
    }

    if app.should_quit {
        return Ok(AppEvent::Quit);
    }

    Ok(AppEvent::Continue)
}
