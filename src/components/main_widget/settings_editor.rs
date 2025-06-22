use crate::{app::App, settings::Config, theme::Theme};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

/// Represents which specific field is currently being edited.
#[derive(Debug, Clone)]
enum EditableField {
    Keybinding(String), // The key of the keybinding
    ThemePreset,
    ThemePrimaryBg,
    ThemeSecondaryBg,
    ThemeTextFg,
    ThemeHighlightFg,
    ThemeHighlightBg,
}

/// Holds the state for when a value is being edited.
#[derive(Debug, Clone)]
struct EditingState {
    field: EditableField,
    input_buffer: String,
}

pub struct SettingsEditor {
    pub state: ListState,
    editing_state: Option<EditingState>,
}

impl Default for SettingsEditor {
    fn default() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            state,
            editing_state: None,
        }
    }
}

impl SettingsEditor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, app: &mut App) {
        let config = &app.config;
        let theme = &app.theme;
        let mut items = Vec::new();

        items.push(ListItem::new(Line::from(Span::styled(
            "[Keybindings]",
            Style::default()
                .fg(theme.highlight_fg)
                .add_modifier(Modifier::BOLD),
        ))));

        let mut sorted_keys: Vec<_> = config.keybindings.global.keys().cloned().collect();
        sorted_keys.sort();
        for key in &sorted_keys {
            let action = config.keybindings.global.get(key).unwrap();
            items.push(ListItem::new(format!("{:<20} -> {}", key, action.clone())));
        }

        items.push(ListItem::new("")); // Spacer
        items.push(ListItem::new(Line::from(Span::styled(
            "[Theme]",
            Style::default()
                .fg(theme.highlight_fg)
                .add_modifier(Modifier::BOLD),
        ))));
        items.push(ListItem::new(format!(
            "preset:         {}",
            config.theme.preset
        )));
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
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Settings")
                    .bg(theme.primary_bg),
            )
            .highlight_style(Style::default().bg(theme.highlight_bg).fg(theme.text_fg))
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, area, &mut self.state);

        // Render the editing popup if we are in editing mode
        if let Some(editing_state) = &self.editing_state {
            let popup_width = 50;
            let popup_height = 3;
            let popup_area = Rect {
                x: area.x + (area.width.saturating_sub(popup_width)) / 2,
                y: area.y + (area.height.saturating_sub(popup_height)) / 2,
                width: popup_width,
                height: popup_height,
            };
            let text = editing_state.input_buffer.as_str();
            let paragraph = Paragraph::new(text)
                .style(Style::default().fg(theme.text_fg).bg(theme.secondary_bg))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Edit Value (Enter to save, Esc to cancel)"),
                );

            f.render_widget(Clear, popup_area); // Clear the area behind the popup
            f.render_widget(paragraph, popup_area);
            // Set the cursor position inside the popup
            f.set_cursor_position((
                popup_area.x + Span::from(text).width() as u16 + 1,
                popup_area.y + 1,
            ));
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, app: &mut App) {
        if let Some(editing_state) = &mut self.editing_state {
            // --- Editing Mode ---
            match key.code {
                KeyCode::Char(c) => {
                    editing_state.input_buffer.push(c);
                }
                KeyCode::Backspace => {
                    editing_state.input_buffer.pop();
                }
                KeyCode::Esc => {
                    self.editing_state = None;
                }
                KeyCode::Enter => {
                    let new_value = editing_state.input_buffer.clone();
                    match &editing_state.field {
                        EditableField::Keybinding(k) => {
                            if let Some(v) = app.config.keybindings.global.get_mut(k) {
                                *v = new_value;
                            }
                        }
                        EditableField::ThemePreset => app.config.theme.preset = new_value,
                        EditableField::ThemePrimaryBg => app.config.theme.primary_bg = new_value,
                        EditableField::ThemeSecondaryBg => {
                            app.config.theme.secondary_bg = new_value
                        }
                        EditableField::ThemeTextFg => app.config.theme.text_fg = new_value,
                        EditableField::ThemeHighlightFg => {
                            app.config.theme.highlight_fg = new_value
                        }
                        EditableField::ThemeHighlightBg => {
                            app.config.theme.highlight_bg = new_value
                        }
                    }
                    // Re-apply the theme from the modified config
                    app.theme = Theme::from_config(&app.config.theme);
                    // Save the updated config to file
                    if let Err(e) = app.config.save() {
                        // In a real app, you'd want to show this error to the user
                        eprintln!("Failed to save config: {}", e);
                    }
                    self.editing_state = None;
                }
                _ => {}
            }
        } else {
            // --- Navigation Mode ---
            let total_items = self.get_total_items(&app.config);
            match key.code {
                KeyCode::Up => self.previous(total_items),
                KeyCode::Down => self.next(total_items),
                KeyCode::Enter => {
                    if let Some(selected_index) = self.state.selected() {
                        if let Some(field) = self.index_to_field(selected_index, &app.config) {
                            // Get the current value and enter editing mode
                            let current_value = match &field {
                                EditableField::Keybinding(k) => app
                                    .config
                                    .keybindings
                                    .global
                                    .get(k)
                                    .cloned()
                                    .unwrap_or_default(),
                                EditableField::ThemePreset => app.config.theme.preset.clone(),
                                EditableField::ThemePrimaryBg => {
                                    app.config.theme.primary_bg.clone()
                                }
                                EditableField::ThemeSecondaryBg => {
                                    app.config.theme.secondary_bg.clone()
                                }
                                EditableField::ThemeTextFg => app.config.theme.text_fg.clone(),
                                EditableField::ThemeHighlightFg => {
                                    app.config.theme.highlight_fg.clone()
                                }
                                EditableField::ThemeHighlightBg => {
                                    app.config.theme.highlight_bg.clone()
                                }
                            };
                            self.editing_state = Some(EditingState {
                                field,
                                input_buffer: current_value,
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Converts a list index to a specific editable field.
    fn index_to_field(&self, index: usize, config: &Config) -> Option<EditableField> {
        let keybinding_start = 1;
        let keybinding_end = keybinding_start + config.keybindings.global.len();
        let theme_header_index = keybinding_end + 1;

        if (keybinding_start..keybinding_end).contains(&index) {
            let mut keys: Vec<_> = config.keybindings.global.keys().cloned().collect();
            keys.sort();
            let key = keys.get(index - keybinding_start)?.clone();
            return Some(EditableField::Keybinding(key));
        }

        match index {
            i if i == theme_header_index + 1 => Some(EditableField::ThemePreset),
            i if i == theme_header_index + 2 => Some(EditableField::ThemePrimaryBg),
            i if i == theme_header_index + 3 => Some(EditableField::ThemeSecondaryBg),
            i if i == theme_header_index + 4 => Some(EditableField::ThemeTextFg),
            i if i == theme_header_index + 5 => Some(EditableField::ThemeHighlightFg),
            i if i == theme_header_index + 6 => Some(EditableField::ThemeHighlightBg),
            _ => None, // Headers and spacers are not editable
        }
    }

    /// Gets the total number of visible items in the settings list.
    fn get_total_items(&self, config: &Config) -> usize {
        1  // [Keybindings] header
        + config.keybindings.global.len()
        + 1  // Spacer
        + 1  // [Theme] header
        + 6 // Theme properties (preset + 5 colors)
    }

    /// Moves the selection to the previous item.
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

    /// Moves the selection to the next item.
    fn next(&mut self, total_items: usize) {
        let i = match self.state.selected() {
            Some(i) => (i + 1) % total_items, // Cycle through items
            None => 0,                        // Select first item if nothing is selected
        };
        self.state.select(Some(i));
    }
}
