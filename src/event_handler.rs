use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::env;
use std::time::Duration;

use crate::{
    app::App,
    components::{
        main_widget::editor::Editor,
        notification::{send_notification, NotificationType},
        panel::term::Term,
        primary_sidebar::component::PrimarySidebarComponent, // Keep this import
    },
    settings::Config,
    theme::Theme,
    ActiveTarget, MainWidgetContent,
};

pub enum AppEvent {
    Quit,
    Continue,
}

/// Converts a KeyEvent to a string representation like "Ctrl-C" or "Alt-H".
fn key_event_to_string(key: event::KeyEvent) -> Option<String> {
    let mut parts = Vec::new();
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        parts.push("Ctrl");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        parts.push("Alt");
    }
    if key.modifiers.contains(KeyModifiers::SHIFT) {
        parts.push("Shift");
    }

    let key_str = match key.code {
        KeyCode::Char(c) => c.to_uppercase().to_string(),
        KeyCode::F(n) => format!("F{}", n),
        _ => return None, // Only handle a subset of keys for global bindings
    };
    parts.push(&key_str);
    Some(parts.join("-"))
}

pub fn handle_events(app: &mut App) -> Result<AppEvent> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            if app.show_command_palette {
                if key.modifiers == KeyModifiers::CONTROL
                    && (key.code == KeyCode::Char('q') || key.code == KeyCode::Char('c'))
                {
                    return Ok(AppEvent::Quit);
                }
                if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('p') {
                    app.show_command_palette = false;
                    return Ok(AppEvent::Continue);
                }
                if key.code == KeyCode::Esc {
                    app.show_command_palette = false;
                    return Ok(AppEvent::Continue);
                }
                // それ以外はコマンドパレットに渡す
                let event = app.command_palette.handle_key(key);

                match event {
                    crate::components::top_bar::command_palette::CommandPaletteEvent::Exit => {
                        app.show_command_palette = false;
                    }
                    crate::components::top_bar::command_palette::CommandPaletteEvent::ExecuteCommand(_cmd_name) => {
                        // ここでパレットを閉じるなどのUI遷移を必ず行う
                        app.show_command_palette = false;
                        // 追加で必要な処理があればここに
                    }
                    crate::components::top_bar::command_palette::CommandPaletteEvent::OpenSettings => {
                        app.add_settings_tab();
                        app.show_command_palette = false;
                    }
                    crate::components::top_bar::command_palette::CommandPaletteEvent::ResetSettings => {
                        app.config = Config::default();
                        app.theme = Theme::from_config(&app.config.theme);
                        if let Err(e) = app.config.save() {
                            let msg = format!("Failed to save reset config: {}", e);
                            send_notification(msg, NotificationType::Error);
                        } else {
                            send_notification("Settings reset to default.".to_string(), NotificationType::Info);
                        }
                        app.show_command_palette = false;
                    }
                    crate::components::top_bar::command_palette::CommandPaletteEvent::OpenFile(path) => {
                        // ファイルを開く処理例
                        let mut editor = crate::components::main_widget::editor::Editor::new();
                        let title = path.clone();
                        editor.open_file(path.into());
                        app.add_editor_tab(editor, title);
                        app.show_command_palette = false;
                    }
                    crate::components::top_bar::command_palette::CommandPaletteEvent::None => {} // None is a valid event, do nothing.
                }
                return Ok(AppEvent::Continue);
            }

            // --- Check for dead processes and remove tabs ---
            // Editors
            if !app.main_tabs.is_empty() {
                let mut is_dead = false;
                if let Some(tab) = app.main_tabs.get(app.active_main_tab) {
                    if let MainWidgetContent::Editor(editor) = &tab.content {
                        if editor.is_dead() {
                            is_dead = true;
                        }
                    }
                }
                if is_dead {
                    app.main_tabs.remove(app.active_main_tab);
                    if app.main_tabs.is_empty() {
                        app.add_editor_tab(
                            crate::components::main_widget::editor::Editor::new(),
                            "Editor 1".to_string(),
                        );
                    } else if app.active_main_tab >= app.main_tabs.len() {
                        app.active_main_tab = app.main_tabs.len() - 1;
                    }
                }
            }

            // Terminals
            if !app.terminals.is_empty() && app.terminals[app.active_terminal_tab].content.is_dead()
            {
                app.terminals.remove(app.active_terminal_tab);
                if app.terminals.is_empty() {
                    // If all terminals are closed, hide the panel and switch focus
                    app.show_panel = false;
                    app.active_target = ActiveTarget::Editor; // Fallback to editor
                } else if app.active_terminal_tab >= app.terminals.len() {
                    app.active_terminal_tab = app.terminals.len() - 1;
                }
                // If the active target was panel, keep it focused
                if app.active_target == ActiveTarget::Panel {
                    return Ok(AppEvent::Continue);
                }
            }
            // --- End of dead process check ---

            // --- Global Keybindings from Config ---
            if let Some(key_str) = key_event_to_string(key) {
                if let Some(action) = app.config.keybindings.global.get(&key_str).cloned() {
                    match action.as_str() {
                        "quit" => return Ok(AppEvent::Quit),
                        "toggle_primary_sidebar" => {
                            if !app.show_primary_sidebar {
                                app.show_primary_sidebar = true;
                                app.active_target = ActiveTarget::PrimarySideBar;
                            } else if app.active_target == ActiveTarget::PrimarySideBar {
                                app.show_primary_sidebar = false;
                                app.active_target = if !app.main_tabs.is_empty() {
                                    ActiveTarget::Editor
                                } else if app.show_panel {
                                    ActiveTarget::Panel
                                } else {
                                    ActiveTarget::Editor // Fallback
                                };
                            } else {
                                app.active_target = ActiveTarget::PrimarySideBar;
                            }
                        }
                        "toggle_panel" => {
                            let cwd_for_new_term = app
                                .primary_sidebar_components
                                .get(app.active_primary_sidebar_tab)
                                .and_then(|tab| match &tab.content {
                                    PrimarySidebarComponent::FileView(fv) => {
                                        Some(fv.current_path().clone())
                                    }
                                    _ => None,
                                })
                                .or_else(|| env::current_dir().ok());

                            if !app.show_panel {
                                if app.terminals.is_empty() {
                                    let term = Term::new(cwd_for_new_term)?;
                                    app.add_terminal_tab(
                                        term,
                                        format!("Term {}", app.terminals.len() + 1),
                                    );
                                } else if let Some(tab) = app.terminals.get_mut(app.active_terminal_tab) {
                                    if tab.content.is_dead() {
                                        tab.content = Term::new(cwd_for_new_term)?;
                                    }
                                }
                                app.show_panel = true;
                                app.active_target = ActiveTarget::Panel;
                            } else if app.active_target == ActiveTarget::Panel {
                                app.show_panel = false;
                                app.active_target = if !app.main_tabs.is_empty() {
                                    ActiveTarget::Editor
                                } else if app.show_primary_sidebar {
                                    ActiveTarget::PrimarySideBar
                                } else {
                                    ActiveTarget::Editor
                                };
                            } else {
                                if let Some(tab) = app.terminals.get_mut(app.active_terminal_tab) {
                                    if tab.content.is_dead() {
                                        tab.content = Term::new(cwd_for_new_term)?;
                                    }
                                }
                                app.active_target = ActiveTarget::Panel;
                            }
                        }
                        "toggle_secondary_sidebar" => {
                            if !app.show_secondary_sidebar {
                                app.show_secondary_sidebar = true;
                                app.active_target = ActiveTarget::SecondarySideBar;
                            } else if app.active_target == ActiveTarget::SecondarySideBar {
                                app.show_secondary_sidebar = false;
                                app.active_target = if !app.main_tabs.is_empty() {
                                    ActiveTarget::Editor
                                } else if app.show_primary_sidebar {
                                    ActiveTarget::PrimarySideBar
                                } else if app.show_panel {
                                    ActiveTarget::Panel
                                } else {
                                    ActiveTarget::Editor // Fallback
                                };
                            } else {
                                app.active_target = ActiveTarget::SecondarySideBar;
                            }
                        }
                        "cycle_focus" => {
                            let mut targets = Vec::new();
                            if !app.main_tabs.is_empty() {
                                targets.push(ActiveTarget::Editor);
                            }
                            if app.show_panel && !app.terminals.is_empty() {
                                targets.push(ActiveTarget::Panel);
                            }
                            if app.show_primary_sidebar {
                                targets.push(ActiveTarget::PrimarySideBar);
                            }

                            if !targets.is_empty() {
                                let current_idx = targets.iter().position(|&t| t == app.active_target);
                                let next_idx = match current_idx {
                                    Some(idx) => (idx + 1) % targets.len(),
                                    None => 0,
                                };
                                app.active_target = targets[next_idx];
                            }
                        }
                        "toggle_command_palette" => {
                            app.show_command_palette = !app.show_command_palette;
                        }
                        "new_tab" => match app.active_target {
                            ActiveTarget::Editor => {
                                let editor = crate::components::main_widget::editor::Editor::new();
                                app.add_editor_tab(
                                    editor,
                                    format!("Editor {}", app.main_tabs.len() + 1),
                                );
                            }
                            ActiveTarget::Panel => {
                                let cwd = env::current_dir().ok();
                                let term = Term::new(cwd)?;
                                app.add_terminal_tab(
                                    term,
                                    format!("Term {}", app.terminals.len() + 1),
                                );
                            }
                            _ => {
                                let editor = crate::components::main_widget::editor::Editor::new();
                                app.add_editor_tab(editor, format!("Editor {}", app.main_tabs.len() + 1));
                                app.active_target = ActiveTarget::Editor;
                            }
                        },
                        "close_tab" => match app.active_target {
                            ActiveTarget::Editor => {
                                if !app.main_tabs.is_empty() {
                                    app.main_tabs.remove(app.active_main_tab);
                                    if app.main_tabs.is_empty() {
                                        app.add_editor_tab(
                                            crate::components::main_widget::editor::Editor::new(),
                                            "Editor 1".to_string(),
                                        );
                                    } else if app.active_main_tab >= app.main_tabs.len() {
                                        app.active_main_tab = app.main_tabs.len().saturating_sub(1);
                                    }
                                }
                            }
                            ActiveTarget::Panel => {
                                if !app.terminals.is_empty() {
                                    app.terminals.remove(app.active_terminal_tab);
                                    if app.terminals.is_empty() {
                                        app.show_panel = false;
                                        app.active_target = ActiveTarget::Editor;
                                    } else if app.active_terminal_tab >= app.terminals.len() {
                                        app.active_terminal_tab = app.terminals.len().saturating_sub(1);
                                    }
                                }
                            }
                            _ => {}
                        },
                        "prev_tab" => match app.active_target {
                            ActiveTarget::Editor if !app.main_tabs.is_empty() => {
                                app.active_main_tab = app
                                    .active_main_tab
                                    .checked_sub(1)
                                    .unwrap_or(app.main_tabs.len() - 1);
                            }
                            ActiveTarget::Panel if !app.terminals.is_empty() => {
                                app.active_terminal_tab = app
                                    .active_terminal_tab
                                    .checked_sub(1)
                                    .unwrap_or(app.terminals.len() - 1);
                            }
                            ActiveTarget::PrimarySideBar if !app.primary_sidebar_components.is_empty() => {
                                app.active_primary_sidebar_tab = app.active_primary_sidebar_tab.checked_sub(1).unwrap_or(app.primary_sidebar_components.len() - 1);
                            }
                            _ => {}
                        },
                        "next_tab" => match app.active_target {
                            ActiveTarget::Editor if !app.main_tabs.is_empty() => {
                                app.active_main_tab = (app.active_main_tab + 1) % app.main_tabs.len();
                            }
                            ActiveTarget::Panel if !app.terminals.is_empty() => {
                                app.active_terminal_tab = (app.active_terminal_tab + 1) % app.terminals.len();
                            }
                            ActiveTarget::PrimarySideBar if !app.primary_sidebar_components.is_empty() => {
                                app.active_primary_sidebar_tab = (app.active_primary_sidebar_tab + 1) % app.primary_sidebar_components.len()
                            }
                            _ => {}
                        },
                        _ => {} // Unhandled action
                    }
                    return Ok(AppEvent::Continue);
                }
            }

            // Switch Primary Sidebar Tabs (Ctrl+Tab for next, Ctrl+Shift+Tab for previous)
            if key.modifiers == KeyModifiers::CONTROL {
                match key.code {
                    KeyCode::Tab => {
                        if app.show_primary_sidebar && !app.primary_sidebar_components.is_empty() {
                            app.active_primary_sidebar_tab = (app.active_primary_sidebar_tab + 1)
                                % app.primary_sidebar_components.len();
                            app.active_target = ActiveTarget::PrimarySideBar;
                            return Ok(AppEvent::Continue);
                        }
                    }
                    KeyCode::BackTab => {
                        if app.show_primary_sidebar && !app.primary_sidebar_components.is_empty() {
                            app.active_primary_sidebar_tab = if app.active_primary_sidebar_tab == 0
                            {
                                app.primary_sidebar_components.len() - 1
                            } else {
                                app.active_primary_sidebar_tab - 1
                            };
                            app.active_target = ActiveTarget::PrimarySideBar;
                            return Ok(AppEvent::Continue);
                        }
                    }
                    _ => {}
                }
            }

            // Switch Terminal Tabs (Ctrl+Shift+Left/Right) - Only when Panel is active
            if app.active_target == ActiveTarget::Panel
                && !app.terminals.is_empty()
                && key
                    .modifiers
                    .contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT)
            {
                match key.code {
                    KeyCode::Up => {
                        app.active_terminal_tab = app.active_terminal_tab.saturating_sub(1);
                        return Ok(AppEvent::Continue);
                    }
                    KeyCode::Down => {
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
                    let active_tab_idx = app.active_main_tab;

                    // Ensure the index is valid before proceeding
                    if active_tab_idx < app.main_tabs.len() {
                        // Check the type of content without creating a long-lived mutable borrow of the tab.
                        let is_settings_editor = matches!(app.main_tabs.get(active_tab_idx).map(|t| &t.content), Some(MainWidgetContent::SettingsEditor(_)));

                        if is_settings_editor {
                            // Temporarily take out the SettingsEditor from the tab.
                            // This directly accesses and replaces the content, avoiding a long-lived `tab` borrow.
                            let content_placeholder = std::mem::replace(
                                &mut app.main_tabs[active_tab_idx].content,
                                MainWidgetContent::Editor(Editor::new()), // Dummy placeholder
                            );

                            if let MainWidgetContent::SettingsEditor(mut settings_editor) = content_placeholder {
                                // Now, `app.main_tabs` is not mutably borrowed, so we can safely pass `app` mutably.
                                settings_editor.handle_key(key, app);
                                // Put the (potentially modified) settings editor back into the tab.
                                app.main_tabs[active_tab_idx].content = MainWidgetContent::SettingsEditor(settings_editor);
                            } else {
                                unreachable!("Expected SettingsEditor content after check."); // Should not happen
                            }
                        } else {
                            // If it's not a SettingsEditor, handle it as a regular Editor.
                            if let Some(editor) = app.main_tabs.get_mut(active_tab_idx).and_then(|t| match &mut t.content {
                                MainWidgetContent::Editor(e) => Some(e),
                                _ => None,
                            }) {
                                send_key_to_terminal(editor, key);
                            }
                        }
                    }
                }
                ActiveTarget::Panel => {
                    if let Some(tab) = app.terminals.get_mut(app.active_terminal_tab) {
                        send_key_to_terminal(&tab.content, key);
                    }
                }
                ActiveTarget::PrimarySideBar => {
                    if let Some(tab) = app
                        .primary_sidebar_components
                        .get_mut(app.active_primary_sidebar_tab)
                    {
                        // Let the active sidebar component handle the key.
                        let key_handled = tab.content.handle_key(key);

                        // Special handling for FileView's Enter key to open files
                        if key.code == KeyCode::Enter {
                            if let PrimarySidebarComponent::FileView(f_view) = &tab.content {
                                if let Some(path) = f_view.selected_file() {
                                    // The selected_file() method correctly returns None for directories.
                                    let mut editor = Editor::new();
                                let title = path.to_string_lossy().to_string();
                                // Full path for title
                                    editor.open_file(path);
                                    app.add_editor_tab(editor, title);
                                    return Ok(AppEvent::Continue);
                                }
                            }
                        }
                        if key_handled {
                            return Ok(AppEvent::Continue);
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
        KeyCode::Backspace => vec![8], // Backspace
        KeyCode::Enter => vec![b'\r'], // Carriage Return
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
