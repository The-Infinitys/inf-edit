pub mod file_view;
pub use self::file_view::FileView;

pub mod search;
pub mod component;
use self::component::PrimarySidebarComponent;
use crate::Tab;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem},
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

    /// Renders the vertical tabs on the far left.
    pub fn render_tabs(&mut self, f: &mut Frame, area: Rect) {
        let tab_titles: Vec<ListItem> = self
            .components
            .iter()
            .enumerate()
            .map(|(i, tab)| {
                let icon = match tab.content {
                    PrimarySidebarComponent::FileView(_) => "📁",
                    PrimarySidebarComponent::Search(_) => "🔍",
                };
                let text = Span::from(format!("{} {}", icon, tab.title));
                let mut list_item = ListItem::new(text);
                if i == self.active_tab_index {
                    list_item = list_item.style(Style::default().fg(Color::Green).bold());
                }
                list_item
            })
            .collect();

        let tabs_list = List::new(tab_titles)
            .block(Block::default().title("Tabs").borders(Borders::RIGHT));

        f.render_widget(tabs_list, area);
    }

    /// Renders the content of the active tab.
    pub fn render_content(&mut self, f: &mut Frame, area: Rect) {
        let border_style = if self.is_active {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        };

        if let Some(active_component) = self.components.get_mut(self.active_tab_index) {
            // The content area itself doesn't need its own border if it's inside the main UI layout
            let content_block = Block::default().border_style(border_style);
            let content_area = content_block.inner(area);
            f.render_widget(content_block, area);

            active_component.content.render(f, content_area, self.is_active);
        }
    }
}
