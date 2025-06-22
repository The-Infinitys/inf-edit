use ratatui::{
    prelude::*,
    style::Style,
    widgets::{Block, Borders},
};

pub mod editor;
pub mod settings_editor;
use crate::{app::App, MainWidgetContent};

pub struct MainWidget;

impl MainWidget {
    pub fn new() -> Self {
        Self
    }

    /// Renders the content of the active editor.
    pub fn render(&self, f: &mut Frame, area: Rect, app: &mut App) {
        let active_tab_idx = app.active_main_tab;
        if active_tab_idx >= app.main_tabs.len() {
            // Render a placeholder if no tabs are available
            f.render_widget(Block::default().borders(Borders::ALL).title("Editor"), area);
            return;
        }

        let is_active = app.active_target == crate::ActiveTarget::Editor;
        let border_style = if is_active {
            Style::default().fg(app.theme.highlight_fg)
        } else {
            Style::default().fg(app.theme.text_fg)
        };

        if matches!(
            app.main_tabs[active_tab_idx].content,
            MainWidgetContent::SettingsEditor(_)
        ) {
            // Temporarily take ownership of the SettingsEditor to avoid double borrow.
            let mut content = std::mem::replace(
                &mut app.main_tabs[active_tab_idx].content,
                MainWidgetContent::Editor(editor::Editor::new()), // Dummy value
            );
            if let MainWidgetContent::SettingsEditor(se) = &mut content {
                se.render(f, area, app);
            }
            // Put it back.
            app.main_tabs[active_tab_idx].content = content;
        } else {
            if let Some(tab) = app.main_tabs.get_mut(active_tab_idx) {
                if let MainWidgetContent::Editor(editor) = &mut tab.content {
                    let content_block = Block::default()
                        .borders(Borders::ALL)
                        .border_style(border_style)
                        .bg(app.theme.primary_bg);
                    editor.render_with_block(f, area, content_block);
                }
            }
        }
    }
}
