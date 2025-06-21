use ratatui::{layout::{Constraint, Direction, Layout, Rect}, Frame, style::{Style, Color}, widgets::{Block, Borders}};
pub mod item; // Declare the sub-module
use self::item::{BottomBarItem, GitInfoItem, CurrentTimeItem, ResourceUsageItem}; // Import from its own sub-module

pub struct BottomBar {
    items: Vec<Box<dyn BottomBarItem>>,
}

impl Default for BottomBar {
    fn default() -> Self {
        Self::new()
    }
}

impl BottomBar {
    pub fn new() -> Self {
        Self {
            items: vec![
                Box::new(GitInfoItem::new()),
                Box::new(CurrentTimeItem::new()),
                Box::new(ResourceUsageItem::new()),
            ],
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, _is_active: bool) {
        let block = Block::default()
            .borders(Borders::NONE)
            .border_style(Style::default().fg(Color::DarkGray));
        f.render_widget(&block, area);

        let inner_area = block.inner(area);
        if inner_area.width == 0 || inner_area.height == 0 {
            return;
        }

        let num_items = self.items.len();
        if num_items == 0 {
            return;
        }

        let constraints: Vec<Constraint> = (0..num_items)
            .map(|_| Constraint::Percentage(100 / num_items as u16))
            .collect();
        
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(inner_area);
        
        for (i, item) in self.items.iter().enumerate() {
            if let Some(chunk_area) = chunks.get(i) {
                item.render_item(f, *chunk_area);
            }
        }
    }
}