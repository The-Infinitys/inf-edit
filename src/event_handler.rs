use anyhow::Result;
use crossterm::event::{self, Event};
use std::time::Duration;

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

    if !event::poll(Duration::from_millis(100))? {
        return Ok(AppEvent::Continue);
    }

    if let Event::Key(key) = event::read()? {
        if app.show_command_palette {
            return palette::handle_command_palette_events(key, app);
        }

        // --- Global Keybindings from Config ---
        if let Some(app_event) = global::handle_global_keys(key, app)? {
            return Ok(app_event);
        }

        // --- Component-specific key handling ---
        component::handle_component_keys(key, app)?;
    }

    // --- Post-event processing: check for dead processes ---
    // Editors
    let initial_editor_len = app.main_tabs.len();
    app.main_tabs.retain(|tab| match &tab.content {
        MainWidgetContent::Editor(editor) => !editor.is_dead(),
        MainWidgetContent::SettingsEditor(_) => true, // Settings editor can't die
    });
    if app.main_tabs.len() < initial_editor_len {
        // If tabs were closed, ensure active tab is valid
        if app.active_main_tab >= app.main_tabs.len() {
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
