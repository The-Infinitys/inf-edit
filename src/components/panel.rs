use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    text::{Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use super::term::Term; // Assuming Term is in the same module or accessible
use crate::{ActiveTarget, Tab};

pub struct Panel<'a> {
    pub terminal_tabs: &'a mut Vec<Tab<Term>>,
    pub active_terminal_tab_index: usize,
    pub active_target: ActiveTarget,
}

impl<'a> Panel<'a> {
    pub fn new(
        terminal_tabs: &'a mut Vec<Tab<Term>>,
        active_terminal_tab_index: usize,
        active_target: ActiveTarget,
    ) -> Self {
        Self { terminal_tabs, active_terminal_tab_index, active_target }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)]) // Terminal, Terminal Tabs
            .split(area);

        // Render active terminal content
        if let Some(active_term) = self.terminal_tabs.get_mut(self.active_terminal_tab_index) {
            let term_block = Block::default().title(format!("Terminal: {}", active_term.title));
            active_term.content.render_with_block(f, chunks[0], term_block);
        }

        // Render terminal tabs list
        let items: Vec<ListItem> = self.terminal_tabs.iter().enumerate().map(|(i, tab)| {
            let mut style = Style::default();
            if i == self.active_terminal_tab_index && self.active_target == ActiveTarget::Panel {
                style = style.fg(Color::Green).add_modifier(Modifier::BOLD);
            }
            ListItem::new(Span::styled(tab.title.clone(), style))
        }).collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Terminal Tabs"));
        f.render_widget(list, chunks[1]);
    }
}