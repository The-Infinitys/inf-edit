use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::theme::Theme;

#[derive(PartialEq, Eq)]
pub enum PopupResult {
    Confirm,
    Cancel,
    None,
}

pub struct Popup {
    title: String,
    message: String,
    confirm_selected: bool,
}

impl Popup {
    pub fn new(title: String, message: String) -> Self {
        Self {
            title,
            message,
            confirm_selected: true,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> PopupResult {
        match key.code {
            KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                self.confirm_selected = !self.confirm_selected;
                PopupResult::None
            }
            KeyCode::Enter => {
                if self.confirm_selected {
                    PopupResult::Confirm
                } else {
                    PopupResult::Cancel
                }
            }
            KeyCode::Esc => PopupResult::Cancel,
            _ => PopupResult::None,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let block = Block::default()
            .title(self.title.as_str())
            .borders(Borders::ALL)
            .style(Style::default().bg(theme.secondary_bg).fg(theme.text_fg));

        let area = centered_rect(60, 20, area);
        f.render_widget(Clear, area); // Clear the area behind the popup
        f.render_widget(&block, area);

        let inner_area = block.inner(area);
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(inner_area);

        let message = Paragraph::new(self.message.as_str())
            .style(Style::default().fg(theme.text_fg))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(message, layout[0]);

        let buttons = Line::from(vec![
            Span::styled(
                " Cancel ",
                Style::default()
                    .fg(if !self.confirm_selected {
                        theme.highlight_fg
                    } else {
                        theme.text_fg
                    })
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                " Confirm ",
                Style::default()
                    .fg(if self.confirm_selected {
                        theme.highlight_fg
                    } else {
                        theme.text_fg
                    })
                    .add_modifier(Modifier::BOLD),
            ),
        ])
        .centered();
        let buttons_paragraph =
            Paragraph::new(buttons).alignment(ratatui::layout::Alignment::Center);
        f.render_widget(buttons_paragraph, layout[1]);
    }
}

// Helper function to center the popup
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
