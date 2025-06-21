use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Tabs},
};

pub mod editor;
use self::editor::Editor;
use crate::{ActiveTarget, Tab};

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
        Self {
            editor_tabs,
            active_editor_tab_index,
            active_target,
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(0)]) // Tabs, Editor
            .split(area);

        let editor_tab_titles: Vec<Line> = self.editor_tabs
            .iter()
            .enumerate()
            .map(|(i, tab)| {
                let is_active_and_focused =
                    i == self.active_editor_tab_index && self.active_target == ActiveTarget::Editor;

                if is_active_and_focused {
                    // Prepend a styled `*` to the title if the editor tab is active and focused
                    Line::from(vec![Span::raw("*").yellow(), Span::raw(" "), Span::raw(&tab.title)])
                } else {
                    Line::from(tab.title.as_str())
                }
            })
            .collect();

        let tabs = Tabs::new(editor_tab_titles)
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .title("Editor Tabs"),
            )
            .highlight_style(Style::default().fg(Color::Green));
        f.render_widget(tabs, chunks[0]);

        if let Some(active_editor) = self.editor_tabs.get_mut(self.active_editor_tab_index) {
            let editor_block = Block::default(); // No borders for the editor content itself
            active_editor
                .content
                .render_with_block(f, chunks[1], editor_block);
        }
    }
}
