pub mod file_view;
pub use self::file_view::FileView;

pub mod component;
use self::component::PrimarySidebarComponent;
use crate::Tab;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Tabs},
};

pub struct PrimarySideBar<'a> {
    pub components: &'a mut Vec<Tab<PrimarySidebarComponent>>,
    pub active_tab_index: usize,
    is_active: bool,
}

impl<'a> PrimarySideBar<'a> {
    pub fn new(
        components: &'a mut Vec<Tab<PrimarySidebarComponent>>,
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
            .title("Primary Sidebar")
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
            active_component.content.render(f, chunks[1], self.is_active);
        }
    }
}
