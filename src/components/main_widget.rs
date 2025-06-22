use ratatui::{
    prelude::*,
    style::{Modifier, Style},
    widgets::{Block, Borders, Tabs},
};

pub mod editor;
pub mod settings_editor;
use crate::{app::App, MainWidgetContent};

pub struct MainWidget;

impl Default for MainWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl MainWidget {
    pub fn new() -> Self {
        Self
    }

    /// Renders the content of the active editor.
    pub fn render(&self, f: &mut Frame, area: Rect, app: &mut App) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(area);

        let tab_titles: Vec<String> = app.main_tabs.iter().map(|t| t.title.clone()).collect();
        let tabs = Tabs::new(tab_titles)
            .block(Block::default().bg(app.theme.primary_bg))
            .select(app.active_main_tab)
            .style(Style::default().fg(app.theme.text_fg))
            .highlight_style(
                Style::default().fg(app.theme.highlight_fg).add_modifier(Modifier::BOLD),
            );
        f.render_widget(tabs, chunks[0]);

        let content_area = chunks[1];
        let active_tab_idx = app.active_main_tab;
        if active_tab_idx >= app.main_tabs.len() {
            // Render a placeholder if no tabs are available
            f.render_widget(Block::default().borders(Borders::ALL).title("Editor"), content_area);
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
                se.render(f, content_area, app);
            }
            // Put it back.
            app.main_tabs[active_tab_idx].content = content;
        } else if let Some(tab) = app.main_tabs.get_mut(active_tab_idx) {
            if let MainWidgetContent::Editor(editor) = &mut tab.content {
                let content_block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .bg(app.theme.primary_bg);
                editor.render_with_block(f, content_area, content_block);
            }
        }
    }
}
