use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::time::{Duration};

use crate::{
    ActiveTarget,
    app::App,
    components::{
        main_widget::editor::Editor, panel::term::Term,
        primary_sidebar::component::PrimarySidebarComponent,
    },
};

pub enum AppEvent {
    Quit,
    Continue,
}

// const DEBOUNCE_DURATION: Duration = Duration::from_millis(50); // Debounce for AltGr or similar issues
// static LAST_CTRL_ALT_EVENT_TIME: std::sync::OnceLock<std::sync::Mutex<Instant>> =
//     std::sync::OnceLock::new();

pub fn handle_events(app: &mut App) -> Result<AppEvent> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            // Global quit shortcuts
            if key.modifiers == KeyModifiers::CONTROL
                && (key.code == KeyCode::Char('q') || key.code == KeyCode::Char('c'))
            {
                return Ok(AppEvent::Quit);
            }

            // Toggle File View
            // - If hidden: show and focus.
            // - If visible and focused: hide and focus editor/panel.
            // - If visible and not focused: just focus.
            if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('b') {
                if !app.show_primary_sidebar {
                    app.show_primary_sidebar = true;
                    app.active_target = ActiveTarget::PrimarySideBar;
                } else if app.active_target == ActiveTarget::PrimarySideBar {
                    app.show_primary_sidebar = false;
                    app.active_target = if !app.editors.is_empty() {
                        ActiveTarget::Editor
                    } else if app.show_panel {
                        ActiveTarget::Panel
                    } else {
                        ActiveTarget::Editor // Fallback
                    };
                } else {
                    app.active_target = ActiveTarget::PrimarySideBar;
                }
                return Ok(AppEvent::Continue);
            }

            // Toggle Panel (Terminal)
            // - If hidden: show and focus.
            // - If visible and focused: hide and focus editor/sidebar.
            // - If visible and not focused: just focus.
            if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('j') {
                if !app.show_panel { // Case 1: Panel is hidden. We want to show it.
                    if app.terminals.is_empty() {
                        // Create the first terminal if none exist.
                        let term = Term::new()?;
                        app.add_terminal_tab(term, format!("Term {}", app.terminals.len() + 1));
                    } else if let Some(tab) = app.terminals.get_mut(app.active_terminal_tab) {
                        if tab.content.is_dead() {
                            // If the active terminal is dead, restart it.
                            tab.content = Term::new()?;
                        }
                    }
                    app.show_panel = true;
                    app.active_target = ActiveTarget::Panel;
                } else { // Case 2: Panel is visible.
                    if app.active_target == ActiveTarget::Panel {
                        // It's visible and focused, so hide it.
                        app.show_panel = false;
                        app.active_target = if !app.editors.is_empty() {
                            ActiveTarget::Editor
                        } else if app.show_primary_sidebar {
                            ActiveTarget::PrimarySideBar
                        } else {
                            ActiveTarget::Editor
                        };
                    } else {
                        // It's visible but not focused. We want to focus it.
                        // Before focusing, check if it's dead and restart if needed.
                        if let Some(tab) = app.terminals.get_mut(app.active_terminal_tab) {
                            if tab.content.is_dead() {
                                tab.content = Term::new()?;
                            }
                        }
                        app.active_target = ActiveTarget::Panel;
                    }
                }
                return Ok(AppEvent::Continue);
            }

            // Toggle Help Widget
            if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('h') {
                // This now toggles the visibility of the secondary sidebar, assuming help is there.
                // A more robust implementation would check if the help tab exists.
                app.show_secondary_sidebar = !app.show_secondary_sidebar;
                if app.show_secondary_sidebar {
                    app.active_target = ActiveTarget::SecondarySideBar;
                } else if app.active_target == ActiveTarget::SecondarySideBar {
                    // If we closed the active sidebar, move focus back to editor
                    app.active_target = ActiveTarget::Editor;
                }

                return Ok(AppEvent::Continue);
            }

            // New Terminal Tab (Ctrl+Shift+J)
            if key
                .modifiers
                .contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT)
                && key.code == KeyCode::Char('j')
            // Changed 'u' back to 'j' as per original intent
            {
                let term = Term::new()?;
                app.add_terminal_tab(term, format!("Term {}", app.terminals.len() + 1));
                app.active_target = ActiveTarget::Panel; // Focus the new terminal panel
                app.show_panel = true; // Ensure panel is visible
                return Ok(AppEvent::Continue);
            }

            // Switch focus (Ctrl+K)
            if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('k') {
                if app.show_panel {
                    match app.active_target {
                        ActiveTarget::Editor => {
                            if app.show_panel && !app.terminals.is_empty() {
                                app.active_target = ActiveTarget::Panel;
                            }
                        }
                        ActiveTarget::Panel => {
                            if !app.editors.is_empty() {
                                app.active_target = ActiveTarget::Editor;
                            } else if app.show_primary_sidebar {
                                app.active_target = ActiveTarget::PrimarySideBar;
                            }
                        }
                        ActiveTarget::PrimarySideBar => {
                            if !app.editors.is_empty() {
                                app.active_target = ActiveTarget::Editor;
                            } else if !app.terminals.is_empty() {
                                app.active_target = ActiveTarget::Panel;
                                app.show_panel = true;
                            }
                        }
                        ActiveTarget::SecondarySideBar => {}
                    }
                }
                return Ok(AppEvent::Continue);
            }

            // New Editor Tab (Ctrl+N)
            if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('n') {
                let editor = Editor::new();
                app.add_editor_tab(editor, format!("Editor {}", app.editors.len() + 1));
                app.active_target = ActiveTarget::Editor;
                app.show_panel = false;
                return Ok(AppEvent::Continue);
            }

            // Switch Tabs (Ctrl+T)
            if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('t') {
                if app.active_target == ActiveTarget::Editor && !app.editors.is_empty() {
                    app.active_editor_tab = (app.active_editor_tab + 1) % app.editors.len();
                } else if app.active_target == ActiveTarget::Panel && !app.terminals.is_empty() {
                    app.active_terminal_tab = (app.active_terminal_tab + 1) % app.terminals.len();
                }
                return Ok(AppEvent::Continue);
            }

            // Switch Terminal Tabs (Ctrl+Shift+Left/Right) - Only when Panel is active
            if app.active_target == ActiveTarget::Panel
                && !app.terminals.is_empty()
                && key
                    .modifiers
                    .contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT)
            {
                match key.code {
                    KeyCode::Left => {
                        app.active_terminal_tab = app.active_terminal_tab.saturating_sub(1);
                        return Ok(AppEvent::Continue);
                    }
                    KeyCode::Right => {
                        app.active_terminal_tab =
                            (app.active_terminal_tab + 1) % app.terminals.len();
                        return Ok(AppEvent::Continue);
                    }
                    _ => {}
                }
            }

            // Component-specific key handling
            match app.active_target {
                ActiveTarget::Editor => {
                    if let Some(tab) = app.editors.get_mut(app.active_editor_tab) {
                        // Simplified: direct key code matching for editor-like inputs
                        // In a real scenario, this would be more complex, potentially passing `key` to `tab.content.handle_key(key)`
                        match key.code {
                            KeyCode::Char(c) => tab.content.send_input(c.to_string().as_bytes()),
                            KeyCode::Enter => tab.content.send_input(b"\n"),
                            KeyCode::Tab => tab.content.send_input(b"\t"),
                            KeyCode::Backspace => tab.content.send_input(&[8]),
                            // Arrow keys, Esc, etc.
                            _ => {}
                        }
                    }
                }
                ActiveTarget::Panel => {
                    if let Some(tab) = app.terminals.get_mut(app.active_terminal_tab) {
                        match key.code {
                            KeyCode::Char(c) => tab.content.send_input(c.to_string().as_bytes()),
                            KeyCode::Enter => tab.content.send_input(b"\n"),
                            KeyCode::Tab => tab.content.send_input(b"\t"),
                            KeyCode::Backspace => tab.content.send_input(&[8]),
                            // Arrow keys, Esc, etc.
                            _ => {}
                        }
                    }
                }
                ActiveTarget::PrimarySideBar => {
                    if let Some(tab) = app
                        .primary_sidebar_components
                        .get_mut(app.active_primary_sidebar_tab)
                    {
                        if let PrimarySidebarComponent::FileView(f_view) = &mut tab.content {
                            match key.code {
                                KeyCode::Down | KeyCode::Char('j') => f_view.next(),
                                KeyCode::Up | KeyCode::Char('k') => f_view.previous(),
                                KeyCode::Enter => {
                                    if let Some(path) = f_view.selected_file() {
                                        let mut editor = Editor::new();
                                        let title = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                                        editor.open_file(path);
                                        app.add_editor_tab(editor, title);
                                    } else {
                                        f_view.enter();
                                    }
                                }
                                KeyCode::Backspace | KeyCode::Char('h') => f_view.back(),
                                _ => {}
                            }
                        }
                    }
                }
                ActiveTarget::SecondarySideBar => {}
            }
        }
    }
    Ok(AppEvent::Continue)
}
