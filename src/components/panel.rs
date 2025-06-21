use ratatui::{
    prelude::*,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Tabs},
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
        let titles: Vec<String> = self.terminal_tabs.iter().map(|t| t.title.clone()).collect();
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::BOTTOM))
            .select(self.active_terminal_tab_index)
            .style(Style::default().fg(Color::Gray))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_widget(tabs, area);
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
