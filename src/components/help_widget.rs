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
            let popup_height = 10u16; // Desired popup height
            let popup_width = 50u16;  // Desired popup width

            let vertical_margin = (app_area.height.saturating_sub(popup_height)) / 2;
            let horizontal_margin = (app_area.width.saturating_sub(popup_width)) / 2;

            let popup_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(vertical_margin),
                    Constraint::Length(popup_height),
                    Constraint::Length(vertical_margin),
                ])
                .split(app_area);

            let popup_area = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(horizontal_margin),
                    Constraint::Length(popup_width),
                    Constraint::Length(horizontal_margin),
                ])
                .split(popup_layout[1])[1]; // Apply horizontal layout to the middle vertical chunk

            let help_text = Text::from(vec![
                Line::from("Ctrl+Q: Quit"),
                Line::from("Ctrl+B: Toggle File View"),
                Line::from("Ctrl+J: Toggle/Focus Terminal Panel"),
                Line::from("Ctrl+Shift+J: New Terminal Tab"),
                Line::from("Ctrl+K: Switch Editor/Panel Focus"),
                Line::from("Ctrl+N: New Editor Tab"),
                Line::from("Ctrl+T: Switch Active Tab (Editor/Panel)"),
                Line::from("Ctrl+Shift+Left/Right: Switch Terminal Tabs (when Panel active)"),
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