use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::time::{Duration};

use crate::{
    ActiveTarget,
    app::App,
    components::{
        main_widget::editor::Editor, panel::term::Term,
        primary_sidebar::component::PrimarySidebarComponent, // Keep this import
        secondary_sidebar::component::SecondarySidebarComponent, // Add this import
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

            // Toggle Help Widget (Ctrl+Alt+B)
            if key.modifiers.contains(KeyModifiers::CONTROL | KeyModifiers::ALT) && key.code == KeyCode::Char('b') {
                let mut help_is_now_visible = false;
                if let Some(tab) = app.secondary_sidebar_components.get_mut(app.active_secondary_sidebar_tab) {
                    // The compiler warns this is an irrefutable pattern because the secondary sidebar
                    // currently only ever contains a Help widget. Using `let` is more direct.
                    // If other component types are added later, this will become a compile error,
                    // prompting a necessary change back to `if let` or `match`.
                    let SecondarySidebarComponent::Help(help_widget) = &mut tab.content;
                    help_is_now_visible = help_widget.toggle_visibility(); // Get the new state
                }

                if help_is_now_visible {
                    app.show_secondary_sidebar = true;
                    app.active_target = ActiveTarget::SecondarySideBar;
                } else if app.active_target == ActiveTarget::SecondarySideBar {
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

            // Switch Primary Sidebar Tabs (Ctrl+Shift+, for previous, Ctrl+Shift+. for next)
            // This corresponds to Ctrl+< and Ctrl+> on many layouts.
            // This avoids the problematic Ctrl+[ which is the same as ESC, and works across keyboard layouts.
            if key.modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) {
                match key.code {
                    KeyCode::Char(',') => { // Previous Tab (Ctrl+<)
                        if app.show_primary_sidebar && !app.primary_sidebar_components.is_empty() {
                            app.active_primary_sidebar_tab = app.active_primary_sidebar_tab.saturating_sub(1);
                            app.active_target = ActiveTarget::PrimarySideBar;
                            return Ok(AppEvent::Continue);
                        }
                    }
                    KeyCode::Char('.') => { // Next Tab (Ctrl+>)
                        if app.show_primary_sidebar && !app.primary_sidebar_components.is_empty() {
                            app.active_primary_sidebar_tab = (app.active_primary_sidebar_tab + 1) % app.primary_sidebar_components.len();
                            app.active_target = ActiveTarget::PrimarySideBar;
                            return Ok(AppEvent::Continue);
                        }
                    }
                    _ => {}
                }
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
                ActiveTarget::Editor => if let Some(tab) = app.editors.get_mut(app.active_editor_tab) {
                    send_key_to_terminal(&mut tab.content, key);
                },
                ActiveTarget::Panel => if let Some(tab) = app.terminals.get_mut(app.active_terminal_tab) {
                    send_key_to_terminal(&mut tab.content, key);
                },
                ActiveTarget::PrimarySideBar => {
                    if let Some(tab) = app
                        .primary_sidebar_components
                        .get_mut(app.active_primary_sidebar_tab)
                    {
                        if let PrimarySidebarComponent::FileView(f_view) = &mut tab.content {
                            // This logic is specific to the FileView and does not use the terminal key sending.
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

/// Converts a crossterm KeyEvent into a byte sequence that can be sent to a PTY.
/// This allows sending special keys like arrows, home, end, etc., to the underlying process.
///
/// This function is generic over any type `T` that implements the `PtyInput` trait.
/// Both `Editor` and `Term` are expected to implement this trait.
fn send_key_to_terminal<T>(target: &T, key: event::KeyEvent)
where
    T: PtyInput + ?Sized,
{
    let mut bytes: Vec<u8> = Vec::new();

    // First, handle the key code to get the base byte sequence.
    let code_bytes = match key.code {
        KeyCode::Backspace => vec![8],      // Backspace
        KeyCode::Enter => vec![b'\r'],     // Carriage Return
        KeyCode::Left => b"\x1b[D".to_vec(),
        KeyCode::Right => b"\x1b[C".to_vec(),
        KeyCode::Up => b"\x1b[A".to_vec(),
        KeyCode::Down => b"\x1b[B".to_vec(),
        KeyCode::Home => b"\x1b[H".to_vec(),
        KeyCode::End => b"\x1b[F".to_vec(),
        KeyCode::PageUp => b"\x1b[5~".to_vec(),
        KeyCode::PageDown => b"\x1b[6~".to_vec(),
        KeyCode::Tab => vec![b'\t'],
        KeyCode::BackTab => b"\x1b[Z".to_vec(),
        KeyCode::Delete => b"\x1b[3~".to_vec(),
        KeyCode::Insert => b"\x1b[2~".to_vec(),
        KeyCode::F(n) => format!("\x1b[{}~", n + 11).into_bytes(), // A common mapping for F-keys
        KeyCode::Char(c) => c.to_string().into_bytes(),
        KeyCode::Esc => vec![0x1b],
        _ => vec![], // Ignore null, caps lock, etc.
    };

    bytes.extend(code_bytes);

    // Handle modifiers. This is a simplified implementation.
    // Note: Ctrl+Char is often handled by the terminal itself by converting 'a' to 1, 'b' to 2, etc.
    // Crossterm might already do this, but if not, we handle it here.
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        if let KeyCode::Char(c @ 'a'..='z') = key.code {
            // This maps Ctrl+a to 1, Ctrl+b to 2, etc.
            bytes = vec![(c as u8) - b'a' + 1];
        }
    }

    // Alt is often sent as an ESC prefix.
    if key.modifiers.contains(KeyModifiers::ALT) && !bytes.is_empty() {
        // Don't add ESC prefix if the key is already ESC
        if !(key.code == KeyCode::Esc) {
            bytes.insert(0, 0x1b);
        }
    }

    if !bytes.is_empty() {
        target.send_input(&bytes);
    }
}

// Define a trait for types that can receive input bytes for a PTY.
pub trait PtyInput {
    fn send_input(&self, bytes: &[u8]);
}
