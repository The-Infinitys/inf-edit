use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::time::Duration;

use crate::components::popup::PopupResult;
use crate::{
    app::App,
    components::notification::{send_notification, NotificationType},
};
use crate::{ActiveTarget, MainWidgetContent};
mod component;
mod global;
mod palette;
mod util;

pub use util::PtyInput; // Re-export for other modules

pub enum AppEvent {
    Quit,
    Continue,
}

pub fn handle_events(app: &mut App) -> Result<AppEvent> {
    // Poll for any async results before checking for blocking input
    app.poll_command_palette_files();
    app.poll_file_watcher();

    // Only handle events if one is available to prevent blocking
    if !event::poll(Duration::from_millis(50))? {
        return Ok(AppEvent::Continue);
    }

    // Read the event ONCE and dispatch it based on application state.
    // This prevents bugs from multiple `event::read()` calls.
    if let Event::Key(key) = event::read()? {
        // 1. Highest priority: Popups are modal and consume all input
        if let Some(popup) = &mut app.quit_popup {
            match popup.handle_key(key) {
                PopupResult::Confirm => return Ok(AppEvent::Quit),
                PopupResult::Cancel => app.quit_popup = None,
                PopupResult::None => {}
            }
            return Ok(AppEvent::Continue);
        }

        // 2. Command Palette is also modal
        if app.show_command_palette {
            return palette::handle_command_palette_events(key, app);
        }

        // 3. Terminal gets priority for most keys when active
        if app.active_target == ActiveTarget::Panel {
            // Allow only Ctrl-J (toggle panel) to be handled globally.
            if key.code == KeyCode::Char('j') && key.modifiers.contains(KeyModifiers::CONTROL) {
                if let Some(app_event) = global::handle_global_keys(key, app)? {
                    return Ok(app_event);
                }
            }
            // Pass all other keys directly to the terminal component.
            component::handle_component_keys(key, app)?;
            return Ok(AppEvent::Continue); // Don't process any other global keybindings
        }

        // 4. Global keybindings from config (e.g., Ctrl-B, Alt-L, etc.)
        if let Some(app_event) = global::handle_global_keys(key, app)? {
            return Ok(app_event);
        }

        // 5. Component-specific key handling (Editor, FileView, etc.)
        component::handle_component_keys(key, app)?;
    }

    // --- Post-event processing: check for dead processes ---
    // This logic runs regardless of whether there was an event.
    let initial_editor_len = app.main_tabs.len();
    app.main_tabs.retain(|tab| match &tab.content {
        MainWidgetContent::Editor(editor) => !editor.is_dead(),
        MainWidgetContent::SettingsEditor(_) => true, // Settings editor can't die
        MainWidgetContent::Welcome(_) => true,        // Welcome screen can't die
    });
    if app.main_tabs.len() < initial_editor_len {
        if app.main_tabs.is_empty() {
            // The last editor was closed, show the welcome screen.
            app.show_welcome_screen();
        } else if app.active_main_tab >= app.main_tabs.len() {
            // If tabs were closed, ensure active tab is valid
            app.active_main_tab = app.main_tabs.len().saturating_sub(1);
        }
    }

    // Terminals
    if app.show_panel {
        let initial_term_len = app.terminals.len();
        app.terminals.retain(|term_tab| !term_tab.content.is_dead());

        if app.terminals.len() < initial_term_len {
            send_notification(
                format!(
                    "{} terminal session(s) ended.",
                    initial_term_len - app.terminals.len()
                ),
                NotificationType::Info,
            );
        }

        if app.terminals.is_empty() {
            app.show_panel = false;
            if app.active_target == ActiveTarget::Panel {
                app.active_target = ActiveTarget::Editor;
            }
        } else if app.active_terminal_tab >= app.terminals.len() {
            // Ensure active tab index is valid
            app.active_terminal_tab = app.terminals.len().saturating_sub(1);
        }
    }

    Ok(AppEvent::Continue)
}
