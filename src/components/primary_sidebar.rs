pub mod file_view;
pub use self::file_view::FileView;

pub mod component;
pub mod git; // Add this line to expose the git module
pub mod search;
use crate::app::App;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem},
};

pub struct PrimarySidebar;

impl Default for PrimarySidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl PrimarySidebar {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, f: &mut Frame, area: Rect, app: &mut App) {
        let is_active = app.active_target == crate::ActiveTarget::PrimarySideBar;

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(4), Constraint::Min(0)])
            .split(area);

        // Render Vertical Tabs
        let items: Vec<ListItem> = app
            .primary_sidebar_components
            .iter()
            .enumerate()
            .map(|(i, tab)| {
                let icon = match &tab.content {
                    component::PrimarySidebarComponent::FileView(_) => "📁",
                    component::PrimarySidebarComponent::Search(_) => "🔍",
                    component::PrimarySidebarComponent::Git(_) => "🐙",
                };
                let mut list_item = ListItem::new(icon);
                if i == app.active_primary_sidebar_tab {
                    list_item = list_item.style(
                        Style::default()
                            .bg(app.theme.highlight_bg)
                            .fg(app.theme.highlight_fg),
                    );
                }
                list_item
            })
            .collect();

        let tabs_list = List::new(items).block(
            Block::default()
                .borders(Borders::RIGHT)
                .bg(app.theme.secondary_bg),
        );
        f.render_widget(tabs_list, chunks[0]);

        // Render Content
        if let Some(active_component) = app
            .primary_sidebar_components
            .get_mut(app.active_primary_sidebar_tab)
        {
            active_component
                .content
                .render(f, chunks[1], is_active, &app.theme);
        }
    }
}
