use ratatui::{
    prelude::*,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Tabs},
};

pub mod editor;
pub mod settings_editor;
use crate::{settings::Config, theme::Theme};
use crate::MainWidgetContent;
use crate::Tab;

pub struct MainWidget<'a> {
    pub editor_tabs: &'a mut Vec<Tab<MainWidgetContent>>,
    pub active_editor_tab_index: usize,
    pub is_active: bool,
}

impl<'a> MainWidget<'a> {
    pub fn new(
        editor_tabs: &'a mut Vec<Tab<MainWidgetContent>>,
        active_editor_tab_index: usize,
        is_active: bool,
    ) -> Self {
        Self {
            editor_tabs,
            active_editor_tab_index,
            is_active,
        }
    }

    /// Renders the editor tabs at the top of its area.
    pub fn render_tabs(&self, f: &mut Frame, area: Rect) {
        let titles: Vec<String> = self.editor_tabs.iter().map(|t| t.title.clone()).collect();
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
    pub fn render_content(&mut self, f: &mut Frame, area: Rect, config: &Config, theme: &Theme) {
        if let Some(active_editor_tab) = self.editor_tabs.get_mut(self.active_editor_tab_index) {
            let border_style = if self.is_active {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };
            match &mut active_editor_tab.content {
                MainWidgetContent::Editor(editor) => {
                    let content_block = Block::default()
                        .borders(Borders::ALL)
                        .border_style(border_style)
                        .title("Editor"); // Added title for clarity
                    editor.render_with_block(f, area, content_block);
                }
                MainWidgetContent::SettingsEditor(se) => se.render(f, area, config, theme),
            }
        }
    }
}
