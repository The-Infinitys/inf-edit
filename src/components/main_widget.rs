use ratatui::{
    prelude::*,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Tabs},
};

pub mod editor;
use self::editor::Editor;
use crate::Tab;

pub struct MainWidget<'a> {
    pub editor_tabs: &'a mut Vec<Tab<Editor>>,
    pub active_editor_tab_index: usize,
    pub is_active: bool,
}

impl<'a> MainWidget<'a> {
    pub fn new(
        editor_tabs: &'a mut Vec<Tab<Editor>>,
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
            .block(Block::default().borders(Borders::BOTTOM))
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
    pub fn render_content(&mut self, f: &mut Frame, area: Rect) {
        if let Some(active_editor_tab) = self.editor_tabs.get_mut(self.active_editor_tab_index) {
            let border_style = if self.is_active {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };
            let content_block = Block::default().borders(Borders::ALL).border_style(border_style).title("Editor"); // Added title for clarity
            active_editor_tab.content.render_with_block(f, area, content_block);
        }
    }
}
