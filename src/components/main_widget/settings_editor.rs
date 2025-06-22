use crate::{app::App, theme::Theme};
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

    pub fn render(&mut self, f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
        let mut items = Vec::new();

        items.push(ListItem::new(Line::from(Span::styled(
            "[Keybindings]",
            Style::default().fg(theme.highlight_fg).add_modifier(Modifier::BOLD),
        ))));
        for (key, action) in &app.config.keybindings.global {
            items.push(ListItem::new(format!("{:<20} -> {}", key, action)));
        }

        items.push(ListItem::new("")); // Spacer
        items.push(ListItem::new(Line::from(Span::styled(
            "[Theme]",
            Style::default().fg(theme.highlight_fg).add_modifier(Modifier::BOLD),
        ))));
        items.push(ListItem::new(format!("primary_bg:     {}", app.config.theme.primary_bg)));
        items.push(ListItem::new(format!("secondary_bg:   {}", app.config.theme.secondary_bg)));
        items.push(ListItem::new(format!("text_fg:        {}", app.config.theme.text_fg)));
        items.push(ListItem::new(format!("highlight_fg:   {}", app.config.theme.highlight_fg)));
        items.push(ListItem::new(format!("highlight_bg:   {}", app.config.theme.highlight_bg)));

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Settings"))
            .highlight_style(Style::default().bg(theme.highlight_bg).fg(theme.text_fg))
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, area, &mut self.state);
    }

    // Note: Key handling for editing values is not implemented in this step.
    // pub fn handle_key(&mut self, key: KeyEvent, app: &mut App) {
    //     match key.code {
    //         KeyCode::Up => self.previous(),
    //         KeyCode::Down => self.next(app.config.keybindings.global.len() + ...),
    //         _ => {}
    //     }
    // }
}