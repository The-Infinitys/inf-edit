use ratatui::{
    layout::{Rect, Constraint, Layout, Direction, Alignment},
    style::{Style, Color},
    text::{Line, Text},
    widgets::{Paragraph, Block, Borders},
    Frame,
};

pub struct HelpWidget {
    pub is_visible: bool,
}

impl HelpWidget {
    pub fn new() -> Self {
        Self { is_visible: false }
    }

    pub fn toggle_visibility(&mut self) {
        self.is_visible = !self.is_visible;
    }

    pub fn render(&self, f: &mut Frame, app_area: Rect) {
        if self.is_visible {
            // Calculate position for centered popup
            let popup_area = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Length(10), // Popup height
                    Constraint::Percentage(30), // Adjusted to make it centered better with 10 height
                ])
                .split(app_area)[1];

            let popup_area = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Length(50), // Popup width
                    Constraint::Percentage(30), // Adjusted
                ])
                .split(popup_area)[1];

            let help_text = Text::from(vec![
                Line::from("Ctrl+Q: Quit"),
                Line::from("Ctrl+B: Toggle File View"),
                Line::from("Ctrl+J: Toggle Terminal Panel"),
                Line::from("Ctrl+K: Switch Editor/Panel Focus"),
                Line::from("Ctrl+N: New Editor Tab"),
                Line::from("Ctrl+T: Switch Active Tab (Editor/Panel)"),
                Line::from("Ctrl+H: Toggle Help (this)"),
            ]);

            let help_paragraph = Paragraph::new(help_text)
                .block(Block::default().title("Help").borders(Borders::ALL))
                .style(Style::default().fg(Color::White).bg(Color::DarkGray)) // Changed bg for better visibility
                .alignment(Alignment::Left);

            f.render_widget(help_paragraph, popup_area);
        }
    }
}