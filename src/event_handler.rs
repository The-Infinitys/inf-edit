use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::time::Duration;

use crate::{
    app::App,
    components::{editor::Editor, file_view::FileView, term::Term},
    ActiveTarget, Tab,
};

pub enum AppEvent {
    Quit,
    Continue,
}

pub fn handle_events(app: &mut App, f_view: &mut FileView) -> Result<AppEvent> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            // Global quit shortcuts
            if key.modifiers == KeyModifiers::CONTROL
                && (key.code == KeyCode::Char('q') || key.code == KeyCode::Char('c'))
            {
                return Ok(AppEvent::Quit);
            }

            // Toggle File View
            if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('b') {
                app.show_file_view = !app.show_file_view;
                app.active_target = if app.show_file_view {
                    ActiveTarget::FileView
                } else if app.show_panel {
                    ActiveTarget::Panel
                } else {
                    ActiveTarget::Editor
                };
                return Ok(AppEvent::Continue);
            }

            // Toggle Panel (Terminal)
            if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('j') {
                if app.active_target == ActiveTarget::Panel {
                    app.show_panel = false;
                    if !app.editors.is_empty() {
                        app.active_target = ActiveTarget::Editor;
                    } else if app.show_file_view { // If no editors, fall back to file view if visible
                        app.active_target = ActiveTarget::FileView;
                    }
                } else {
                    if app.terminals.is_empty() {
                        app.terminals.push(Tab {
                            content: Term::new()?,
                            title: format!("Term {}", app.terminals.len() + 1),
                        });
                        app.active_terminal_tab = app.terminals.len() - 1;
                    } else {
                        if app.terminals[app.active_terminal_tab].content.is_dead() {
                            app.terminals[app.active_terminal_tab].content = Term::new()?;
                        }
                        app.active_terminal_tab = app
                            .active_terminal_tab
                            .min(app.terminals.len().saturating_sub(1));
                    }
                    app.show_panel = true;
                    // Ensure other main content areas are not focused if panel is now the target
                    app.active_target = ActiveTarget::Panel;
                }
                return Ok(AppEvent::Continue);
            }

            // Toggle Help Widget
            if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('h') {
                app.help_widget.toggle_visibility();
                return Ok(AppEvent::Continue);
            }

            // New Terminal Tab (Ctrl+Shift+J)
            if key.modifiers.contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT)
                && key.code == KeyCode::Char('j') // Changed 'u' back to 'j' as per original intent
            {
                app.terminals.push(Tab {
                    content: Term::new()?,
                    title: format!("Term {}", app.terminals.len() + 1),
                });
                app.active_terminal_tab = app.terminals.len() - 1;
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
                            } else if app.show_file_view {
                                app.active_target = ActiveTarget::FileView;
                            }
                        }
                        ActiveTarget::FileView | ActiveTarget::PrimarySideBar => {
                            if !app.editors.is_empty() {
                                app.active_target = ActiveTarget::Editor;
                            } else if !app.terminals.is_empty() {
                                app.active_target = ActiveTarget::Panel;
                                app.show_panel = true;
                            }
                        }
                        ActiveTarget::SecondarySideBar => {}
                        ActiveTarget::Term => {}
                    }
                }
                return Ok(AppEvent::Continue);
            }

            // New Editor Tab (Ctrl+N)
            if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('n') {
                app.editors.push(Tab {
                    content: Editor::new(),
                    title: format!("Editor {}", app.editors.len() + 1),
                });
                app.active_editor_tab = app.editors.len() - 1;
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
            if app.active_target == ActiveTarget::Panel && !app.terminals.is_empty() {
                if key.modifiers.contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT) {
                    match key.code {
                        KeyCode::Left => {
                            app.active_terminal_tab = app.active_terminal_tab.saturating_sub(1);
                            return Ok(AppEvent::Continue);
                        }
                        KeyCode::Right => {
                            app.active_terminal_tab = (app.active_terminal_tab + 1) % app.terminals.len();
                            return Ok(AppEvent::Continue);
                        }
                        _ => {}
                    }
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
                ActiveTarget::FileView | ActiveTarget::PrimarySideBar => match key.code {
                    KeyCode::Down | KeyCode::Char('j') => f_view.next(),
                    KeyCode::Up | KeyCode::Char('k') => f_view.previous(),
                    KeyCode::Enter => {
                        if let Some(file) = f_view.selected_file() {
                            if let Some(tab) = app.editors.get_mut(app.active_editor_tab) {
                                tab.content.open_file(file);
                            }
                            app.active_target = ActiveTarget::Editor;
                        } else {
                            f_view.enter();
                        }
                    }
                    KeyCode::Backspace | KeyCode::Char('h') => f_view.back(),
                    _ => {}
                },
                ActiveTarget::SecondarySideBar => {}
                ActiveTarget::Term => {}
            }
        }
    }
    Ok(AppEvent::Continue)
}