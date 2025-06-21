pub mod component;
pub mod help_widget;
use self::component::SecondarySidebarComponent;
use crate::Tab;
use ratatui::{
    Frame,
    layout::Rect,
    prelude::*,
    widgets::{Block, Borders, Tabs},
};

pub struct SecondarySideBar<'a> {
    pub components: &'a mut Vec<Tab<SecondarySidebarComponent>>,
    pub active_tab_index: usize,
    is_active: bool,
}

impl<'a> SecondarySideBar<'a> {
    pub fn new(
        components: &'a mut Vec<Tab<SecondarySidebarComponent>>,
        active_tab_index: usize,
        is_active: bool,
    ) -> Self {
        Self {
            components,
            active_tab_index,
            is_active,
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let border_style = if self.is_active {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        };

        let outer_block = Block::default()
            .title("Secondary Sidebar")
            .borders(Borders::ALL)
            .border_style(border_style);
        let inner_area = outer_block.inner(area);
        f.render_widget(outer_block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(0)]) // Tabs, Content
            .split(inner_area);

        let tab_titles: Vec<Line> = self
            .components
            .iter()
            .map(|tab| Line::from(tab.title.as_str()))
            .collect();

        let tabs = Tabs::new(tab_titles)
            .block(Block::default().borders(Borders::BOTTOM))
            .select(self.active_tab_index)
            .highlight_style(Style::default().fg(Color::Green).bold());
        f.render_widget(tabs, chunks[0]);

        if let Some(active_component) = self.components.get_mut(self.active_tab_index) {
            active_component.content.render(f, chunks[1]);
        }
    }
}
