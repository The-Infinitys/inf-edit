use ratatui::{
    prelude::*,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Tabs},
};

pub mod editor;
pub mod settings_editor;
use crate::{app::App, components::main_widget::editor::Editor};
use crate::MainWidgetContent;

pub struct MainWidget {
    pub active_editor_tab_index: usize,
    pub is_active: bool,
}

impl MainWidget {
    pub fn new(active_editor_tab_index: usize, is_active: bool) -> Self {
        Self {
            active_editor_tab_index,
            is_active,
        }
    }

    /// Renders the editor tabs at the top of its area.
    pub fn render_tabs(&self, f: &mut Frame, area: Rect, app: &App) {
        let titles: Vec<String> = app.main_tabs.iter().map(|t| t.title.clone()).collect();
        let tabs = Tabs::new(titles)
            .select(self.active_editor_tab_index)
            .style(Style::default().fg(Color::Gray))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_widget(tabs, area);
    }

    /// Renders the content of the active editor.
    pub fn render_content(&mut self, f: &mut Frame, area: Rect, app: &mut App) {
        let active_tab_idx = self.active_editor_tab_index;
        if active_tab_idx >= app.main_tabs.len() {
            return;
        }

        let is_settings_editor = matches!(
            app.main_tabs.get(active_tab_idx).map(|t| &t.content),
            Some(MainWidgetContent::SettingsEditor(_))
        );

        let border_style = if self.is_active {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        };

        if is_settings_editor {
            // Temporarily take ownership of the SettingsEditor to avoid double borrow.
            let mut content = std::mem::replace(
                &mut app.main_tabs[active_tab_idx].content,
                MainWidgetContent::Editor(Editor::new()), // Dummy value
            );
            if let MainWidgetContent::SettingsEditor(se) = &mut content {
                se.render(f, area, app);
            }
            // Put it back.
            app.main_tabs[active_tab_idx].content = content;
        } else {
            // Handle regular editor.
            if let Some(tab) = app.main_tabs.get_mut(active_tab_idx) {
                if let MainWidgetContent::Editor(editor) = &mut tab.content {
                    let content_block = Block::default()
                        .borders(Borders::ALL)
                        .border_style(border_style)
                        .title("Editor"); // Added title for clarity
                    editor.render_with_block(f, area, content_block);
                }
            }
        }
    }
}
