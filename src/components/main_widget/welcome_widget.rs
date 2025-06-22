use crate::theme::Theme;
use ratatui::{prelude::*, widgets::Paragraph};

const LOGO: &[&str] = &[
    "██╗███╗ ██╗███████╗",
    "██║████╗██║██╔════╝",
    "██║██╔████║█████╗  ",
    "██║██║╚███║██╔══╝  ",
    "██║██║ ╚██║███████╗",
    "╚═╝╚═╝  ╚═╝╚══════╝",
    "",
    "Welcome to Inf-Edit",
];

#[derive(Default)]
pub struct WelcomeWidget;

impl WelcomeWidget {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let parent_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let logo_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0)])
            .vertical_margin(parent_layout[0].height.saturating_sub(10) / 2)
            .split(parent_layout[0])[0];

        let logo_text: Vec<Line> = LOGO.iter().map(|&s| Line::from(s).centered()).collect();

        let keybinds = [
            ("Ctrl-P", "Show Command Palette"),
            ("Ctrl-B", "Toggle File Explorer"),
            ("Ctrl-N", "New Tab"),
            ("Ctrl-W", "Close Tab"),
        ];

        let keybinds_lines: Vec<Line> = keybinds
            .iter()
            .map(|(key, desc)| {
                Line::from(vec![
                    Span::styled(
                        format!("{:<12}", key), // Left-align key in a fixed width
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(theme.highlight_fg),
                    ),
                    Span::raw(*desc),
                ])
                .alignment(Alignment::Center)
            })
            .collect();
        let keybinds_text = [vec![Line::from("")], keybinds_lines].concat();

        let logo_paragraph = Paragraph::new(logo_text).style(
            Style::default()
                .fg(theme.text_fg)
                .add_modifier(Modifier::ITALIC),
        );
        let keybinds_paragraph =
            Paragraph::new(keybinds_text).style(Style::default().fg(theme.text_fg));

        f.render_widget(logo_paragraph, logo_area);
        f.render_widget(keybinds_paragraph, parent_layout[1]);
    }
}
