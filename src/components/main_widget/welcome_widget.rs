use crate::theme::Theme;
use ratatui::{
    prelude::*,
    widgets::Paragraph,
};

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

        let keybinds_text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Ctrl-P", Style::default().add_modifier(Modifier::BOLD).fg(theme.highlight_fg)),
                Span::raw("      Show Command Palette"),
            ]).centered(),
            Line::from(vec![
                Span::styled("  Ctrl-B", Style::default().add_modifier(Modifier::BOLD).fg(theme.highlight_fg)),
                Span::raw("      Toggle File Explorer"),
            ]).centered(),
            Line::from(vec![
                Span::styled("  Ctrl-W", Style::default().add_modifier(Modifier::BOLD).fg(theme.highlight_fg)),
                Span::raw("      Close Active Tab"),
            ]).centered(),
        ];

        let logo_paragraph = Paragraph::new(logo_text)
            .style(Style::default().fg(theme.text_fg).add_modifier(Modifier::ITALIC));
        let keybinds_paragraph =
            Paragraph::new(keybinds_text).style(Style::default().fg(theme.text_fg));

        f.render_widget(logo_paragraph, logo_area);
        f.render_widget(keybinds_paragraph, parent_layout[1]);
    }
}