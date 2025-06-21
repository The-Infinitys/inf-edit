use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders, List, ListItem},
};

#[derive(Default)]
pub struct HelpWidget {
    is_visible: bool,
}

impl HelpWidget {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle_visibility(&mut self) -> bool {
        self.is_visible = !self.is_visible;
        self.is_visible // Return the new state
    }

    pub fn render(&self, f: &mut Frame, app_area: Rect) {
        if self.is_visible {
            // The help widget now renders directly into the area provided by the sidebar.
            // No need for centering logic.
            let help_items = vec![
                ListItem::new("Ctrl+Q: Quit"),
                ListItem::new("Ctrl+B: Toggle File View"),
                ListItem::new("Ctrl+J: Toggle Terminal"),
                ListItem::new("Ctrl+K: Switch Focus"),
                ListItem::new("Ctrl+N: New Editor"),
                ListItem::new("Ctrl+T: Next Editor/Term Tab"),
                ListItem::new("Alt+H / Alt+L: Prev/Next Sidebar Tab"),
                ListItem::new("Ctrl+Alt+B: Toggle Help"),
            ];

            let help_list = List::new(help_items)
                .block(Block::default().title("Help").borders(Borders::ALL));

            f.render_widget(help_list, app_area);
        }
    }
}
