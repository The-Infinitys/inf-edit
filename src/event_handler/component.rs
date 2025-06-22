use super::util::send_key_to_terminal;
use crate::{
    app::App,
    components::{
        main_widget::editor::Editor, primary_sidebar::component::PrimarySidebarComponent,
    },
    ActiveTarget, MainWidgetContent,
};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_component_keys(key: KeyEvent, app: &mut App) -> Result<()> {
    match app.active_target {
        ActiveTarget::Editor => {
            let active_tab_idx = app.active_main_tab;
            if active_tab_idx < app.main_tabs.len() {
                let is_settings_editor = matches!(
                    app.main_tabs[active_tab_idx].content,
                    MainWidgetContent::SettingsEditor(_)
                );

                if is_settings_editor {
                    let content_placeholder = std::mem::replace(
                        &mut app.main_tabs[active_tab_idx].content,
                        MainWidgetContent::Editor(Editor::new()),
                    );

                    if let MainWidgetContent::SettingsEditor(mut settings_editor) =
                        content_placeholder
                    {
                        settings_editor.handle_key(key, app);
                        app.main_tabs[active_tab_idx].content =
                            MainWidgetContent::SettingsEditor(settings_editor);
                    }
                } else if let Some(editor) =
                    app.main_tabs
                        .get_mut(active_tab_idx)
                        .and_then(|t| match &mut t.content {
                            MainWidgetContent::Editor(e) => Some(e),
                            _ => None,
                        })
                {
                    send_key_to_terminal(editor, key);
                }
            }
        }
        ActiveTarget::Panel => {
            if let Some(tab) = app.terminals.get_mut(app.active_terminal_tab) {
                send_key_to_terminal(&tab.content, key);
            }
        }
        ActiveTarget::PrimarySideBar => {
            // To avoid multiple mutable borrows of `app`, we first check if we need to open a file
            // without holding a mutable reference to the sidebar component.
            let mut file_to_open = None;
            if key.code == KeyCode::Enter {
                if let Some(tab) = app
                    .primary_sidebar_components
                    .get(app.active_primary_sidebar_tab)
                {
                    if let PrimarySidebarComponent::FileView(f_view) = &tab.content {
                        file_to_open = f_view.selected_file();
                    }
                }
            }

            if let Some(path) = file_to_open {
                // If a file was selected, open it in a new tab.
                let editor = Editor::with_file(path.to_path_buf());
                let title = path.to_string_lossy().to_string(); // PathBuf to String for title
                app.add_editor_tab(editor, title);
            } else {
                // Otherwise, pass the key event to the active sidebar component.
                if let Some(tab) = app
                    .primary_sidebar_components
                    .get_mut(app.active_primary_sidebar_tab)
                {
                    tab.content.handle_key(key);
                }
            }
        }
        ActiveTarget::SecondarySideBar => {
            // No specific key handling for secondary sidebar for now
        }
    }
    Ok(())
}
