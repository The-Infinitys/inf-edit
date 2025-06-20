use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::{ActiveTarget, Tab}; // Assuming Tab and ActiveTarget are in crate root (lib.rs)
use super::editor::Editor;

pub struct MainWidget<'a> {
    pub editor_tabs: &'a mut Vec<Tab<Editor>>,
    pub active_editor_tab_index: usize,
    pub active_target: ActiveTarget,
}

impl<'a> MainWidget<'a> {
    pub fn new(
        editor_tabs: &'a mut Vec<Tab<Editor>>,
        active_editor_tab_index: usize,
        active_target: ActiveTarget,
    ) -> Self {
        Self { editor_tabs, active_editor_tab_index, active_target }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(0)]) // Tabs, Editor
            .split(area);

        let editor_tab_titles: Vec<Line> = self.editor_tabs.iter().enumerate().map(|(i, tab)| {
            let mut title = tab.title.clone();
            if i == self.active_editor_tab_index && self.active_target == ActiveTarget::Editor {
                title = format!("*{}", title);
            }
            Line::from(Span::raw(title))
        }).collect();

        let tabs = Tabs::new(editor_tab_titles)
            .block(Block::default().borders(Borders::BOTTOM).title("Editor Tabs"))
            .highlight_style(Style::default().fg(Color::Green));
        f.render_widget(tabs, chunks[0]);

        if let Some(active_editor) = self.editor_tabs.get_mut(self.active_editor_tab_index) {
            let editor_block = Block::default(); // No borders for the editor content itself
            active_editor.content.render_with_block(f, chunks[1], editor_block);
        }
    }
}