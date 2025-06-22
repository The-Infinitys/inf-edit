use super::{util::key_event_to_string, AppEvent};
use crate::{
    app::App,
    components::{panel::term::Term, primary_sidebar::component::PrimarySidebarComponent},
    ActiveTarget,
};
use anyhow::Result;
use crossterm::event::KeyEvent;
use std::env;

pub fn handle_global_keys(key: KeyEvent, app: &mut App) -> Result<Option<AppEvent>> {
    if let Some(key_str) = key_event_to_string(key) {
        if let Some(action) = app.config.keybindings.global.get(&key_str).cloned() {
            match action.as_str() {
                "quit" => return Ok(Some(AppEvent::Quit)),
                "toggle_primary_sidebar" => {
                    if !app.show_primary_sidebar {
                        app.show_primary_sidebar = true;
                        app.active_target = ActiveTarget::PrimarySideBar;
                    } else if app.active_target == ActiveTarget::PrimarySideBar {
                        app.show_primary_sidebar = false;
                        app.active_target = ActiveTarget::Editor;
                    } else {
                        app.active_target = ActiveTarget::PrimarySideBar;
                    }
                }
                "toggle_panel" => {
                    let cwd_for_new_term = app
                        .primary_sidebar_components
                        .get(app.active_primary_sidebar_tab)
                        .and_then(|tab| match &tab.content {
                            PrimarySidebarComponent::FileView(fv) => Some(fv.current_path().clone()),
                            _ => None,
                        })
                        .or_else(|| env::current_dir().ok());

                    if !app.show_panel {
                        if app.terminals.is_empty() {
                            let term = Term::new(cwd_for_new_term)?;
                            app.add_terminal_tab(term, format!("Term {}", app.terminals.len() + 1));
                        }
                        app.show_panel = true;
                        app.active_target = ActiveTarget::Panel;
                    } else if app.active_target == ActiveTarget::Panel {
                        app.show_panel = false;
                        app.active_target = ActiveTarget::Editor;
                    } else {
                        app.active_target = ActiveTarget::Panel;
                    }
                }
                "toggle_secondary_sidebar" => {
                    if !app.show_secondary_sidebar {
                        app.show_secondary_sidebar = true;
                        app.active_target = ActiveTarget::SecondarySideBar;
                    } else if app.active_target == ActiveTarget::SecondarySideBar {
                        app.show_secondary_sidebar = false;
                        app.active_target = ActiveTarget::Editor;
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
                        app.add_editor_tab(editor, format!("Editor {}", app.main_tabs.len() + 1));
                    }
                    ActiveTarget::Panel => {
                        let cwd = env::current_dir().ok();
                        let term = Term::new(cwd)?;
                        app.add_terminal_tab(term, format!("Term {}", app.terminals.len() + 1));
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
                        app.active_primary_sidebar_tab = app
                            .active_primary_sidebar_tab
                            .checked_sub(1)
                            .unwrap_or(app.primary_sidebar_components.len() - 1);
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
                        app.active_primary_sidebar_tab =
                            (app.active_primary_sidebar_tab + 1) % app.primary_sidebar_components.len()
                    }
                    _ => {}
                },
                _ => { /* Unhandled action */ }
            }
            return Ok(Some(AppEvent::Continue));
        }
    }
    Ok(None) // No global key was handled
}