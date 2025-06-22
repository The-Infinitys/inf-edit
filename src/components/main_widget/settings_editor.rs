use crate::{settings::Config, theme::Theme};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState},
};
pub struct SettingsEditor {
    pub state: ListState,
}

impl Default for SettingsEditor {
    fn default() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self { state }
    }
}

impl SettingsEditor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, config: &Config, theme: &Theme) {
        let mut items = Vec::new();

        items.push(ListItem::new(Line::from(Span::styled(
            "[Keybindings]",
            Style::default()
                .fg(theme.highlight_fg)
                .add_modifier(Modifier::BOLD),
        ))));
        for (key, action) in &config.keybindings.global {
            items.push(ListItem::new(format!("{:<20} -> {}", key, action)));
        }

        items.push(ListItem::new("")); // Spacer
        items.push(ListItem::new(Line::from(Span::styled(
            "[Theme]",
            Style::default()
                .fg(theme.highlight_fg)
                .add_modifier(Modifier::BOLD),
        ))));
        items.push(ListItem::new(format!(
            "primary_bg:     {}",
            config.theme.primary_bg
        )));
        items.push(ListItem::new(format!(
            "secondary_bg:   {}",
            config.theme.secondary_bg
        )));
        items.push(ListItem::new(format!(
            "text_fg:        {}",
            config.theme.text_fg
        )));
        items.push(ListItem::new(format!(
            "highlight_fg:   {}",
            config.theme.highlight_fg
        )));
        items.push(ListItem::new(format!(
            "highlight_bg:   {}",
            config.theme.highlight_bg
        )));

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Settings"))
            .highlight_style(Style::default().bg(theme.highlight_bg).fg(theme.text_fg))
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, area, &mut self.state);
    }

    pub fn handle_key(&mut self, key: KeyEvent, config: &Config) {
        // Calculate total items dynamically based on the config
        let mut total_items = 0;
        total_items += 1; // "[Keybindings]" header
        total_items += config.keybindings.global.len(); // Actual keybindings
        total_items += 1; // Empty spacer
        total_items += 1; // "[Theme]" header
        total_items += 5; // Theme properties (primary_bg, secondary_bg, text_fg, highlight_fg, highlight_bg)

        match key.code {
            KeyCode::Up => self.previous(total_items),
            KeyCode::Down => self.next(total_items),
            _ => {}
        }
    }

    fn previous(&mut self, total_items: usize) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    total_items.saturating_sub(1) // Go to last item
                } else {
                    i - 1
                }
            }
            None => 0, // Select first item if nothing is selected
        };
        self.state.select(Some(i));
    }

    fn next(&mut self, total_items: usize) {
        let i = match self.state.selected() {
            Some(i) => (i + 1) % total_items, // Cycle through items
            None => 0,                        // Select first item if nothing is selected
        };
        self.state.select(Some(i));
    }
}
