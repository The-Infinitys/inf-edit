use ratatui::{
    prelude::*,
    style::{Modifier, Style},
    widgets::{Block, Borders, Tabs},
};
pub mod editor;
pub mod settings_editor;
pub mod welcome_widget;
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
                Style::default()
                    .fg(app.theme.highlight_fg)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_widget(tabs, chunks[0]);

        let content_area = chunks[1];
        let active_tab_idx = app.active_main_tab;

        if let Some(tab) = app.main_tabs.get_mut(active_tab_idx) {
            let is_active = app.active_target == crate::ActiveTarget::Editor;
            let border_style = if is_active {
                Style::default().fg(app.theme.highlight_fg)
            } else {
                Style::default().fg(app.theme.text_fg)
            };

            match &mut tab.content {
                MainWidgetContent::Editor(editor) => {
                    let content_block = Block::default()
                        .borders(Borders::ALL)
                        .border_style(border_style)
                        .bg(app.theme.primary_bg);
                    editor.render_with_block(f, content_area, content_block);
                }
                MainWidgetContent::Welcome(welcome_widget) => {
                    welcome_widget.render(f, content_area, &app.theme);
                }
                // SettingsEditor needs a special dance to avoid borrow checker issues
                // because its render method needs `&mut App`.
                MainWidgetContent::SettingsEditor(_) => {
                    render_settings_editor(f, content_area, app, active_tab_idx);
                }
            }
        }
    }
}

/// Helper to render the settings editor, working around the borrow checker.
fn render_settings_editor(f: &mut Frame, area: Rect, app: &mut App, tab_idx: usize) {
    // Temporarily take ownership of the SettingsEditor to avoid double borrow.
    let mut content = std::mem::replace(
        &mut app.main_tabs[tab_idx].content,
        // This is a dummy value that will be replaced immediately.
        // Using Welcome as it's the simplest to construct.
        MainWidgetContent::Welcome(welcome_widget::WelcomeWidget::new()),
    );

    if let MainWidgetContent::SettingsEditor(se) = &mut content {
        se.render(f, area, app);
    }

    // Put the real content back.
    app.main_tabs[tab_idx].content = content;
}
