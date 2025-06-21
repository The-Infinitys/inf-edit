use ratatui::{
    prelude::*,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem},
};

pub mod term;
use self::term::Term;
use crate::Tab;

pub struct Panel<'a> {
    pub terminal_tabs: &'a mut Vec<Tab<Term>>,
    pub active_terminal_tab_index: usize,
    pub is_active: bool,
}

impl<'a> Panel<'a> {
    pub fn new(
        terminal_tabs: &'a mut Vec<Tab<Term>>,
        active_terminal_tab_index: usize,
        is_active: bool,
    ) -> Self {
        Self {
            terminal_tabs,
            active_terminal_tab_index,
            is_active,
        }
    }

    /// Renders the terminal tabs at the top of its area.
    pub fn render_tabs(&self, f: &mut Frame, area: Rect) {
        let tab_titles: Vec<ListItem> = self
            .terminal_tabs
            .iter()
            .enumerate()
            .map(|(i, tab)| {
                let mut list_item = ListItem::new(tab.title.clone());
                if i == self.active_terminal_tab_index {
                    list_item = list_item.style(Style::default().fg(Color::Yellow).bold());
                }
                list_item
            })
            .collect();

        let tabs_list = List::new(tab_titles)
            .block(Block::default().title("Terminals").borders(Borders::ALL));

        f.render_widget(tabs_list, area);
    }

    /// Renders the content of the active terminal.
    pub fn render_content(&mut self, f: &mut Frame, area: Rect) {
        if let Some(active_term_tab) = self.terminal_tabs.get_mut(self.active_terminal_tab_index) {
            let border_style = if self.is_active {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };
            let content_block = Block::default().borders(Borders::ALL).border_style(border_style).title("Terminal"); // Added title for clarity
            active_term_tab.content.render_with_block(f, area, content_block);
        }
    }
}
